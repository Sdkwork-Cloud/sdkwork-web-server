use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    path::{Component, Path},
};

use url::Url;

use super::{
    CertificateSource, ConfigDiagnostic, ListenerProtocol, ResourceConfig, RouteConfig, TlsVersion,
    WebServerAppConfig, WebServerConfigError,
};

const MAX_DIAGNOSTICS: usize = 128;
const MAX_TOTAL_ROUTES: usize = 10_000;
const MAX_TOTAL_UPSTREAM_TARGETS: usize = 10_000;

pub(crate) fn validate_webserver_config(
    config: &WebServerAppConfig,
) -> Result<(), WebServerConfigError> {
    let mut validator = SemanticValidator::default();
    validator.validate(config);
    validator.finish()
}

#[derive(Default)]
struct SemanticValidator {
    diagnostics: Vec<ConfigDiagnostic>,
    ids: HashMap<String, String>,
}

impl SemanticValidator {
    fn validate(&mut self, config: &WebServerAppConfig) {
        if config.schema_version != 1 {
            self.push("/schemaVersion", "only schemaVersion 1 is supported");
        }
        if config.kind != "sdkwork.webserver.app" {
            self.push("/kind", "kind must be sdkwork.webserver.app");
        }
        if config.compatibility.nginx_profile != "http-core-v1" {
            self.push(
                "/compatibility/nginxProfile",
                "only the http-core-v1 compatibility profile is supported",
            );
        }
        if config.compatibility.unknown_directive_policy != "error" {
            self.push(
                "/compatibility/unknownDirectivePolicy",
                "Rust activation requires unknownDirectivePolicy=error",
            );
        }

        self.validate_limits(config);
        self.register_ids(config);

        let listeners = config
            .listeners
            .iter()
            .map(|listener| (listener.id.as_str(), listener))
            .collect::<HashMap<_, _>>();
        let certificates = config
            .certificates
            .iter()
            .map(|certificate| (certificate.id.as_str(), certificate))
            .collect::<HashMap<_, _>>();
        let tls_policies = config
            .tls_policies
            .iter()
            .map(|policy| (policy.id.as_str(), policy))
            .collect::<HashMap<_, _>>();
        let resources = config
            .resources
            .iter()
            .map(|resource| (resource.id(), resource))
            .collect::<HashMap<_, _>>();
        let upstreams = config
            .upstreams
            .iter()
            .map(|upstream| (upstream.id.as_str(), upstream))
            .collect::<HashMap<_, _>>();
        let virtual_hosts = config
            .virtual_hosts
            .iter()
            .map(|virtual_host| (virtual_host.id.as_str(), virtual_host))
            .collect::<HashMap<_, _>>();

        let mut sockets = HashSet::new();
        for (index, listener) in config.listeners.iter().enumerate() {
            let path = format!("/listeners/{index}");
            if listener.bind.parse::<IpAddr>().is_err() {
                self.push(
                    format!("{path}/bind"),
                    "bind must be an explicit IPv4 or IPv6 address",
                );
            }
            let socket_key = format!("{}:{}", listener.bind.to_ascii_lowercase(), listener.port);
            if !sockets.insert(socket_key) {
                self.push(&path, "another listener already owns this bind and port");
            }
            if listener.protocols.contains(&ListenerProtocol::Http2)
                && listener.tls_policy_ref.is_none()
            {
                self.push(
                    format!("{path}/protocols"),
                    "HTTP/2 requires TLS in the foundation profile; public h2c is unsupported",
                );
            }
            if let Some(policy_ref) = &listener.tls_policy_ref {
                if !tls_policies.contains_key(policy_ref.as_str()) {
                    self.push(
                        format!("{path}/tlsPolicyRef"),
                        format!("unknown TLS policy {policy_ref}"),
                    );
                } else if let Some(policy) = tls_policies.get(policy_ref.as_str()) {
                    let expected_alpn = if listener.protocols == [ListenerProtocol::Http1] {
                        ["http/1.1"].as_slice()
                    } else if listener.protocols == [ListenerProtocol::Http2] {
                        ["h2"].as_slice()
                    } else {
                        ["h2", "http/1.1"].as_slice()
                    };
                    if policy.alpn.iter().map(String::as_str).collect::<Vec<_>>() != expected_alpn {
                        self.push(
                            format!("{path}/tlsPolicyRef"),
                            "TLS policy ALPN must exactly match the listener protocols",
                        );
                    }
                }
            }
            if listener
                .max_connections
                .is_some_and(|maximum| maximum > config.limits.max_connections)
            {
                self.push(
                    format!("{path}/maxConnections"),
                    "listener maxConnections cannot exceed the application maximum",
                );
            }
            if let Some(default_ref) = &listener.default_virtual_host_ref {
                match virtual_hosts.get(default_ref.as_str()) {
                    Some(virtual_host)
                        if virtual_host
                            .listener_refs
                            .iter()
                            .any(|listener_ref| listener_ref == &listener.id) => {}
                    Some(_) => self.push(
                        format!("{path}/defaultVirtualHostRef"),
                        "default virtual host does not reference this listener",
                    ),
                    None => self.push(
                        format!("{path}/defaultVirtualHostRef"),
                        format!("unknown virtual host {default_ref}"),
                    ),
                }
            }
        }

        for (index, certificate) in config.certificates.iter().enumerate() {
            let path = format!("/certificates/{index}");
            for (name_index, server_name) in certificate.server_names.iter().enumerate() {
                if normalize_server_name(server_name).is_none() {
                    self.push(
                        format!("{path}/serverNames/{name_index}"),
                        "invalid exact or leading-wildcard DNS server name",
                    );
                }
            }
            let CertificateSource::ProtectedFile {
                certificate_file,
                private_key_file,
            } = &certificate.source;
            if certificate_file == private_key_file {
                self.push(
                    format!("{path}/source"),
                    "certificateFile and privateKeyFile must be different files",
                );
            }
        }

        for (index, policy) in config.tls_policies.iter().enumerate() {
            let path = format!("/tlsPolicies/{index}");
            if !certificates.contains_key(policy.certificate_ref.as_str()) {
                self.push(
                    format!("{path}/certificateRef"),
                    format!("unknown certificate {}", policy.certificate_ref),
                );
            }
            if policy.minimum_version > policy.maximum_version {
                self.push(
                    &path,
                    "minimumVersion must not be greater than maximumVersion",
                );
            }
            if policy.minimum_version < TlsVersion::Tls12 {
                self.push(&path, "TLS versions below 1.2 are forbidden");
            }
            if policy.minimum_version != TlsVersion::Tls12
                || policy.maximum_version != TlsVersion::Tls13
            {
                self.push(
                    &path,
                    "REQ-2026-0003 currently supports the TLS 1.2 through TLS 1.3 policy only",
                );
            }
        }

        if !config.resolvers.is_empty() {
            self.push(
                "/resolvers",
                "custom resolver profiles are not implemented by REQ-2026-0003; use the bounded system resolver",
            );
        }

        for (index, resource) in config.resources.iter().enumerate() {
            let path = format!("/resources/{index}");
            match resource {
                ResourceConfig::Static {
                    root,
                    index_files,
                    spa_fallback,
                    follow_symlinks,
                    ..
                } => {
                    validate_relative_path(self, &format!("{path}/root"), root, true);
                    for (file_index, file) in index_files.iter().enumerate() {
                        validate_relative_path(
                            self,
                            &format!("{path}/indexFiles/{file_index}"),
                            file,
                            false,
                        );
                    }
                    if index_files.as_slice() != ["index.html"] {
                        self.push(
                            format!("{path}/indexFiles"),
                            "REQ-2026-0003 currently supports indexFiles=[\"index.html\"] only",
                        );
                    }
                    if let Some(file) = spa_fallback {
                        validate_relative_path(self, &format!("{path}/spaFallback"), file, false);
                    }
                    if *follow_symlinks {
                        self.push(
                            format!("{path}/followSymlinks"),
                            "following symlinks is forbidden by the foundation profile",
                        );
                    }
                }
                ResourceConfig::Proxy { upstream_ref, .. } => {
                    if !upstreams.contains_key(upstream_ref.as_str()) {
                        self.push(
                            format!("{path}/upstreamRef"),
                            format!("unknown upstream {upstream_ref}"),
                        );
                    }
                }
                ResourceConfig::Redirect {
                    status, location, ..
                } => {
                    if !matches!(status, 301 | 302 | 307 | 308) {
                        self.push(format!("{path}/status"), "unsupported redirect status");
                    }
                    validate_redirect_location(self, &format!("{path}/location"), location);
                }
                ResourceConfig::Respond { status, body, .. } => {
                    if !(200..=599).contains(status) {
                        self.push(format!("{path}/status"), "invalid HTTP response status");
                    }
                    if matches!(status, 204 | 205 | 304) && !body.is_empty() {
                        self.push(
                            format!("{path}/body"),
                            "HTTP 204, 205, and 304 fixed responses cannot contain a body",
                        );
                    }
                }
            }
        }

        let mut total_targets = 0usize;
        for (index, upstream) in config.upstreams.iter().enumerate() {
            let path = format!("/upstreams/{index}");
            total_targets = total_targets.saturating_add(upstream.targets.len());
            let mut target_urls = HashSet::new();
            for (target_index, target) in upstream.targets.iter().enumerate() {
                let target_path = format!("{path}/targets/{target_index}/url");
                match Url::parse(&target.url) {
                    Ok(url)
                        if matches!(url.scheme(), "http" | "https")
                            && url.host_str().is_some()
                            && url.username().is_empty()
                            && url.password().is_none()
                            && url.query().is_none()
                            && url.fragment().is_none() =>
                    {
                        let normalized = url.as_str().trim_end_matches('/').to_ascii_lowercase();
                        if !target_urls.insert(normalized) {
                            self.push(&target_path, "duplicate upstream target URL");
                        }
                    }
                    _ => self.push(
                        &target_path,
                        "upstream URL must be an http/https origin without credentials, query, or fragment",
                    ),
                }
                if target.weight != 1 {
                    self.push(
                        format!("{path}/targets/{target_index}/weight"),
                        "weighted upstream selection is not implemented by REQ-2026-0003; weight must be 1",
                    );
                }
            }
        }
        if total_targets > MAX_TOTAL_UPSTREAM_TARGETS {
            self.push(
                "/upstreams",
                format!(
                    "configuration has {total_targets} upstream targets; maximum is {MAX_TOTAL_UPSTREAM_TARGETS}"
                ),
            );
        }

        let mut host_owners: HashMap<(String, String), String> = HashMap::new();
        let mut total_routes = 0usize;
        for (index, virtual_host) in config.virtual_hosts.iter().enumerate() {
            let path = format!("/virtualHosts/{index}");
            for (listener_index, listener_ref) in virtual_host.listener_refs.iter().enumerate() {
                if !listeners.contains_key(listener_ref.as_str()) {
                    self.push(
                        format!("{path}/listenerRefs/{listener_index}"),
                        format!("unknown listener {listener_ref}"),
                    );
                    continue;
                }
                for (name_index, server_name) in virtual_host.server_names.iter().enumerate() {
                    let Some(normalized) = normalize_server_name(server_name) else {
                        self.push(
                            format!("{path}/serverNames/{name_index}"),
                            "invalid exact or leading-wildcard DNS server name",
                        );
                        continue;
                    };
                    let key = (listener_ref.clone(), normalized.clone());
                    if let Some(owner) = host_owners.insert(key, virtual_host.id.clone()) {
                        if owner != virtual_host.id {
                            self.push(
                                format!("{path}/serverNames/{name_index}"),
                                format!(
                                    "server name {normalized} on listener {listener_ref} is already owned by {owner}"
                                ),
                            );
                        }
                    }
                }
            }

            total_routes = total_routes.saturating_add(virtual_host.routes.len());
            validate_routes(self, &path, &virtual_host.routes, &resources);
        }
        if total_routes > MAX_TOTAL_ROUTES {
            self.push(
                "/virtualHosts",
                format!("configuration has {total_routes} routes; maximum is {MAX_TOTAL_ROUTES}"),
            );
        }

        for (listener_index, listener) in config.listeners.iter().enumerate() {
            if !config.virtual_hosts.iter().any(|virtual_host| {
                virtual_host
                    .listener_refs
                    .iter()
                    .any(|listener_ref| listener_ref == &listener.id)
            }) {
                self.push(
                    format!("/listeners/{listener_index}"),
                    "listener has no virtual hosts",
                );
            }

            let Some(policy_ref) = &listener.tls_policy_ref else {
                continue;
            };
            let Some(policy) = tls_policies.get(policy_ref.as_str()) else {
                continue;
            };
            let Some(certificate) = certificates.get(policy.certificate_ref.as_str()) else {
                continue;
            };
            for virtual_host in config.virtual_hosts.iter().filter(|virtual_host| {
                virtual_host
                    .listener_refs
                    .iter()
                    .any(|listener_ref| listener_ref == &listener.id)
            }) {
                for server_name in &virtual_host.server_names {
                    if !certificate
                        .server_names
                        .iter()
                        .any(|certificate_name| server_name_covers(certificate_name, server_name))
                    {
                        self.push(
                            format!("/listeners/{listener_index}/tlsPolicyRef"),
                            format!(
                                "certificate {} does not cover server name {server_name}",
                                certificate.id
                            ),
                        );
                    }
                }
            }
        }
    }

