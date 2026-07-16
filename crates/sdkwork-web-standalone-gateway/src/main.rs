use std::{error::Error, io, path::PathBuf};

use sdkwork_web_standalone_gateway::{
    build_router, run_data_plane_from_config_until, run_database_migrate_only,
};
use sdkwork_webserver_core::load_and_compile_webserver_config_revision;
use tokio::signal;

type MainResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

fn init_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

#[tokio::main]
async fn main() {
    init_tracing();
    if let Err(error) = run().await {
        tracing::error!(error = %error, "sdkwork-web-standalone-gateway failed");
        std::process::exit(1);
    }
}

async fn run() -> MainResult<()> {
    let mut arguments = std::env::args().skip(1);
    match arguments.next().as_deref() {
        None | Some("serve-management") => run_management_plane().await?,
        Some("db-migrate") => run_database_migrate_only()
            .await
            .map_err(|error| io::Error::other(format!("database migration failed: {error}")))?,
        Some("validate") => validate_config(config_path(arguments.next())?)?,
        Some("data-plane") => {
            run_data_plane_from_config_until(config_path(arguments.next())?, shutdown_signal())
                .await?;
        }
        Some("help" | "--help" | "-h") => print_help(),
        Some(command) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown operation {command}; run with --help"),
            )
            .into())
        }
    }
    Ok(())
}

async fn run_management_plane() -> MainResult<()> {
    let bind_address = std::env::var("SDKWORK_WEB_APPLICATION_PUBLIC_INGRESS_BIND")
        .unwrap_or_else(|_| "127.0.0.1:3800".to_owned());
    let app = build_router()
        .await
        .map_err(|error| io::Error::other(format!("management bootstrap failed: {error}")))?;
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    tracing::info!(address = %bind_address, "management listener started");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn validate_config(path: PathBuf) -> MainResult<()> {
    let revision = load_and_compile_webserver_config_revision(&path)?;
    let compiled = revision.app();
    let route_count = compiled
        .config()
        .virtual_hosts
        .iter()
        .map(|virtual_host| virtual_host.routes.len())
        .sum::<usize>();
    println!(
        "validated appKey={} revision={} bytes={} listeners={} virtualHosts={} routes={} resources={} upstreams={} tlsPolicies={}",
        compiled.config().app_key,
        revision.sha256(),
        revision.size_bytes(),
        compiled.config().listeners.len(),
        compiled.config().virtual_hosts.len(),
        route_count,
        compiled.config().resources.len(),
        compiled.config().upstreams.len(),
        compiled.config().tls_policies.len(),
    );
    Ok(())
}

fn config_path(argument: Option<String>) -> MainResult<PathBuf> {
    argument
        .or_else(|| std::env::var("SDKWORK_WEB_SERVER_CONFIG_FILE").ok())
        .map(PathBuf::from)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "a config path argument or SDKWORK_WEB_SERVER_CONFIG_FILE is required",
            )
            .into()
        })
}

fn print_help() {
    println!(
        "sdkwork-web-standalone-gateway\n\
         \n\
         Operations:\n\
           serve-management       Start the existing management API (default).\n\
           db-migrate             Run database migration and exit.\n\
           validate <config>      Validate and compile Web Server app config.\n\
           data-plane <config>    Start HTTP/HTTPS application listeners without a database.\n"
    );
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = signal::ctrl_c().await {
            tracing::error!(error = %error, "failed to receive Ctrl+C signal");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::error!(error = %error, "failed to install SIGTERM handler");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
