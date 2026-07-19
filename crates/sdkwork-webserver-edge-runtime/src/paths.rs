use std::fs::OpenOptions;
use std::io::Cursor;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use rustls::pki_types::PrivateKeyDer;
use rustls::sign::CertifiedKey;
use rustls_pemfile::Item;
use tempfile::{Builder, TempDir};

use crate::config::EdgeRuntimeConfig;
use crate::{EdgeRuntimeError, EdgeRuntimeResult};

const MAX_CERTIFICATE_PEM_BYTES: usize = 1024 * 1024;
const MAX_PRIVATE_KEY_PEM_BYTES: usize = 128 * 1024;
const MAX_PROCESS_CERTIFICATE_ACTIVATIONS: usize = 8;
static ACTIVE_CERTIFICATE_ACTIVATIONS: AtomicUsize = AtomicUsize::new(0);

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
    write_certificate_bundle_with_activator(cert_live_root, material, |staged, target| {
        std::fs::rename(staged, target)
    })
}

fn write_certificate_bundle_with_activator<F>(
    cert_live_root: &Path,
    material: &sdkwork_webserver_acme_service::IssuedCertificateMaterial,
    activate: F,
) -> EdgeRuntimeResult<()>
where
    F: FnOnce(&Path, &Path) -> std::io::Result<()>,
{
    validate_certificate_name(&material.cert_name)?;
    validate_certificate_material(&material.cert_pem, &material.private_key_pem)?;
    let _guard = CertificateActivationGuard::try_acquire()?;

    std::fs::create_dir_all(cert_live_root).map_err(|error| {
        EdgeRuntimeError::Filesystem(format!(
            "create certificate live root {}: {error}",
            cert_live_root.display()
        ))
    })?;

    let target = cert_live_root.join(&material.cert_name);
    if let Ok(metadata) = std::fs::symlink_metadata(&target) {
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(EdgeRuntimeError::Filesystem(format!(
                "certificate target {} must be a real directory",
                target.display()
            )));
        }
    }

    let staged = Builder::new()
        .prefix(".cert-stage-")
        .tempdir_in(cert_live_root)
        .map_err(|error| {
            EdgeRuntimeError::Filesystem(format!("stage certificate bundle: {error}"))
        })?;
    write_staged_bundle(&staged, material)?;
    sync_directory(staged.path())?;
    activate_staged_generation(cert_live_root, staged, &target, activate)?;
    sync_directory(cert_live_root)?;
    Ok(())
}

struct CertificateActivationGuard;

impl CertificateActivationGuard {
    fn try_acquire() -> EdgeRuntimeResult<Self> {
        ACTIVE_CERTIFICATE_ACTIVATIONS
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < MAX_PROCESS_CERTIFICATE_ACTIVATIONS).then_some(active + 1)
            })
            .map_err(|_| {
                EdgeRuntimeError::Filesystem(format!(
                    "process certificate activation capacity exhausted; maximum concurrent operations: {MAX_PROCESS_CERTIFICATE_ACTIVATIONS}"
                ))
            })?;
        Ok(Self)
    }
}

impl Drop for CertificateActivationGuard {
    fn drop(&mut self) {
        ACTIVE_CERTIFICATE_ACTIVATIONS.fetch_sub(1, Ordering::AcqRel);
    }
}

fn write_staged_bundle(
    staged: &TempDir,
    material: &sdkwork_webserver_acme_service::IssuedCertificateMaterial,
) -> EdgeRuntimeResult<()> {
    let fullchain = staged.path().join("fullchain.pem");
    let privkey = staged.path().join("privkey.pem");
    write_new_file(&fullchain, material.cert_pem.as_bytes())?;
    write_new_file(&privkey, material.private_key_pem.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&privkey)
            .map_err(|error| {
                EdgeRuntimeError::Filesystem(format!(
                    "read {} permissions: {error}",
                    privkey.display()
                ))
            })?
            .permissions();
        permissions.set_mode(0o600);
        std::fs::set_permissions(&privkey, permissions).map_err(|error| {
            EdgeRuntimeError::Filesystem(format!(
                "secure {} permissions: {error}",
                privkey.display()
            ))
        })?;
    }
    Ok(())
}