    fn validate_limits(&mut self, config: &WebServerAppConfig) {
        let limits = &config.limits;
        if limits.max_request_body_bytes > 2_147_483_648 {
            self.push(
                "/limits/maxRequestBodyBytes",
                "maximum request body limit is 2 GiB",
            );
        }
        if !(100..=3_600_000).contains(&limits.request_timeout_ms) {
            self.push(
                "/limits/requestTimeoutMs",
                "request timeout must be between 100 ms and 1 hour",
            );
        }
        if !(100..=600_000).contains(&limits.drain_timeout_ms) {
            self.push(
                "/limits/drainTimeoutMs",
                "drain timeout must be between 100 ms and 10 minutes",
            );
        }
        if !(1..=1_000_000).contains(&limits.max_connections) {
            self.push(
                "/limits/maxConnections",
                "maxConnections must be between 1 and 1,000,000",
            );
        }
    }

    fn register_ids(&mut self, config: &WebServerAppConfig) {
        for (index, listener) in config.listeners.iter().enumerate() {
            self.register_id(&listener.id, format!("/listeners/{index}/id"));
        }
        for (index, certificate) in config.certificates.iter().enumerate() {
            self.register_id(&certificate.id, format!("/certificates/{index}/id"));
        }
        for (index, policy) in config.tls_policies.iter().enumerate() {
            self.register_id(&policy.id, format!("/tlsPolicies/{index}/id"));
        }
        for (index, resolver) in config.resolvers.iter().enumerate() {
            self.register_id(&resolver.id, format!("/resolvers/{index}/id"));
        }
        for (index, resource) in config.resources.iter().enumerate() {
            self.register_id(resource.id(), format!("/resources/{index}/id"));
        }
        for (index, upstream) in config.upstreams.iter().enumerate() {
            self.register_id(&upstream.id, format!("/upstreams/{index}/id"));
        }
        for (host_index, virtual_host) in config.virtual_hosts.iter().enumerate() {
            self.register_id(&virtual_host.id, format!("/virtualHosts/{host_index}/id"));
            for (route_index, route) in virtual_host.routes.iter().enumerate() {
                self.register_id(
                    &route.id,
                    format!("/virtualHosts/{host_index}/routes/{route_index}/id"),
                );
            }
        }
    }

