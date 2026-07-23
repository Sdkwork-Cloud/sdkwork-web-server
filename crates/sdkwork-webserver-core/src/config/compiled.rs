use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use super::{
    validate::normalize_server_name, CertificateConfig, CertificateSource, ConfigDiagnostic,
    ListenerConfig, ResourceConfig, RouteConfig, RoutePathType, TlsPolicyConfig, UpstreamConfig,
    VirtualHostConfig, WebServerAppConfig, WebServerConfigError,
};

#[derive(Debug)]
pub struct CompiledWebServerApp {
    config: WebServerAppConfig,
    base_directory: PathBuf,
    listeners: HashMap<String, usize>,
    resources: HashMap<String, usize>,
    upstreams: HashMap<String, usize>,
    certificates: HashMap<String, usize>,
    tls_policies: HashMap<String, usize>,
    compiled_listeners: HashMap<String, CompiledListener>,
    compiled_hosts: Vec<CompiledVirtualHost>,
    static_roots: HashMap<String, PathBuf>,
    certificate_paths: HashMap<String, (PathBuf, PathBuf)>,
    upstream_tls_paths: HashMap<String, CompiledUpstreamTlsPaths>,
}

#[derive(Debug)]
struct CompiledUpstreamTlsPaths {
    ca_certificates: Vec<PathBuf>,
    client_identity: Option<(PathBuf, PathBuf)>,
}

#[derive(Debug)]
struct CompiledListener {
    exact_hosts: HashMap<String, usize>,
    wildcard_hosts: Vec<(String, usize)>,
    default_host: Option<usize>,
}

#[derive(Debug)]
struct CompiledVirtualHost {
    config_index: usize,
    exact_routes: HashMap<String, Vec<usize>>,
    prefix_routes: PrefixMatcher,
}

#[derive(Debug, Default)]
struct PrefixMatcher {
    nodes: Vec<PrefixNode>,
}

#[derive(Debug, Default)]
struct PrefixNode {
    children: HashMap<u8, usize>,
    routes: Vec<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct SelectedRoute<'a> {
    pub virtual_host: &'a VirtualHostConfig,
    pub route: &'a RouteConfig,
    pub resource: &'a ResourceConfig,
}

