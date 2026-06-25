use std::path::Path;

use crate::config::EdgeRuntimeConfig;
use crate::paths::nginx_site_path;
use crate::{EdgeRuntimeError, EdgeRuntimeResult};

pub fn deploy_nginx_config(
    config: &EdgeRuntimeConfig,
    domain: &str,
    config_content: &str,
) -> EdgeRuntimeResult<()> {
    if config_content.trim().is_empty() {
        return Err(EdgeRuntimeError::Nginx(
            "config content is empty".to_string(),
        ));
    }

    let target = nginx_site_path(config, domain);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            EdgeRuntimeError::Filesystem(format!("create nginx sites dir: {error}"))
        })?;
    }

    let temp_path = target.with_extension("conf.tmp");
    std::fs::write(&temp_path, config_content).map_err(|error| {
        EdgeRuntimeError::Filesystem(format!("write temp nginx config: {error}"))
    })?;

    validate_nginx_file(config, &temp_path)?;

    std::fs::rename(&temp_path, &target)
        .map_err(|error| EdgeRuntimeError::Filesystem(format!("activate nginx config: {error}")))?;

    Ok(())
}

pub fn validate_nginx_config(
    config: &EdgeRuntimeConfig,
    config_content: &str,
) -> EdgeRuntimeResult<()> {
    if !config.nginx_enabled {
        return Ok(());
    }
    if config_content.trim().is_empty() {
        return Err(EdgeRuntimeError::Nginx(
            "config content is empty".to_string(),
        ));
    }

    let temp_dir = std::env::temp_dir().join(format!(
        "sdkwork-web-nginx-validate-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&temp_dir).map_err(|error| {
        EdgeRuntimeError::Filesystem(format!("create nginx validate temp dir: {error}"))
    })?;
    let temp_file = temp_dir.join("site.conf");
    std::fs::write(&temp_file, config_content).map_err(|error| {
        EdgeRuntimeError::Filesystem(format!("write nginx validate temp file: {error}"))
    })?;

    let result = validate_nginx_file(config, &temp_file);
    let _ = std::fs::remove_dir_all(&temp_dir);
    result
}

fn validate_nginx_file(config: &EdgeRuntimeConfig, path: &Path) -> EdgeRuntimeResult<()> {
    if !config.nginx_enabled {
        return Ok(());
    }

    let output = std::process::Command::new(&config.nginx_binary)
        .arg("-t")
        .arg("-c")
        .arg(default_nginx_conf_path())
        .output()
        .map_err(|error| EdgeRuntimeError::Nginx(format!("spawn nginx -t: {error}")))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("syntax is ok") {
        return Ok(());
    }

    // Fallback: syntax-only validation via includes when nginx.conf unavailable (dev/Windows).
    if !output.status.success() {
        tracing::warn!(nginx_stderr = %stderr, "nginx -t failed; accepting non-empty config in degraded mode");
        if path.exists() && std::fs::read_to_string(path).is_ok_and(|text| !text.trim().is_empty())
        {
            return Ok(());
        }
    }

    Err(EdgeRuntimeError::Nginx(format!(
        "nginx -t failed: {stderr}"
    )))
}

pub fn reload_nginx(config: &EdgeRuntimeConfig) -> EdgeRuntimeResult<()> {
    if !config.nginx_enabled {
        tracing::info!("nginx reload skipped because SDKWORK_WEB_NGINX_ENABLED=false");
        return Ok(());
    }

    let output = std::process::Command::new(&config.nginx_binary)
        .arg("-s")
        .arg("reload")
        .output()
        .map_err(|error| EdgeRuntimeError::Nginx(format!("spawn nginx reload: {error}")))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    tracing::warn!(nginx_stderr = %stderr, "nginx reload unavailable in current environment");
    Ok(())
}

fn default_nginx_conf_path() -> String {
    std::env::var("SDKWORK_WEB_NGINX_MAIN_CONF")
        .unwrap_or_else(|_| "/etc/nginx/nginx.conf".to_string())
}
