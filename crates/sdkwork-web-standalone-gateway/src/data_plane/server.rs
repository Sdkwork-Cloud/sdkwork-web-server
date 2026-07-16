use std::{future::Future, io, net::SocketAddr, sync::Arc, time::Duration};

use axum::{http::StatusCode, Router};
use axum_server::{
    accept::{Accept, DefaultAcceptor},
    service::{MakeService, SendService},
    tls_rustls::{RustlsAcceptor, RustlsConfig},
};
use hyper::{body::Incoming, Request};
use hyper_util::{
    rt::{TokioExecutor, TokioIo, TokioTimer},
    server::conn::auto::Builder,
    service::TowerToHyperService,
};
use sdkwork_webserver_core::{
    CompiledWebServerApp, ListenerConfig, ListenerProtocol, WebServerLimits,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
    sync::watch,
    task::JoinSet,
    time::timeout,
};
use tower_http::timeout::TimeoutLayer;

use super::{
    connection_limit::{ConnectionLimitedStream, ConnectionLimiter},
    handler::route_request,
    http1_wire::Http1WireGuardAcceptor,
    http2_wire::Http2WireGuardAcceptor,
    io_timeout::WriteTimeoutAcceptor,
    keep_alive_timeout::Http1KeepAliveTimeoutAcceptor,
    runtime::RuntimeGeneration,
    tls::build_tls_config,
    DataPlaneError, DataPlaneRuntime, ListenerState,
};

struct PreparedListener {
    config: ListenerConfig,
    socket: TcpListener,
    address: SocketAddr,
    tls: Option<RustlsConfig>,
}

pub async fn run_data_plane_until<F>(
    app: CompiledWebServerApp,
    shutdown: F,
) -> Result<(), DataPlaneError>
where
    F: Future<Output = ()> + Send,
{
    let runtime = DataPlaneRuntime::build(app)?;
    run_data_plane_runtime_until(runtime, shutdown).await
}

pub(crate) async fn run_data_plane_runtime_until<F>(
    runtime: Arc<DataPlaneRuntime>,
    shutdown: F,
) -> Result<(), DataPlaneError>
where
    F: Future<Output = ()> + Send,
{
    let initial = runtime.current();
    let mut prepared = Vec::with_capacity(initial.app.config().listeners.len());
    for listener in initial.app.listeners() {
        prepared.push(prepare_listener(&initial, listener).await?);
    }

    let drain_timeout = Duration::from_millis(
        initial
            .app
            .config()
            .deployment
            .drain_timeout_ms
            .unwrap_or(initial.app.config().limits.drain_timeout_ms),
    );
    let mut shutdown_senders = Vec::with_capacity(prepared.len());
    let mut tasks = JoinSet::new();
    for listener in prepared {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        shutdown_senders.push(shutdown_tx);
        let runtime = runtime.clone();
        let listener_id = listener.config.id.clone();
        tracing::info!(
            listener_id = %listener_id,
            address = %listener.address,
            tls = listener.tls.is_some(),
            "data-plane listener prepared"
        );
        tasks.spawn(async move {
            let result = serve_listener(runtime, listener, shutdown_rx, drain_timeout).await;
            (listener_id, result)
        });
    }

    tokio::pin!(shutdown);
    tokio::select! {
        () = &mut shutdown => {
            request_listener_shutdown(&shutdown_senders);
            collect_shutdown_results(&mut tasks).await
        }
        result = tasks.join_next() => {
            request_listener_shutdown(&shutdown_senders);
            match result {
                Some(Ok((listener_id, Ok(())))) => {
                    let _ = collect_shutdown_results(&mut tasks).await;
                    Err(DataPlaneError::ListenerStopped { listener_id })
                }
                Some(Ok((_listener_id, Err(error)))) => {
                    let _ = collect_shutdown_results(&mut tasks).await;
                    Err(error)
                }
                Some(Err(error)) => {
                    let _ = collect_shutdown_results(&mut tasks).await;
                    Err(DataPlaneError::ListenerTask(error))
                }
                None => Ok(()),
            }
        }
    }
}

async fn prepare_listener(
    generation: &Arc<RuntimeGeneration>,
    listener: &ListenerConfig,
) -> Result<PreparedListener, DataPlaneError> {
    let ip = listener
        .bind
        .parse()
        .map_err(|source| DataPlaneError::InvalidBind {
            bind: listener.bind.clone(),
            source,
        })?;
    let requested_address = SocketAddr::new(ip, listener.port);
    let socket = TcpListener::bind(requested_address)
        .await
        .map_err(|source| DataPlaneError::Listener {
            listener_id: listener.id.clone(),
            source,
        })?;
    let address = socket
        .local_addr()
        .map_err(|source| DataPlaneError::Listener {
            listener_id: listener.id.clone(),
            source,
        })?;
    let tls = build_tls_config(generation, listener)?;
    Ok(PreparedListener {
        config: listener.clone(),
        socket,
        address,
        tls,
    })
}