fn write_new_file(path: &Path, content: &[u8]) -> EdgeRuntimeResult<()> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| {
            EdgeRuntimeError::Filesystem(format!("create {}: {error}", path.display()))
        })?;
    file.write_all(content)
        .and_then(|_| file.flush())
        .and_then(|_| file.sync_all())
        .map_err(|error| EdgeRuntimeError::Filesystem(format!("write {}: {error}", path.display())))
}

fn activate_staged_generation<F>(
    cert_live_root: &Path,
    staged: TempDir,
    target: &Path,
    activate: F,
) -> EdgeRuntimeResult<()>
where
    F: FnOnce(&Path, &Path) -> std::io::Result<()>,
{
    let backup = if target.exists() {
        let holder = Builder::new()
            .prefix(".cert-backup-")
            .tempdir_in(cert_live_root)
            .map_err(|error| {
                EdgeRuntimeError::Filesystem(format!("create certificate backup: {error}"))
            })?;
        let previous = holder.path().join("previous");
        std::fs::rename(target, &previous).map_err(|error| {
            EdgeRuntimeError::Filesystem(format!(
                "backup certificate bundle {}: {error}",
                target.display()
            ))
        })?;
        Some((holder, previous))
    } else {
        None
    };

    let staged_path = staged.keep();
    if let Err(activation_error) = activate(&staged_path, target) {
        let cleanup_error = std::fs::remove_dir_all(&staged_path)
            .err()
            .filter(|error| error.kind() != std::io::ErrorKind::NotFound);
        if let Some((holder, previous)) = backup {
            if let Err(restore_error) = std::fs::rename(&previous, target) {
                let retained_backup = holder.keep();
                return Err(EdgeRuntimeError::Filesystem(format!(
                    "activate certificate bundle: {activation_error}; restore failed: {restore_error}; previous bundle retained at {}",
                    retained_backup.display()
                )));
            }
        }
        return Err(EdgeRuntimeError::Filesystem(match cleanup_error {
            Some(error) => format!(
                "activate certificate bundle: {activation_error}; staged cleanup failed: {error}"
            ),
            None => format!("activate certificate bundle: {activation_error}"),
        }));
    }

    drop(backup);
    Ok(())
}

fn validate_certificate_name(cert_name: &str) -> EdgeRuntimeResult<()> {
    if cert_name.is_empty()
        || cert_name.len() > 253
        || matches!(cert_name, "." | "..")
        || cert_name.starts_with('.')
        || cert_name.ends_with('.')
        || cert_name.contains("..")
        || !cert_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(EdgeRuntimeError::Config(
            "certificate name must contain 1..253 safe ASCII name bytes".to_string(),
        ));
    }
    Ok(())
}

