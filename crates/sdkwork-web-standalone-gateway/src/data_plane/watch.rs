use std::{
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use sdkwork_webserver_core::{
    inspect_webserver_config_revision, load_and_compile_webserver_config_revision,
    CompiledWebServerRevision, ReloadMode, WebServerConfigError,
};
use tokio::{sync::watch, time::MissedTickBehavior};

use super::{runtime::DataPlaneRuntime, server::run_data_plane_runtime_until, DataPlaneError};

pub async fn run_data_plane_from_config_until<F>(
    config_path: impl Into<PathBuf>,
    shutdown: F,
) -> Result<(), DataPlaneError>
where
    F: Future<Output = ()> + Send,
{
    let config_path = config_path.into();
    let initial = load_and_compile_webserver_config_revision(&config_path)?;
    let reload = initial.app().config().deployment.reload.clone();
    let runtime = DataPlaneRuntime::build_revision(initial)?;
    if reload.mode == ReloadMode::Disabled {
        return run_data_plane_runtime_until(runtime, shutdown).await;
    }

    let (stop_tx, stop_rx) = watch::channel(false);
    let worker_runtime = runtime.clone();
    let worker_path = config_path.clone();
    let worker = tokio::spawn(async move {
        watch_config(
            worker_runtime,
            worker_path,
            Duration::from_millis(reload.poll_interval_ms),
            stop_rx,
        )
        .await;
    });

    let result = run_data_plane_runtime_until(runtime, shutdown).await;
    let _ = stop_tx.send(true);
    if let Err(error) = worker.await {
        if result.is_ok() {
            return Err(DataPlaneError::ReloadWorker(error));
        }
    }
    result
}

async fn watch_config(
    runtime: Arc<DataPlaneRuntime>,
    config_path: PathBuf,
    poll_interval: Duration,
    mut stop: watch::Receiver<bool>,
) {
    let mut ticker = tokio::time::interval(poll_interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last_observed_revision = Some(runtime.current().revision.clone());
    let mut last_error = None;

    loop {
        tokio::select! {
            changed = stop.changed() => {
                if changed.is_err() || *stop.borrow() {
                    return;
                }
            }
            _ = ticker.tick() => {
                let inspection_path = config_path.clone();
                let inspection = tokio::task::spawn_blocking(move || {
                    inspect_webserver_config_revision(inspection_path)
                })
                .await;
                let source_revision = match inspection {
                    Ok(Ok(revision)) => revision,
                    Ok(Err(error)) => {
                        log_reload_error_once(
                            &mut last_error,
                            format!("cannot inspect watched config: {error}"),
                            &config_path,
                        );
                        continue;
                    }
                    Err(error) => {
                        log_reload_error_once(
                            &mut last_error,
                            format!("config inspection task failed: {error}"),
                            &config_path,
                        );
                        continue;
                    }
                };
                if last_observed_revision.as_deref() == Some(source_revision.sha256()) {
                    continue;
                }
                last_observed_revision = Some(source_revision.sha256().to_owned());
                last_error = None;

                let candidate_path = config_path.clone();
                let loaded = tokio::task::spawn_blocking(move || {
                    load_and_compile_webserver_config_revision(candidate_path)
                })
                .await;
                let candidate = match loaded {
                    Ok(Ok(candidate)) => candidate,
                    Ok(Err(error)) => {
                        log_reload_error_once(
                            &mut last_error,
                            format!("candidate-{}", config_error_class(&error)),
                            &config_path,
                        );
                        continue;
                    }
                    Err(error) => {
                        log_reload_error_once(
                            &mut last_error,
                            format!("candidate-loader-task-{error}"),
                            &config_path,
                        );
                        continue;
                    }
                };

                match publish_candidate(&runtime, candidate).await {
                    Ok(report) => {
                        last_error = None;
                        if report.changed {
                            tracing::info!(
                                config_path = %config_path.display(),
                                config_generation = report.generation,
                                previous_revision = %report.previous_revision,
                                config_revision = %report.revision,
                                "data-plane configuration generation published"
                            );
                        }
                    }
                    Err(error) => {
                        log_reload_error_once(
                            &mut last_error,
                            format!("candidate-{}", publication_error_class(&error)),
                            &config_path,
                        );
                    }
                }
            }
        }
    }
}

async fn publish_candidate(
    runtime: &DataPlaneRuntime,
    candidate: CompiledWebServerRevision,
) -> Result<super::DataPlaneReloadReport, DataPlaneError> {
    runtime.reload(candidate).await
}

fn log_reload_error_once(last_error: &mut Option<String>, error: String, path: &Path) {
    if last_error.as_deref() == Some(error.as_str()) {
        return;
    }
    tracing::warn!(
        config_path = %path.display(),
        error = %error,
        "data-plane configuration reload retained the active generation"
    );
    *last_error = Some(error);
}

fn config_error_class(error: &WebServerConfigError) -> &'static str {
    match error {
        WebServerConfigError::Inspect { .. } => "inspect-failed",
        WebServerConfigError::TooLarge { .. } => "too-large",
        WebServerConfigError::Read { .. } => "read-failed",
        WebServerConfigError::Json { .. } => "invalid-json",
        WebServerConfigError::InvalidSchema(_) => "invalid-embedded-schema",
        WebServerConfigError::Validation { .. } => "validation-failed",
    }
}

fn publication_error_class(error: &DataPlaneError) -> &'static str {
    match error {
        DataPlaneError::ReloadRequiresRestart => "restart-required",
        DataPlaneError::UpstreamClient { .. } => "upstream-client-build-failed",
        DataPlaneError::InvalidUpstreamTarget { .. } => "invalid-upstream-target",
        DataPlaneError::TlsMaterialRead { .. } => "tls-material-read-failed",
        DataPlaneError::TlsMaterialTooLarge { .. } => "tls-material-too-large",
        _ => "runtime-generation-build-failed",
    }
}