impl CompiledWebServerApp {
    pub(crate) fn compile(
        config: WebServerAppConfig,
        base_directory: &Path,
    ) -> Result<Self, WebServerConfigError> {
        let base_directory = canonical_directory(base_directory, "/")?;
        let listeners = index_by_id(config.listeners.iter().map(|item| item.id.as_str()));
        let resources = index_by_id(config.resources.iter().map(ResourceConfig::id));
        let upstreams = index_by_id(config.upstreams.iter().map(|item| item.id.as_str()));
        let certificates = index_by_id(config.certificates.iter().map(|item| item.id.as_str()));
        let tls_policies = index_by_id(config.tls_policies.iter().map(|item| item.id.as_str()));

        let mut static_roots = HashMap::new();
        for (index, resource) in config.resources.iter().enumerate() {
            if let ResourceConfig::Static { id, root, .. } = resource {
                let resolved = canonical_directory(
                    &base_directory.join(root),
                    &format!("/resources/{index}/root"),
                )?;
                if !resolved.starts_with(&base_directory) {
                    return Err(validation_error(
                        format!("/resources/{index}/root"),
                        "static root escapes the configuration directory",
                    ));
                }
                static_roots.insert(id.clone(), resolved);
            }
        }

        let mut certificate_paths = HashMap::new();
        for (index, certificate) in config.certificates.iter().enumerate() {
            let CertificateSource::ProtectedFile {
                certificate_file,
                private_key_file,
            } = &certificate.source;
            let certificate_path = resolve_required_file(
                &base_directory,
                certificate_file,
                &format!("/certificates/{index}/source/certificateFile"),
            )?;
            let private_key_path = resolve_required_file(
                &base_directory,
                private_key_file,
                &format!("/certificates/{index}/source/privateKeyFile"),
            )?;
            certificate_paths.insert(certificate.id.clone(), (certificate_path, private_key_path));
        }

        let mut upstream_tls_paths = HashMap::new();
        for (index, upstream) in config.upstreams.iter().enumerate() {
            let Some(tls) = &upstream.tls else {
                continue;
            };
            let ca_certificates = tls
                .ca_certificate_files
                .iter()
                .enumerate()
                .map(|(file_index, configured)| {
                    resolve_protected_relative_file(
                        &base_directory,
                        configured,
                        &format!("/upstreams/{index}/tls/caCertificateFiles/{file_index}"),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let client_identity = tls
                .client_certificate_file
                .as_deref()
                .zip(tls.client_private_key_file.as_deref())
                .map(|(certificate, private_key)| {
                    Ok((
                        resolve_protected_relative_file(
                            &base_directory,
                            certificate,
                            &format!("/upstreams/{index}/tls/clientCertificateFile"),
                        )?,
                        resolve_protected_relative_file(
                            &base_directory,
                            private_key,
                            &format!("/upstreams/{index}/tls/clientPrivateKeyFile"),
                        )?,
                    ))
                })
                .transpose()?;
            upstream_tls_paths.insert(
                upstream.id.clone(),
                CompiledUpstreamTlsPaths {
                    ca_certificates,
                    client_identity,
                },
            );
        }

        let compiled_hosts = config
            .virtual_hosts
            .iter()
            .enumerate()
            .map(|(config_index, virtual_host)| {
                CompiledVirtualHost::compile(config_index, virtual_host)
            })
            .collect::<Vec<_>>();

        let host_indexes = config
            .virtual_hosts
            .iter()
            .enumerate()
            .map(|(index, host)| (host.id.as_str(), index))
            .collect::<HashMap<_, _>>();
        let mut compiled_listeners = HashMap::new();
        for listener in &config.listeners {
            let mut exact_hosts = HashMap::new();
            let mut wildcard_hosts = Vec::new();
            for (host_index, virtual_host) in config.virtual_hosts.iter().enumerate() {
                if !virtual_host
                    .listener_refs
                    .iter()
                    .any(|listener_ref| listener_ref == &listener.id)
                {
                    continue;
                }
                for server_name in &virtual_host.server_names {
                    let normalized = normalize_server_name(server_name).ok_or_else(|| {
                        validation_error(
                            format!("/virtualHosts/{host_index}/serverNames"),
                            "server name was not normalized after semantic validation",
                        )
                    })?;
                    if let Some(suffix) = normalized.strip_prefix("*.") {
                        wildcard_hosts.push((suffix.to_owned(), host_index));
                    } else {
                        exact_hosts.insert(normalized, host_index);
                    }
                }
            }
            wildcard_hosts.sort_by(|left, right| right.0.len().cmp(&left.0.len()));
            let default_host = listener
                .default_virtual_host_ref
                .as_deref()
                .and_then(|id| host_indexes.get(id).copied());
            compiled_listeners.insert(
                listener.id.clone(),
                CompiledListener {
                    exact_hosts,
                    wildcard_hosts,
                    default_host,
                },
            );
        }

        Ok(Self {
            config,
            base_directory,
            listeners,
            resources,
            upstreams,
            certificates,
            tls_policies,
            compiled_listeners,
            compiled_hosts,
            static_roots,
            certificate_paths,
            upstream_tls_paths,
        })
    }

    pub fn config(&self) -> &WebServerAppConfig {
        &self.config
    }

    pub fn base_directory(&self) -> &Path {
        &self.base_directory
    }

    pub fn listeners(&self) -> impl Iterator<Item = &ListenerConfig> {
        self.config.listeners.iter()
    }

    pub fn listener(&self, id: &str) -> Option<&ListenerConfig> {
        self.listeners
            .get(id)
            .map(|index| &self.config.listeners[*index])
    }

    pub fn resource(&self, id: &str) -> Option<&ResourceConfig> {
        self.resources
            .get(id)
            .map(|index| &self.config.resources[*index])
    }

    pub fn upstream(&self, id: &str) -> Option<&UpstreamConfig> {
        self.upstreams
            .get(id)
            .map(|index| &self.config.upstreams[*index])
    }

    pub fn tls_policy(&self, id: &str) -> Option<&TlsPolicyConfig> {
        self.tls_policies
            .get(id)
            .map(|index| &self.config.tls_policies[*index])
    }

    pub fn certificate(&self, id: &str) -> Option<&CertificateConfig> {
        self.certificates
            .get(id)
            .map(|index| &self.config.certificates[*index])
    }

    pub fn static_root(&self, resource_id: &str) -> Option<&Path> {
        self.static_roots.get(resource_id).map(PathBuf::as_path)
    }

    pub fn certificate_paths(&self, certificate_id: &str) -> Option<(&Path, &Path)> {
        self.certificate_paths
            .get(certificate_id)
            .map(|(certificate, private_key)| (certificate.as_path(), private_key.as_path()))
    }

    pub fn upstream_tls_ca_certificate_paths(&self, upstream_id: &str) -> Option<&[PathBuf]> {
        self.upstream_tls_paths
            .get(upstream_id)
            .map(|paths| paths.ca_certificates.as_slice())
    }

    pub fn upstream_tls_client_identity_paths(&self, upstream_id: &str) -> Option<(&Path, &Path)> {
        self.upstream_tls_paths
            .get(upstream_id)
            .and_then(|paths| paths.client_identity.as_ref())
            .map(|(certificate, private_key)| (certificate.as_path(), private_key.as_path()))
    }

    pub fn select_route(
        &self,
        listener_id: &str,
        authority: &str,
        path: &str,
        method: &str,
    ) -> Option<SelectedRoute<'_>> {
        let listener = self.compiled_listeners.get(listener_id)?;
        let normalized_host = normalize_authority_host(authority);
        let host_index = normalized_host
            .as_deref()
            .and_then(|host| listener.exact_hosts.get(host).copied())
            .or_else(|| {
                normalized_host.as_deref().and_then(|host| {
                    listener
                        .wildcard_hosts
                        .iter()
                        .find(|(suffix, _)| wildcard_matches(suffix, host))
                        .map(|(_, index)| *index)
                })
            })
            .or(listener.default_host)?;

        let compiled_host = self.compiled_hosts.get(host_index)?;
        let virtual_host = self.config.virtual_hosts.get(compiled_host.config_index)?;
        let route_index = compiled_host.select_route(virtual_host, path, method)?;
        let route = virtual_host.routes.get(route_index)?;
        let resource = self.resource(&route.resource_ref)?;
        Some(SelectedRoute {
            virtual_host,
            route,
            resource,
        })
    }
}

impl CompiledVirtualHost {
    fn compile(config_index: usize, virtual_host: &VirtualHostConfig) -> Self {
        let mut exact_routes: HashMap<String, Vec<usize>> = HashMap::new();
        let mut prefix_routes = PrefixMatcher::new();
        for (route_index, route) in virtual_host.routes.iter().enumerate() {
            match route.route_match.path_type {
                RoutePathType::Exact => exact_routes
                    .entry(route.route_match.path.clone())
                    .or_default()
                    .push(route_index),
                RoutePathType::Prefix => {
                    prefix_routes.insert(route.route_match.path.as_bytes(), route_index)
                }
            }
        }
        Self {
            config_index,
            exact_routes,
            prefix_routes,
        }
    }

    fn select_route(
        &self,
        virtual_host: &VirtualHostConfig,
        path: &str,
        method: &str,
    ) -> Option<usize> {
        self.exact_routes
            .get(path)
            .and_then(|routes| {
                routes
                    .iter()
                    .copied()
                    .find(|index| route_matches_method(&virtual_host.routes[*index], method))
            })
            .or_else(|| {
                self.prefix_routes.select(path.as_bytes(), |index| {
                    route_matches_method(&virtual_host.routes[index], method)
                })
            })
    }
}

impl PrefixMatcher {
    fn new() -> Self {
        Self {
            nodes: vec![PrefixNode::default()],
        }
    }

    fn insert(&mut self, prefix: &[u8], route_index: usize) {
        let mut node_index = 0usize;
        for byte in prefix {
            let next = if let Some(index) = self.nodes[node_index].children.get(byte) {
                *index
            } else {
                let index = self.nodes.len();
                self.nodes.push(PrefixNode::default());
                self.nodes[node_index].children.insert(*byte, index);
                index
            };
            node_index = next;
        }
        self.nodes[node_index].routes.push(route_index);
    }

    fn select(&self, path: &[u8], matches: impl Fn(usize) -> bool) -> Option<usize> {
        let mut node_index = 0usize;
        let mut selected = self.nodes[0]
            .routes
            .iter()
            .copied()
            .find(|index| matches(*index));
        for byte in path {
            let Some(next) = self.nodes[node_index].children.get(byte).copied() else {
                break;
            };
            node_index = next;
            if let Some(route) = self.nodes[node_index]
                .routes
                .iter()
                .copied()
                .find(|index| matches(*index))
            {
                selected = Some(route);
            }
        }
        selected
    }
}

fn index_by_id<'a>(ids: impl Iterator<Item = &'a str>) -> HashMap<String, usize> {
    ids.enumerate()
        .map(|(index, id)| (id.to_owned(), index))
        .collect()
}

fn canonical_directory(
    path: &Path,
    diagnostic_path: &str,
) -> Result<PathBuf, WebServerConfigError> {
    let canonical = fs::canonicalize(path).map_err(|error| {
        validation_error(
            diagnostic_path,
            format!("directory {} is unavailable: {error}", path.display()),
        )
    })?;
    if !canonical.is_dir() {
        return Err(validation_error(
            diagnostic_path,
            format!("{} is not a directory", canonical.display()),
        ));
    }
    Ok(canonical)
}

fn resolve_required_file(
    base_directory: &Path,
    configured: &str,
    diagnostic_path: &str,
) -> Result<PathBuf, WebServerConfigError> {
    let configured = Path::new(configured);
    let candidate = if configured.is_absolute() {
        configured.to_path_buf()
    } else {
        base_directory.join(configured)
    };
    let canonical = fs::canonicalize(&candidate).map_err(|error| {
        validation_error(
            diagnostic_path,
            format!("file {} is unavailable: {error}", candidate.display()),
        )
    })?;
    if !canonical.is_file() {
        return Err(validation_error(
            diagnostic_path,
            format!("{} is not a regular file", canonical.display()),
        ));
    }
    Ok(canonical)
}

fn resolve_protected_relative_file(
    base_directory: &Path,
    configured: &str,
    diagnostic_path: &str,
) -> Result<PathBuf, WebServerConfigError> {
    let resolved = resolve_required_file(base_directory, configured, diagnostic_path)?;
    if !resolved.starts_with(base_directory) {
        return Err(validation_error(
            diagnostic_path,
            "protected TLS file escapes the configuration directory",
        ));
    }
    Ok(resolved)
}

fn validation_error(path: impl Into<String>, message: impl Into<String>) -> WebServerConfigError {
    WebServerConfigError::Validation {
        diagnostics: vec![ConfigDiagnostic::new(path, message)],
    }
}

pub fn normalize_authority_host(authority: &str) -> Option<String> {
    let authority = authority.trim();
    if authority.is_empty() {
        return None;
    }
    if let Some(rest) = authority.strip_prefix('[') {
        let end = rest.find(']')?;
        return Some(rest[..end].to_ascii_lowercase());
    }
    let host = match authority.rsplit_once(':') {
        Some((host, port))
            if !host.contains(':') && port.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            host
        }
        _ => authority,
    };
    normalize_server_name(host)
}

fn wildcard_matches(suffix: &str, host: &str) -> bool {
    host.strip_suffix(suffix).is_some_and(|prefix| {
        prefix.ends_with('.') && prefix.len() > 1 && !prefix[..prefix.len() - 1].contains('.')
    })
}

fn route_matches_method(route: &RouteConfig, method: &str) -> bool {
    route
        .route_match
        .methods
        .as_ref()
        .is_none_or(|methods| methods.iter().any(|configured| configured == method))
}