fn validate_certificate_material(cert_pem: &str, private_key_pem: &str) -> EdgeRuntimeResult<()> {
    if cert_pem.is_empty()
        || cert_pem.len() > MAX_CERTIFICATE_PEM_BYTES
        || !cert_pem.contains("-----BEGIN CERTIFICATE-----")
        || !cert_pem.contains("-----END CERTIFICATE-----")
    {
        return Err(EdgeRuntimeError::Config(format!(
            "certificate PEM must be valid-looking and at most {MAX_CERTIFICATE_PEM_BYTES} bytes"
        )));
    }
    if private_key_pem.is_empty() || private_key_pem.len() > MAX_PRIVATE_KEY_PEM_BYTES {
        return Err(EdgeRuntimeError::Config(format!(
            "private-key PEM must be non-empty and at most {MAX_PRIVATE_KEY_PEM_BYTES} bytes"
        )));
    }

    let mut certificates = Vec::new();
    for item in rustls_pemfile::read_all(&mut Cursor::new(cert_pem.as_bytes())) {
        match item
            .map_err(|error| EdgeRuntimeError::Config(format!("parse certificate PEM: {error}")))?
        {
            Item::X509Certificate(certificate) => certificates.push(certificate),
            _ => {
                return Err(EdgeRuntimeError::Config(
                    "certificate PEM contains a non-certificate item".to_string(),
                ));
            }
        }
    }
    if certificates.is_empty() {
        return Err(EdgeRuntimeError::Config(
            "certificate PEM contains no certificate".to_string(),
        ));
    }

    let mut private_keys = rustls_pemfile::read_all(&mut Cursor::new(private_key_pem.as_bytes()))
        .map(|item| {
            item.map_err(|error| {
                EdgeRuntimeError::Config(format!("parse private-key PEM: {error}"))
            })
            .and_then(|item| match item {
                Item::Pkcs1Key(key) => Ok(Some(PrivateKeyDer::Pkcs1(key))),
                Item::Pkcs8Key(key) => Ok(Some(PrivateKeyDer::Pkcs8(key))),
                Item::Sec1Key(key) => Ok(Some(PrivateKeyDer::Sec1(key))),
                _ => Err(EdgeRuntimeError::Config(
                    "private-key PEM contains a non-key item".to_string(),
                )),
            })
        })
        .collect::<EdgeRuntimeResult<Vec<_>>>()?
        .into_iter()
        .flatten();
    let private_key = private_keys.next().ok_or_else(|| {
        EdgeRuntimeError::Config("private-key PEM contains no private key".to_string())
    })?;
    if private_keys.next().is_some() {
        return Err(EdgeRuntimeError::Config(
            "private-key PEM contains more than one private key".to_string(),
        ));
    }

    let provider = rustls::crypto::aws_lc_rs::default_provider();
    CertifiedKey::from_der(certificates, private_key, &provider).map_err(|error| {
        EdgeRuntimeError::Config(format!(
            "certificate and private key are incompatible: {error}"
        ))
    })?;
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> EdgeRuntimeResult<()> {
    std::fs::File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            EdgeRuntimeError::Filesystem(format!("sync directory {}: {error}", path.display()))
        })
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> EdgeRuntimeResult<()> {
    Ok(())
}

