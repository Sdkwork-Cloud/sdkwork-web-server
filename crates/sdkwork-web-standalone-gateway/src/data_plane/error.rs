use std::{io, net::AddrParseError, path::PathBuf};

use sdkwork_webserver_core::WebServerConfigError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataPlaneError {
    #[error(transparent)]
    Config(#[from] WebServerConfigError),

    #[error("invalid listener bind address {bind}: {source}")]
    InvalidBind {
        bind: String,
        source: AddrParseError,
    },

    #[error("cannot build upstream client {upstream_id}: {source}")]
    UpstreamClient {
        upstream_id: String,
        source: reqwest::Error,
    },

    #[error("cannot load {material} for upstream {upstream_id}: {source}")]
    UpstreamTls {
        upstream_id: String,
        material: &'static str,
        source: reqwest::Error,
    },

    #[error("upstream {upstream_id} CA bundle {path} contains no certificates")]
    EmptyUpstreamCaBundle { upstream_id: String, path: PathBuf },

    #[error("upstream {upstream_id} has {actual} custom root certificates; maximum is {maximum}")]
    TooManyUpstreamRootCertificates {
        upstream_id: String,
        actual: usize,
        maximum: usize,
    },

    #[error("upstream {upstream_id} has an invalid target URL {target}")]
    InvalidUpstreamTarget { upstream_id: String, target: String },

    #[error("listener {listener_id} references missing TLS policy {policy_id}")]
    MissingTlsPolicy {
        listener_id: String,
        policy_id: String,
    },

    #[error("TLS policy {policy_id} references missing certificate {certificate_id}")]
    MissingCertificate {
        policy_id: String,
        certificate_id: String,
    },

    #[error("certificate {certificate_id} has no resolved protected files")]
    MissingCertificateFiles { certificate_id: String },

    #[error("TLS policy {policy_id} has an ambiguous certificate mapping for {server_name}")]
    AmbiguousTlsServerName {
        policy_id: String,
        server_name: String,
    },

    #[error("cannot install a process-wide Rustls cryptography provider")]
    TlsCryptoProvider,

    #[error("cannot load TLS certificate {certificate_file} or key {private_key_file}: {source}")]
    TlsFiles {
        certificate_file: PathBuf,
        private_key_file: PathBuf,
        source: io::Error,
    },

    #[error("cannot fingerprint TLS material {path}: {source}")]
    TlsMaterialRead { path: PathBuf, source: io::Error },

    #[error("TLS material {path} is {actual_bytes} bytes; maximum is {maximum_bytes}")]
    TlsMaterialTooLarge {
        path: PathBuf,
        actual_bytes: u64,
        maximum_bytes: u64,
    },

    #[error("candidate configuration changes restart-only listener, TLS, or admission topology")]
    ReloadRequiresRestart,

    #[error("configuration reload worker failed: {0}")]
    ReloadWorker(#[source] tokio::task::JoinError),

    #[error("listener {listener_id} failed: {source}")]
    Listener {
        listener_id: String,
        source: io::Error,
    },

    #[error("listener {listener_id} stopped before shutdown")]
    ListenerStopped { listener_id: String },

    #[error("listener task failed: {0}")]
    ListenerTask(#[from] tokio::task::JoinError),
}
