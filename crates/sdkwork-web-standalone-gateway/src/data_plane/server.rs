use std::{future::Future, net::SocketAddr, sync::Arc, time::Duration};

use axum::{http::StatusCode, Router};
use axum_server::{accept::DefaultAcceptor, tls_rustls::RustlsConfig, Handle};
use sdkwork_webserver_core::{CompiledWebServerApp, ListenerConfig, ListenerProtocol};
use tokio::task::JoinSet;
use tower_http::timeout::TimeoutLayer;

use super::{
    connection_limit::ConnectionLimitAcceptor, handler::route_request, DataPlaneError,
    DataPlaneRuntime, ListenerState,
};

struct PreparedListener {
    config: ListenerConfig,
    socket: std::net::TcpListener,
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
    let mut prepared = Vec::with_capacity(runtime.app.config().listeners.len());
    for listener in runtime.app.listeners() {
        prepared.push(prepare_listener(&runtime, listener).await?);
    }

    let drain_timeout = Duration::from_millis(
        runtime
            .app
            .config()
            .deployment
            .drain_timeout_ms
            .unwrap_or(runtime.app.config().limits.drain_timeout_ms),
    );
    let mut handles = Vec::with_capacity(prepared.len());
    let mut tasks = JoinSet::new();
    for listener in prepared {
        let handle = Handle::new();
        handles.push(handle.clone());
        let runtime = runtime.clone();
        let listener_id = listener.config.id.clone();
        tracing::info!(
            listener_id = %listener_id,
            address = %listener.address,
            tls = listener.tls.is_some(),
            "data-plane listener prepared"
        );
        tasks.spawn(async move {
            let result = serve_listener(runtime, listener, handle).await;
            (listener_id, result)
        });
    }

    tokio::pin!(shutdown);
    tokio::select! {
        () = &mut shutdown => {
            for handle in &handles {
                handle.graceful_shutdown(Some(drain_timeout));
            }
            collect_shutdown_results(&mut tasks).await
        }
        result = tasks.join_next() => {
            for handle in &handles {
                handle.graceful_shutdown(Some(drain_timeout));
            }
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
    runtime: &Arc<DataPlaneRuntime>,
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
    let socket = tokio::net::TcpListener::bind(requested_address)
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
    let socket = socket
        .into_std()
        .map_err(|source| DataPlaneError::Listener {
            listener_id: listener.id.clone(),
            source,
        })?;
    let tls = build_tls_config(runtime, listener).await?;
    Ok(PreparedListener {
        config: listener.clone(),
        socket,
        address,
        tls,
    })
}

async fn build_tls_config(
    runtime: &Arc<DataPlaneRuntime>,
    listener: &ListenerConfig,
) -> Result<Option<RustlsConfig>, DataPlaneError> {
    let Some(policy_id) = &listener.tls_policy_ref else {
        return Ok(None);
    };
    let policy =
        runtime
            .app
            .tls_policy(policy_id)
            .ok_or_else(|| DataPlaneError::MissingTlsPolicy {
                listener_id: listener.id.clone(),
                policy_id: policy_id.clone(),
            })?;
    let certificate = runtime
        .app
        .certificate(&policy.certificate_ref)
        .ok_or_else(|| DataPlaneError::MissingCertificate {
            policy_id: policy.id.clone(),
            certificate_id: policy.certificate_ref.clone(),
        })?;
    let (certificate_file, private_key_file) = runtime
        .app
        .certificate_paths(&certificate.id)
        .ok_or_else(|| DataPlaneError::MissingCertificateFiles {
            certificate_id: certificate.id.clone(),
        })?;
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    }
    let loaded = RustlsConfig::from_pem_file(certificate_file, private_key_file)
        .await
        .map_err(|source| DataPlaneError::TlsFiles {
            certificate_file: certificate_file.to_path_buf(),
            private_key_file: private_key_file.to_path_buf(),
            source,
        })?;
    let mut server_config = (*loaded.get_inner()).clone();
    server_config.alpn_protocols = policy
        .alpn
        .iter()
        .map(|protocol| protocol.as_bytes().to_vec())
        .collect();
    Ok(Some(RustlsConfig::from_config(Arc::new(server_config))))
}

async fn serve_listener(
    runtime: Arc<DataPlaneRuntime>,
    listener: PreparedListener,
    handle: Handle<SocketAddr>,
) -> Result<(), DataPlaneError> {
    let listener_id = listener.config.id.clone();
    let maximum_connections = listener
        .config
        .max_connections
        .unwrap_or(runtime.app.config().limits.max_connections);
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
            Duration::from_millis(runtime.app.config().limits.request_timeout_ms),
        ));
    let service = app.into_make_service_with_connect_info::<SocketAddr>();
    let global_permits = runtime.connection_permits.clone();
    let http1_only = listener.config.protocols == [ListenerProtocol::Http1];
    let http2_only = listener.config.protocols == [ListenerProtocol::Http2];

    let result = if let Some(tls) = listener.tls {
        let limiter = ConnectionLimitAcceptor::new(
            DefaultAcceptor::new(),
            global_permits,
            maximum_connections,
        );
        let mut server = axum_server::from_tcp_rustls(listener.socket, tls)
            .map_err(|source| DataPlaneError::Listener {
                listener_id: listener_id.clone(),
                source,
            })?
            .map(|acceptor| acceptor.acceptor(limiter));
        if http1_only {
            server = server.http1_only();
        } else if http2_only {
            server = server.http2_only();
        }
        server.handle(handle).serve(service).await
    } else {
        let limiter = ConnectionLimitAcceptor::new(
            DefaultAcceptor::new(),
            global_permits,
            maximum_connections,
        );
        let mut server = axum_server::from_tcp(listener.socket)
            .map_err(|source| DataPlaneError::Listener {
                listener_id: listener_id.clone(),
                source,
            })?
            .acceptor(limiter);
        if http1_only {
            server = server.http1_only();
        } else if http2_only {
            server = server.http2_only();
        }
        server.handle(handle).serve(service).await
    };
    result.map_err(|source| DataPlaneError::Listener {
        listener_id,
        source,
    })
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