pub(crate) fn validate_domain_file_name(domain: &str) -> EdgeRuntimeResult<()> {
    if domain.len() > 253
        || domain.is_empty()
        || domain.starts_with('.')
        || domain.ends_with('.')
        || domain.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
    {
        return Err(EdgeRuntimeError::Config(
            "domain must be a safe ASCII DNS name".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rcgen::{CertificateParams, DistinguishedName, KeyPair};
    use sdkwork_webserver_acme_service::IssuedCertificateMaterial;
    use tempfile::TempDir;

    use super::*;

    fn sample_material(cert_name: &str, generation: &str) -> IssuedCertificateMaterial {
        let mut params = CertificateParams::new(vec![format!("{generation}.localhost")])
            .expect("certificate params");
        params.distinguished_name = DistinguishedName::new();
        let key_pair = KeyPair::generate().expect("key pair");
        let certificate = params.self_signed(&key_pair).expect("certificate");
        IssuedCertificateMaterial {
            cert_name: cert_name.to_string(),
            cert_type: 3,
            issuer: "Self-Signed".to_string(),
            subject: "dev.localhost".to_string(),
            san_list: "dev.localhost".to_string(),
            fingerprint: "abc123".to_string(),
            cert_pem: certificate.pem(),
            private_key_pem: key_pair.serialize_pem(),
            chain_pem: None,
            not_before: "2024-01-01T00:00:00Z".to_string(),
            not_after: "2027-01-01T00:00:00Z".to_string(),
            cert_path: format!("/tmp/live/{cert_name}/fullchain.pem"),
            key_path: format!("/tmp/live/{cert_name}/privkey.pem"),
            chain_path: None,
        }
    }

    #[test]
    fn write_certificate_bundle_creates_and_replaces_complete_generation() {
        let temp = TempDir::new().expect("tempdir");
        write_certificate_bundle(
            temp.path(),
            &sample_material("dev-localhost", "generation-1"),
        )
        .expect("first bundle");
        let replacement = sample_material("dev-localhost", "generation-2");
        let expected_certificate = replacement.cert_pem.clone();
        let expected_key = replacement.private_key_pem.clone();
        write_certificate_bundle(temp.path(), &replacement).expect("replacement bundle");

        let dir = temp.path().join("dev-localhost");
        assert_eq!(
            std::fs::read_to_string(dir.join("fullchain.pem")).expect("read fullchain"),
            expected_certificate
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("privkey.pem")).expect("read privkey"),
            expected_key
        );
        assert_eq!(
            std::fs::read_dir(temp.path()).expect("read root").count(),
            1
        );
    }

    #[test]
    fn activation_failure_restores_previous_complete_generation() {
        let temp = TempDir::new().expect("tempdir");
        let old = sample_material("dev-localhost", "generation-1");
        let old_certificate = old.cert_pem.clone();
        let old_key = old.private_key_pem.clone();
        write_certificate_bundle(temp.path(), &old).expect("first bundle");
        let replacement = sample_material("dev-localhost", "generation-2");
        let result = write_certificate_bundle_with_activator(
            temp.path(),
            &replacement,
            |_staged, _target| Err(std::io::Error::other("injected activation failure")),
        );
        assert!(result.is_err());
        let dir = temp.path().join("dev-localhost");
        assert_eq!(
            std::fs::read_to_string(dir.join("fullchain.pem")).expect("read fullchain"),
            old_certificate
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("privkey.pem")).expect("read privkey"),
            old_key
        );
        assert_eq!(
            std::fs::read_dir(temp.path()).expect("read root").count(),
            1
        );
    }

    #[test]
    fn certificate_bundle_rejects_unsafe_name_and_oversize_material() {
        let temp = TempDir::new().expect("tempdir");
        assert!(
            write_certificate_bundle(temp.path(), &sample_material("../escape", "one")).is_err()
        );
        let mut oversized = sample_material("safe-name", "one");
        oversized.cert_pem = format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
            "a".repeat(MAX_CERTIFICATE_PEM_BYTES)
        );
        assert!(write_certificate_bundle(temp.path(), &oversized).is_err());
        let first = sample_material("safe-name", "first");
        let second = sample_material("safe-name", "second");
        let mut mismatched = first;
        mismatched.private_key_pem = second.private_key_pem;
        assert!(write_certificate_bundle(temp.path(), &mismatched).is_err());
        assert_eq!(
            std::fs::read_dir(temp.path()).expect("read root").count(),
            0
        );
    }

    #[test]
    fn nginx_site_path_joins_domain_conf() {
        let config = EdgeRuntimeConfig {
            nginx_enabled: true,
            nginx_binary: "nginx".to_string(),
            nginx_main_config: PathBuf::from("/etc/nginx/nginx.conf"),
            nginx_sites_root: PathBuf::from("/etc/nginx/sites-enabled"),
            cert_live_root: PathBuf::from("/etc/letsencrypt/live"),
            site_family: "sdkwork".to_string(),
            nginx_command_timeout_ms: 10_000,
        };
        assert_eq!(
            nginx_site_path(&config, "example.com"),
            PathBuf::from("/etc/nginx/sites-enabled/example.com.conf")
        );
    }

    #[test]
    fn domain_file_name_rejects_path_and_dns_ambiguity() {
        assert!(validate_domain_file_name("api.sdkwork.com").is_ok());
        for invalid in [
            "",
            ".",
            "..",
            "../escape",
            "example.com/escape",
            "example.com\\escape",
            "example..com",
            "-example.com",
            "example-.com",
            "example.com:443",
        ] {
            assert!(
                validate_domain_file_name(invalid).is_err(),
                "{invalid} must be rejected"
            );
        }
    }
}