async fn serve_listener(
    runtime: Arc<DataPlaneRuntime>,
    listener: PreparedListener,
    shutdown: watch::Receiver<bool>,
    drain_timeout: Duration,
) -> Result<(), DataPlaneError> {
    let listener_id = listener.config.id.clone();
    let initial = runtime.current();
    let maximum_connections = listener
        .config
        .max_connections
        .unwrap_or(initial.app.config().limits.max_connections);
    let state = ListenerState {
        runtime: runtime.clone(),
        listener_id: listener_id.clone(),
        is_tls: listener.tls.is_some(),
    };
    let app = Router::new()
        .fallback(route_request)
        .with_state(state)
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_millis(initial.app.config().limits.request_timeout_ms),
        ));
    let service = app.into_make_service_with_connect_info::<SocketAddr>();
    let global_permits = runtime.connection_permits.clone();
    let http1_only = listener.config.protocols == [ListenerProtocol::Http1];
    let http2_only = listener.config.protocols == [ListenerProtocol::Http2];
    let connection_write_timeout =
        Duration::from_millis(initial.app.config().limits.connection_write_timeout_ms);
    let http1_keep_alive_idle_timeout =
        Duration::from_millis(initial.app.config().limits.http1_keep_alive_idle_timeout_ms);
    let maximum_connection_age =
        Duration::from_millis(initial.app.config().limits.max_connection_age_ms);
    let mut builder = Builder::new(TokioExecutor::new());
    configure_http_protocols(&mut builder, &initial.app.config().limits);
    let allow_upgrades = !http1_only && !http2_only;
    let builder = if http1_only {
        builder.http1_only()
    } else if http2_only {
        builder.http2_only()
    } else {
        builder
    };
    let builder = Arc::new(builder);

    let result = if let Some(tls) = listener.tls {
        let limiter = ConnectionLimiter::new(global_permits, maximum_connections);
        let http1 = Http1WireGuardAcceptor::new(
            RustlsAcceptor::new(tls).acceptor(DefaultAcceptor::new()),
            &initial.app.config().limits,
        );
        let http2 = Http2WireGuardAcceptor::new(http1, &initial.app.config().limits);
        let keep_alive = Http1KeepAliveTimeoutAcceptor::new(http2, http1_keep_alive_idle_timeout);
        let acceptor = WriteTimeoutAcceptor::new(keep_alive, connection_write_timeout);
        serve_connections(
            listener.socket,
            limiter,
            acceptor,
            service,
            builder,
            allow_upgrades,
            shutdown,
            maximum_connection_age,
            drain_timeout,
        )
        .await
    } else {
        let limiter = ConnectionLimiter::new(global_permits, maximum_connections);
        let acceptor = WriteTimeoutAcceptor::new(
            Http1KeepAliveTimeoutAcceptor::new(
                Http2WireGuardAcceptor::new(
                    Http1WireGuardAcceptor::new(
                        DefaultAcceptor::new(),
                        &initial.app.config().limits,
                    ),
                    &initial.app.config().limits,
                ),
                http1_keep_alive_idle_timeout,
            ),
            connection_write_timeout,
        );
        serve_connections(
            listener.socket,
            limiter,
            acceptor,
            service,
            builder,
            allow_upgrades,
            shutdown,
            maximum_connection_age,
            drain_timeout,
        )
        .await
    };
    result.map_err(|source| DataPlaneError::Listener {
        listener_id,
        source,
    })
}

