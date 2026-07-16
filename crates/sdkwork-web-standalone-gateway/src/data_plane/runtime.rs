use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use arc_swap::ArcSwap;
use sdkwork_webserver_core::{
    CertificateConfig, CompiledWebServerApp, CompiledWebServerRevision, ListenerProtocol,
    ReloadConfig, TlsPolicyConfig,
};
use sha2::{Digest, Sha256};
use tokio::sync::{Mutex, Semaphore};

use super::{dns::BoundedSystemResolver, proxy::ProxyUpstream, DataPlaneError};

const MAX_TLS_MATERIAL_BYTES: u64 = 1024 * 1024;

pub(crate) struct RuntimeGeneration {
    pub id: u64,
    pub revision: String,
    pub app: Arc<CompiledWebServerApp>,
    pub upstreams: HashMap<String, ProxyUpstream>,
}

impl RuntimeGeneration {
    fn build(
        app: CompiledWebServerApp,
        revision: String,
        id: u64,
    ) -> Result<Arc<Self>, DataPlaneError> {
        let app = Arc::new(app);
        let implicit_resolver = BoundedSystemResolver::implicit();
        let resolvers = app
            .config()
            .resolvers
            .iter()
            .map(|resolver| {
                (
                    resolver.id.clone(),
                    BoundedSystemResolver::from_config(resolver),
                )
            })
            .collect::<HashMap<_, _>>();
        let upstreams = app
            .config()
            .upstreams
            .iter()
            .map(|upstream| {
                let resolver = upstream
                    .resolver_ref
                    .as_ref()
                    .map(|resolver_ref| {
                        resolvers
                            .get(resolver_ref)
                            .expect("semantic validation guarantees resolver references")
                    })
                    .unwrap_or(&implicit_resolver)
                    .clone();
                ProxyUpstream::build(upstream, resolver)
                    .map(|runtime| (upstream.id.clone(), runtime))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        Ok(Arc::new(Self {
            id,
            revision,
            app,
            upstreams,
        }))
    }
}

pub(crate) struct DataPlaneRuntime {
    current: ArcSwap<RuntimeGeneration>,
    topology: ReloadTopology,
    reload_lock: Mutex<()>,
    pub connection_permits: Arc<Semaphore>,
    pub request_permits: Arc<Semaphore>,
}

impl DataPlaneRuntime {
    pub fn build(app: CompiledWebServerApp) -> Result<Arc<Self>, DataPlaneError> {
        let revision = revision_for_compiled_app(&app);
        Self::build_inner(app, revision)
    }

    pub fn build_revision(
        revision: CompiledWebServerRevision,
    ) -> Result<Arc<Self>, DataPlaneError> {
        let sha256 = revision.sha256().to_owned();
        Self::build_inner(revision.into_app(), sha256)
    }

    fn build_inner(
        app: CompiledWebServerApp,
        revision: String,
    ) -> Result<Arc<Self>, DataPlaneError> {
        let topology = ReloadTopology::from_app(&app)?;
        let maximum_connections = app.config().limits.max_connections;
        let maximum_requests = app.config().limits.max_concurrent_requests;
        let initial = RuntimeGeneration::build(app, revision, 1)?;
        Ok(Arc::new(Self {
            current: ArcSwap::from(initial),
            topology,
            reload_lock: Mutex::new(()),
            connection_permits: Arc::new(Semaphore::new(maximum_connections)),
            request_permits: Arc::new(Semaphore::new(maximum_requests)),
        }))
    }

    pub fn current(&self) -> Arc<RuntimeGeneration> {
        self.current.load_full()
    }

    pub async fn reload(
        &self,
        revision: CompiledWebServerRevision,
    ) -> Result<DataPlaneReloadReport, DataPlaneError> {
        let _guard = self.reload_lock.lock().await;
        let candidate_topology = ReloadTopology::from_app(revision.app())?;
        if candidate_topology != self.topology {
            return Err(DataPlaneError::ReloadRequiresRestart);
        }

        let current = self.current();
        if current.revision == revision.sha256() {
            return Ok(DataPlaneReloadReport {
                generation: current.id,
                previous_revision: current.revision.clone(),
                revision: current.revision.clone(),
                changed: false,
            });
        }

        let generation = current.id.saturating_add(1);
        let next_revision = revision.sha256().to_owned();
        let candidate =
            RuntimeGeneration::build(revision.into_app(), next_revision.clone(), generation)?;
        self.current.store(candidate);
        Ok(DataPlaneReloadReport {
            generation,
            previous_revision: current.revision.clone(),
            revision: next_revision,
            changed: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPlaneReloadReport {
    pub generation: u64,
    pub previous_revision: String,
    pub revision: String,
    pub changed: bool,
}

#[derive(PartialEq, Eq)]
struct ReloadTopology {
    app_key: String,
    maximum_connections: usize,
    maximum_requests: usize,
    request_timeout_ms: u64,
    request_body_start_timeout_ms: u64,
    request_body_idle_timeout_ms: u64,
    response_body_idle_timeout_ms: u64,
    connection_write_timeout_ms: u64,
    http1_keep_alive_idle_timeout_ms: u64,
    max_connection_age_ms: u64,
    drain_timeout_ms: u64,
    reload: ReloadConfig,
    protocol_limits: HttpProtocolTopology,
    listeners: Vec<ListenerTopology>,
}

impl ReloadTopology {
    fn from_app(app: &CompiledWebServerApp) -> Result<Self, DataPlaneError> {
        let mut listeners = app
            .listeners()
            .map(|listener| {
                let tls = listener
                    .tls_policy_ref
                    .as_deref()
                    .map(|policy_id| tls_topology(app, policy_id))
                    .transpose()?;
                Ok(ListenerTopology {
                    id: listener.id.clone(),
                    bind: listener.bind.clone(),
                    port: listener.port,
                    protocols: listener.protocols.clone(),
                    maximum_connections: listener
                        .max_connections
                        .unwrap_or(app.config().limits.max_connections),
                    tls,
                })
            })
            .collect::<Result<Vec<_>, DataPlaneError>>()?;
        listeners.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(Self {
            app_key: app.config().app_key.clone(),
            maximum_connections: app.config().limits.max_connections,
            maximum_requests: app.config().limits.max_concurrent_requests,
            request_timeout_ms: app.config().limits.request_timeout_ms,
            request_body_start_timeout_ms: app.config().limits.request_body_start_timeout_ms,
            request_body_idle_timeout_ms: app.config().limits.request_body_idle_timeout_ms,
            response_body_idle_timeout_ms: app.config().limits.response_body_idle_timeout_ms,
            connection_write_timeout_ms: app.config().limits.connection_write_timeout_ms,
            http1_keep_alive_idle_timeout_ms: app.config().limits.http1_keep_alive_idle_timeout_ms,
            max_connection_age_ms: app.config().limits.max_connection_age_ms,
            drain_timeout_ms: app
                .config()
                .deployment
                .drain_timeout_ms
                .unwrap_or(app.config().limits.drain_timeout_ms),
            reload: app.config().deployment.reload.clone(),
            protocol_limits: HttpProtocolTopology::from_app(app),
            listeners,
        })
    }
}

#[derive(PartialEq, Eq)]
struct HttpProtocolTopology {
    http1_max_pipeline_depth: usize,
    max_request_header_bytes: usize,
    max_request_line_bytes: usize,
    max_request_method_bytes: usize,
    max_request_target_bytes: usize,
    max_header_name_bytes: usize,
    max_header_value_bytes: usize,
    max_request_headers: usize,
    request_header_timeout_ms: u64,
    max_chunk_line_bytes: usize,
    max_trailer_bytes: usize,
    max_trailers: usize,
    http2_max_concurrent_streams: u32,
    http2_keep_alive_interval_ms: u64,
    http2_keep_alive_timeout_ms: u64,
    http2_max_pending_accept_reset_streams: usize,
    http2_max_local_error_reset_streams: usize,
    http2_max_send_buffer_bytes: usize,
    http2_max_header_list_bytes: u32,
    http2_max_frame_bytes: u32,
    http2_abuse_window_ms: u64,
    http2_max_frames_per_window: usize,
    http2_max_new_streams_per_window: usize,
    http2_max_reset_frames_per_window: usize,
    http2_max_continuation_frames: usize,
    http2_max_encoded_header_block_bytes: usize,
}

impl HttpProtocolTopology {
    fn from_app(app: &CompiledWebServerApp) -> Self {
        let limits = &app.config().limits;
        Self {
            http1_max_pipeline_depth: limits.http1_max_pipeline_depth,
            max_request_header_bytes: limits.max_request_header_bytes,
            max_request_line_bytes: limits.max_request_line_bytes,
            max_request_method_bytes: limits.max_request_method_bytes,
            max_request_target_bytes: limits.max_request_target_bytes,
            max_header_name_bytes: limits.max_header_name_bytes,
            max_header_value_bytes: limits.max_header_value_bytes,
            max_request_headers: limits.max_request_headers,
            request_header_timeout_ms: limits.request_header_timeout_ms,
            max_chunk_line_bytes: limits.max_chunk_line_bytes,
            max_trailer_bytes: limits.max_trailer_bytes,
            max_trailers: limits.max_trailers,
            http2_max_concurrent_streams: limits.http2_max_concurrent_streams,
            http2_keep_alive_interval_ms: limits.http2_keep_alive_interval_ms,
            http2_keep_alive_timeout_ms: limits.http2_keep_alive_timeout_ms,
            http2_max_pending_accept_reset_streams: limits.http2_max_pending_accept_reset_streams,
            http2_max_local_error_reset_streams: limits.http2_max_local_error_reset_streams,
            http2_max_send_buffer_bytes: limits.http2_max_send_buffer_bytes,
            http2_max_header_list_bytes: limits.http2_max_header_list_bytes,
            http2_max_frame_bytes: limits.http2_max_frame_bytes,
            http2_abuse_window_ms: limits.http2_abuse_window_ms,
            http2_max_frames_per_window: limits.http2_max_frames_per_window,
            http2_max_new_streams_per_window: limits.http2_max_new_streams_per_window,
            http2_max_reset_frames_per_window: limits.http2_max_reset_frames_per_window,
            http2_max_continuation_frames: limits.http2_max_continuation_frames,
            http2_max_encoded_header_block_bytes: limits.http2_max_encoded_header_block_bytes,
        }
    }
}

#[derive(PartialEq, Eq)]
struct ListenerTopology {
    id: String,
    bind: String,
    port: u16,
    protocols: Vec<ListenerProtocol>,
    maximum_connections: usize,
    tls: Option<TlsTopology>,
}

#[derive(PartialEq, Eq)]
struct TlsTopology {
    policy: TlsPolicyConfig,
    certificates: Vec<TlsCertificateTopology>,
}

#[derive(PartialEq, Eq)]
struct TlsCertificateTopology {
    certificate: CertificateConfig,
    certificate_path: PathBuf,
    certificate_sha256: [u8; 32],
    private_key_path: PathBuf,
    private_key_sha256: [u8; 32],
}

fn tls_topology(
    app: &CompiledWebServerApp,
    policy_id: &str,
) -> Result<TlsTopology, DataPlaneError> {
    let policy = app
        .tls_policy(policy_id)
        .expect("semantic validation guarantees TLS policy references");
    let certificates = policy
        .certificate_refs()
        .map(|certificate_ref| tls_certificate_topology(app, certificate_ref))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TlsTopology {
        policy: policy.clone(),
        certificates,
    })
}

fn tls_certificate_topology(
    app: &CompiledWebServerApp,
    certificate_ref: &str,
) -> Result<TlsCertificateTopology, DataPlaneError> {
    let certificate = app
        .certificate(certificate_ref)
        .expect("semantic validation guarantees certificate references");
    let (certificate_path, private_key_path) = app
        .certificate_paths(&certificate.id)
        .expect("compilation resolves certificate paths");
    Ok(TlsCertificateTopology {
        certificate: certificate.clone(),
        certificate_path: certificate_path.to_path_buf(),
        certificate_sha256: bounded_file_sha256(certificate_path)?,
        private_key_path: private_key_path.to_path_buf(),
        private_key_sha256: bounded_file_sha256(private_key_path)?,
    })
}

pub(crate) fn read_bounded_tls_material(path: &Path) -> Result<Vec<u8>, DataPlaneError> {
    let mut file = File::open(path).map_err(|source| DataPlaneError::TlsMaterialRead {
        path: path.to_path_buf(),
        source,
    })?;
    let mut bytes = Vec::with_capacity(16 * 1024);
    file.by_ref()
        .take(MAX_TLS_MATERIAL_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| DataPlaneError::TlsMaterialRead {
            path: path.to_path_buf(),
            source,
        })?;
    if bytes.len() as u64 > MAX_TLS_MATERIAL_BYTES {
        return Err(DataPlaneError::TlsMaterialTooLarge {
            path: path.to_path_buf(),
            actual_bytes: bytes.len() as u64,
            maximum_bytes: MAX_TLS_MATERIAL_BYTES,
        });
    }
    Ok(bytes)
}

fn bounded_file_sha256(path: &Path) -> Result<[u8; 32], DataPlaneError> {
    Ok(Sha256::digest(read_bounded_tls_material(path)?).into())
}

fn revision_for_compiled_app(app: &CompiledWebServerApp) -> String {
    let bytes = serde_json::to_vec(app.config()).expect("Web Server config always serializes");
    format!("runtime:{}", hex::encode(Sha256::digest(bytes)))
}