    fn register_id(&mut self, id: &str, path: String) {
        if let Some(previous_path) = self.ids.insert(id.to_owned(), path.clone()) {
            self.push(
                path,
                format!("duplicate id {id}; first declared at {previous_path}"),
            );
        }
    }

    fn push(&mut self, path: impl Into<String>, message: impl Into<String>) {
        if self.diagnostics.len() < MAX_DIAGNOSTICS {
            self.diagnostics.push(ConfigDiagnostic::new(path, message));
        }
    }

    fn finish(self) -> Result<(), WebServerConfigError> {
        if self.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(WebServerConfigError::Validation {
                diagnostics: self.diagnostics,
            })
        }
    }
}

fn validate_routes(
    validator: &mut SemanticValidator,
    host_path: &str,
    routes: &[RouteConfig],
    resources: &HashMap<&str, &ResourceConfig>,
) {
    for (index, route) in routes.iter().enumerate() {
        let path = format!("{host_path}/routes/{index}");
        let configured_path = &route.route_match.path;
        if !configured_path.starts_with('/')
            || configured_path.contains('\\')
            || configured_path.contains('?')
            || configured_path.contains('#')
            || configured_path.contains('\0')
        {
            validator.push(
                format!("{path}/match/path"),
                "route path must be an absolute URI path without query, fragment, NUL, or backslash",
            );
        }
        if !resources.contains_key(route.resource_ref.as_str()) {
            validator.push(
                format!("{path}/resourceRef"),
                format!("unknown resource {}", route.resource_ref),
            );
        }

        for (other_index, other) in routes.iter().take(index).enumerate() {
            if route.route_match.path_type == other.route_match.path_type
                && route.route_match.path == other.route_match.path
                && methods_overlap(
                    route.route_match.methods.as_deref(),
                    other.route_match.methods.as_deref(),
                )
            {
                validator.push(
                    &path,
                    format!(
                        "route overlaps route {} at {host_path}/routes/{other_index}",
                        other.id
                    ),
                );
            }
        }
    }
}

