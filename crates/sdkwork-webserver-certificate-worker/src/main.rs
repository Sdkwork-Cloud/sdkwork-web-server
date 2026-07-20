//! Background certificate renewal worker: scans autoRenew certificates and re-issues before expiry.

use std::time::Duration;

use sdkwork_intelligence_webserver_repository_sqlx::bootstrap_web_runtime_from_env;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    sdkwork_database_sqlx::enable_process_shared_database_pool();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let interval_secs = std::env::var("SDKWORK_WEB_CERT_RENEW_SCAN_INTERVAL_SECS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3600);

    info!(
        interval_secs,
        "sdkwork-webserver-certificate-worker started"
    );

    let runtime = bootstrap_web_runtime_from_env()
        .await
        .map_err(|error| anyhow::anyhow!(error))?;

    loop {
        match runtime.service.run_certificate_renewal_cycle().await {
            Ok(report) => {
                if report.scanned > 0 {
                    info!(
                        scanned = report.scanned,
                        renewed = report.renewed,
                        failed = report.failed,
                        "certificate renewal cycle completed"
                    );
                }
            }
            Err(error) => {
                warn!(error = %error, "certificate renewal cycle failed");
            }
        }
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}
