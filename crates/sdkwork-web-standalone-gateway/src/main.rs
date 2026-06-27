use sdkwork_web_standalone_gateway::{build_router, run_database_migrate_only};
use tokio::signal;

fn init_tracing() {
    let environment =
        std::env::var("SDKWORK_WEB_ENVIRONMENT").unwrap_or_else(|_| "development".to_owned());
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = environment;
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

#[tokio::main]
async fn main() {
    init_tracing();

    if matches!(std::env::args().nth(1).as_deref(), Some("db-migrate")) {
        run_database_migrate_only()
            .await
            .expect("Web database migration failed");
        return;
    }

    let bind_address = std::env::var("SDKWORK_WEB_APPLICATION_PUBLIC_INGRESS_BIND")
        .unwrap_or_else(|_| "127.0.0.1:3800".to_owned());
    let app = build_router()
        .await
        .expect("Web standalone-gateway bootstrap failed");
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .expect("bind Web standalone-gateway listener failed");
    tracing::info!("sdkwork-web-standalone-gateway listening on {bind_address}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("serve Web standalone-gateway failed");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("sdkwork-web-standalone-gateway shutdown signal received");
}
