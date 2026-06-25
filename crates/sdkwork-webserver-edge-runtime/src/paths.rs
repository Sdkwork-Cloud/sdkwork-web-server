use std::path::{Path, PathBuf};

use crate::config::EdgeRuntimeConfig;
use crate::{EdgeRuntimeError, EdgeRuntimeResult};

pub fn nginx_site_path(config: &EdgeRuntimeConfig, domain: &str) -> PathBuf {
    config.nginx_sites_root.join(format!("{domain}.conf"))
}

pub fn cert_bundle_paths(config: &EdgeRuntimeConfig, cert_name: &str) -> (PathBuf, PathBuf) {
    let dir = config.cert_live_root.join(cert_name);
    (dir.join("fullchain.pem"), dir.join("privkey.pem"))
}

pub fn write_certificate_bundle(
    cert_live_root: &Path,
    material: &sdkwork_webserver_acme_service::IssuedCertificateMaterial,
) -> EdgeRuntimeResult<()> {
    let dir = cert_live_root.join(&material.cert_name);
    std::fs::create_dir_all(&dir).map_err(|error| {
        EdgeRuntimeError::Filesystem(format!("create cert dir {}: {error}", dir.display()))
    })?;

    let fullchain = dir.join("fullchain.pem");
    let privkey = dir.join("privkey.pem");
    std::fs::write(&fullchain, &material.cert_pem).map_err(|error| {
        EdgeRuntimeError::Filesystem(format!("write {}: {error}", fullchain.display()))
    })?;
    std::fs::write(&privkey, &material.private_key_pem).map_err(|error| {
        EdgeRuntimeError::Filesystem(format!("write {}: {error}", privkey.display()))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&privkey) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o600);
            let _ = std::fs::set_permissions(&privkey, perms);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use sdkwork_webserver_acme_service::IssuedCertificateMaterial;
    use tempfile::TempDir;

    use super::*;

    fn sample_material(cert_name: &str) -> IssuedCertificateMaterial {
        IssuedCertificateMaterial {
            cert_name: cert_name.to_string(),
            cert_type: 3,
            issuer: "Self-Signed".to_string(),
            subject: "dev.localhost".to_string(),
            san_list: "dev.localhost".to_string(),
            fingerprint: "abc123".to_string(),
            cert_pem: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----".to_string(),
            private_key_pem: "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----"
                .to_string(),
            chain_pem: None,
            not_before: "Mon, 01 Jan 2024 00:00:00 GMT".to_string(),
            not_after: "Tue, 01 Jan 2027 00:00:00 GMT".to_string(),
            cert_path: format!("/tmp/live/{cert_name}/fullchain.pem"),
            key_path: format!("/tmp/live/{cert_name}/privkey.pem"),
            chain_path: None,
        }
    }

    #[test]
    fn write_certificate_bundle_creates_pem_files() {
        let temp = TempDir::new().expect("tempdir");
        let material = sample_material("dev-localhost");
        write_certificate_bundle(temp.path(), &material).expect("write bundle");

        let dir = temp.path().join("dev-localhost");
        let fullchain = dir.join("fullchain.pem");
        let privkey = dir.join("privkey.pem");
        assert!(fullchain.is_file());
        assert!(privkey.is_file());
        assert!(std::fs::read_to_string(fullchain)
            .expect("read fullchain")
            .contains("BEGIN CERTIFICATE"));
        assert!(std::fs::read_to_string(privkey)
            .expect("read privkey")
            .contains("BEGIN PRIVATE KEY"));
    }

    #[test]
    fn nginx_site_path_joins_domain_conf() {
        let config = EdgeRuntimeConfig {
            nginx_enabled: true,
            nginx_binary: "nginx".to_string(),
            nginx_sites_root: PathBuf::from("/etc/nginx/sites-enabled"),
            cert_live_root: PathBuf::from("/etc/letsencrypt/live"),
            site_family: "sdkwork".to_string(),
        };
        assert_eq!(
            nginx_site_path(&config, "example.com"),
            PathBuf::from("/etc/nginx/sites-enabled/example.com.conf")
        );
    }
}