#[allow(clippy::too_many_arguments)]
async fn serve_connections<A, M>(
    listener: TcpListener,
    limiter: ConnectionLimiter,
    acceptor: A,
    mut make_service: M,
    builder: Arc<Builder<TokioExecutor>>,
    allow_upgrades: bool,
    mut shutdown: watch::Receiver<bool>,
    maximum_connection_age: Duration,
    drain_timeout: Duration,
) -> io::Result<()>
where
    M: MakeService<SocketAddr, Request<Incoming>> + Send,
    M::MakeFuture: Send,
    A: Accept<ConnectionLimitedStream<TcpStream>, M::Service> + Clone + Send + Sync + 'static,
    A::Stream: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    A::Service: SendService<Request<Incoming>> + Send + 'static,
    A::Future: Send,
{
    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            biased;
            () = wait_for_shutdown(&mut shutdown) => break,
            Some(result) = connections.join_next(), if !connections.is_empty() => {
                if let Err(error) = result {
                    tracing::warn!(%error, "connection task failed");
                }
            }
            accepted = listener.accept() => {
                let (stream, peer) = accepted?;
                let stream = match limiter.try_admit(stream) {
                    Ok(stream) => stream,
                    Err(_) => continue,
                };
                std::future::poll_fn(|context| make_service.poll_ready(context))
                    .await
                    .map_err(|error| {
                        let error: Box<dyn std::error::Error + Send + Sync> = error.into();
                        io::Error::other(error)
                    })?;
                let service = match make_service.make_service(peer).await {
                    Ok(service) => service,
                    Err(_) => continue,
                };
                let acceptor = acceptor.clone();
                let builder = builder.clone();
                let mut connection_shutdown = shutdown.clone();
                connections.spawn(async move {
                    let Ok((stream, service)) = acceptor.accept(stream, service).await else {
                        return;
                    };
                    let io = TokioIo::new(stream);
                    let service = TowerToHyperService::new(service.into_service());
                    let age = tokio::time::sleep(maximum_connection_age);
                    tokio::pin!(age);
                    if allow_upgrades {
                        let connection = builder.serve_connection_with_upgrades(io, service);
                        tokio::pin!(connection);
                        tokio::select! {
                            biased;
                            _ = &mut connection => return,
                            () = wait_for_shutdown(&mut connection_shutdown) => {}
                            () = &mut age => {}
                        }
                        connection.as_mut().graceful_shutdown();
                        let _ = timeout(drain_timeout, &mut connection).await;
                    } else {
                        let connection = builder.serve_connection(io, service);
                        tokio::pin!(connection);
                        tokio::select! {
                            biased;
                            _ = &mut connection => return,
                            () = wait_for_shutdown(&mut connection_shutdown) => {}
                            () = &mut age => {}
                        }
                        connection.as_mut().graceful_shutdown();
                        let _ = timeout(drain_timeout, &mut connection).await;
                    }
                });
            }
        }
    }

    drain_connection_tasks(&mut connections, drain_timeout).await;
    Ok(())
}

async fn wait_for_shutdown(shutdown: &mut watch::Receiver<bool>) {
    if *shutdown.borrow() {
        return;
    }
    while shutdown.changed().await.is_ok() {
        if *shutdown.borrow() {
            return;
        }
    }
}

async fn drain_connection_tasks(connections: &mut JoinSet<()>, drain_timeout: Duration) {
    if timeout(drain_timeout, async {
        while let Some(result) = connections.join_next().await {
            if let Err(error) = result {
                tracing::warn!(%error, "connection task failed during drain");
            }
        }
    })
    .await
    .is_err()
    {
        connections.abort_all();
        while connections.join_next().await.is_some() {}
    }
}

fn request_listener_shutdown(shutdown_senders: &[watch::Sender<bool>]) {
    for sender in shutdown_senders {
        let _ = sender.send(true);
    }
}

fn configure_http_protocols(builder: &mut Builder<TokioExecutor>, limits: &WebServerLimits) {
    let mut http1 = builder.http1();
    http1
        .half_close(true)
        .ignore_invalid_headers(false)
        .max_buf_size(limits.max_request_header_bytes)
        .header_read_timeout(Duration::from_millis(limits.request_header_timeout_ms))
        .timer(TokioTimer::new());
    if limits.max_request_headers != 100 {
        http1.max_headers(limits.max_request_headers);
    }

    let mut http2 = builder.http2();
    http2
        .adaptive_window(false)
        .initial_stream_window_size(65_535_u32)
        .initial_connection_window_size(65_535_u32)
        .max_frame_size(limits.http2_max_frame_bytes)
        .max_concurrent_streams(limits.http2_max_concurrent_streams)
        .max_pending_accept_reset_streams(limits.http2_max_pending_accept_reset_streams)
        .max_local_error_reset_streams(limits.http2_max_local_error_reset_streams)
        .max_send_buf_size(limits.http2_max_send_buffer_bytes)
        .max_header_list_size(limits.http2_max_header_list_bytes)
        .keep_alive_interval(Duration::from_millis(limits.http2_keep_alive_interval_ms))
        .keep_alive_timeout(Duration::from_millis(limits.http2_keep_alive_timeout_ms))
        .timer(TokioTimer::new());
}

async fn collect_shutdown_results(
    tasks: &mut JoinSet<(String, Result<(), DataPlaneError>)>,
) -> Result<(), DataPlaneError> {
    let mut first_error = None;
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok((_listener_id, Ok(()))) => {}
            Ok((_listener_id, Err(error))) if first_error.is_none() => first_error = Some(error),
            Err(error) if first_error.is_none() => {
                first_error = Some(DataPlaneError::ListenerTask(error))
            }
            _ => {}
        }
    }
    first_error.map_or(Ok(()), Err)
}