fn methods_overlap(left: Option<&[String]>, right: Option<&[String]>) -> bool {
    match (left, right) {
        (None, _) | (_, None) => true,
        (Some(left), Some(right)) => left.iter().any(|method| right.contains(method)),
    }
}

fn validate_relative_path(
    validator: &mut SemanticValidator,
    path: &str,
    value: &str,
    allow_nested: bool,
) {
    let candidate = Path::new(value);
    let has_unsafe_component = candidate.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    });
    if value.contains('\\')
        || value.contains('\0')
        || has_unsafe_component
        || (!allow_nested && candidate.components().count() != 1)
    {
        validator.push(
            path,
            "path must be a safe relative path without parent, root, prefix, backslash, or NUL",
        );
    }
}

fn validate_redirect_location(validator: &mut SemanticValidator, path: &str, location: &str) {
    if location.starts_with('/')
        && !location.starts_with("//")
        && location.bytes().all(|byte| (0x21..=0x7e).contains(&byte))
    {
        return;
    }
    match Url::parse(location) {
        Ok(url)
            if matches!(url.scheme(), "http" | "https")
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none() => {}
        _ => validator.push(
            path,
            "redirect location must be a safe absolute path or http/https URL without credentials",
        ),
    }
}

pub(crate) fn normalize_server_name(value: &str) -> Option<String> {
    let normalized = value.trim_end_matches('.').to_ascii_lowercase();
    let dns_name = normalized.strip_prefix("*.").unwrap_or(&normalized);
    if dns_name.is_empty() || dns_name.len() > 253 || dns_name.contains('*') {
        return None;
    }
    if dns_name.parse::<IpAddr>().is_ok() {
        return (!normalized.starts_with("*.")).then_some(normalized);
    }
    if dns_name.split('.').all(valid_dns_label) {
        Some(normalized)
    } else {
        None
    }
}

fn valid_dns_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

pub(crate) fn server_name_covers(certificate_name: &str, server_name: &str) -> bool {
    let Some(certificate_name) = normalize_server_name(certificate_name) else {
        return false;
    };
    let Some(server_name) = normalize_server_name(server_name) else {
        return false;
    };
    if certificate_name == server_name {
        return true;
    }
    let Some(suffix) = certificate_name.strip_prefix("*.") else {
        return false;
    };
    server_name
        .strip_suffix(suffix)
        .is_some_and(|prefix| prefix.ends_with('.') && !prefix[..prefix.len() - 1].contains('.'))
}
