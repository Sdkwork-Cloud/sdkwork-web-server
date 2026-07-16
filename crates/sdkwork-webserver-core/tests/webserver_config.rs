use std::{fs, path::Path};

use sdkwork_webserver_core::{
    inspect_webserver_config_revision, load_and_compile_webserver_config,
    load_and_compile_webserver_config_revision, ResourceConfig, WebServerConfigError,
};
use serde_json::{json, Value};
use tempfile::TempDir;

fn base_config() -> Value {
    json!({
        "schemaVersion": 1,
        "kind": "sdkwork.webserver.app",
        "appKey": "sdkwork-test-web",
        "listeners": [{
            "id": "http",
            "bind": "127.0.0.1",
            "port": 18080,
            "protocols": ["http1"],
            "defaultVirtualHostRef": "default-host"
        }],
        "resources": [
            {
                "id": "exact-response",
                "type": "respond",
                "status": 200,
                "body": "exact"
            },
            {
                "id": "prefix-response",
                "type": "respond",
                "status": 200,
                "body": "prefix"
            },
            {
                "id": "wildcard-response",
                "type": "respond",
                "status": 200,
                "body": "wildcard"
            }
        ],
        "virtualHosts": [
            {
                "id": "default-host",
                "listenerRefs": ["http"],
                "serverNames": ["example.com"],
                "routes": [
                    {
                        "id": "exact-route",
                        "match": {
                            "pathType": "exact",
                            "path": "/api/status",
                            "methods": ["GET"]
                        },
                        "resourceRef": "exact-response"
                    },
                    {
                        "id": "prefix-route",
                        "match": {
                            "pathType": "prefix",
                            "path": "/api"
                        },
                        "resourceRef": "prefix-response"
                    }
                ]
            },
            {
                "id": "wildcard-host",
                "listenerRefs": ["http"],
                "serverNames": ["*.example.net"],
                "routes": [{
                    "id": "wildcard-route",
                    "match": {
                        "pathType": "prefix",
                        "path": "/"
                    },
                    "resourceRef": "wildcard-response"
                }]
            }
        ]
    })
}

fn write_config(directory: &Path, config: &Value) -> std::path::PathBuf {
    let path = directory.join("sdkwork.webserver.config.json");
    fs::write(
        &path,
        serde_json::to_vec_pretty(config).expect("serialize test config"),
    )
    .expect("write test config");
    path
}

#[test]
fn checked_in_example_validates_and_compiles() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/examples/sdkwork.webserver.config.json");
    let compiled = load_and_compile_webserver_config(path).expect("compile checked-in example");
    assert_eq!(compiled.config().limits.max_connection_age_ms, 3_600_000);

    let selected = compiled
        .select_route("public-http", "LOCALHOST:8080", "/healthz", "GET")
        .expect("select health route");
    assert_eq!(selected.virtual_host.id, "example-web");
    assert_eq!(selected.route.id, "health");
    assert!(matches!(selected.resource, ResourceConfig::Respond { .. }));
}

#[test]
fn configuration_revision_is_stable_for_exact_bytes_and_changes_with_content() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    let path = write_config(directory.path(), &config);

    let first = load_and_compile_webserver_config_revision(&path).expect("compile first revision");
    let second =
        load_and_compile_webserver_config_revision(&path).expect("compile identical revision");
    let inspected = inspect_webserver_config_revision(&path).expect("inspect identical revision");
    assert_eq!(first.sha256(), second.sha256());
    assert_eq!(first.sha256(), inspected.sha256());
    assert_eq!(first.size_bytes(), inspected.size_bytes());
    assert_eq!(first.sha256().len(), 64);
    assert!(first.size_bytes() > 0);

    config["resources"][0]["body"] = json!("changed");
    write_config(directory.path(), &config);
    let changed =
        load_and_compile_webserver_config_revision(&path).expect("compile changed revision");
    assert_ne!(first.sha256(), changed.sha256());
}

#[test]
fn exact_route_precedes_prefix_and_methods_are_enforced() {
    let directory = TempDir::new().expect("create temp directory");
    let path = write_config(directory.path(), &base_config());
    let compiled = load_and_compile_webserver_config(path).expect("compile config");

    let get = compiled
        .select_route("http", "example.com", "/api/status", "GET")
        .expect("select exact GET route");
    assert_eq!(get.route.id, "exact-route");

    let post = compiled
        .select_route("http", "example.com", "/api/status", "POST")
        .expect("fall back to prefix POST route");
    assert_eq!(post.route.id, "prefix-route");

    let wildcard = compiled
        .select_route("http", "api.eu.example.net:18080", "/", "GET")
        .expect("select wildcard host");
    assert_eq!(wildcard.virtual_host.id, "wildcard-host");
}

#[test]
fn schema_rejects_unknown_fields() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["listeners"][0]["pretendTls"] = json!(true);
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path).expect_err("unknown field must fail");
    assert!(matches!(error, WebServerConfigError::Validation { .. }));
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.message.contains("pretendTls")));
}

#[test]
fn schema_rejects_unsupported_route_modes_and_unbounded_limits() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["virtualHosts"][0]["routes"][0]["match"]["pathType"] = json!("regex");
    config["limits"] = json!({"maxConnections": 1_000_001});
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("unsupported route modes and unbounded limits must fail");
    assert!(matches!(error, WebServerConfigError::Validation { .. }));
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("pathType")));
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("maxConnections")));
}

#[test]
fn schema_rejects_invalid_response_progress_timeouts() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["limits"] = json!({
        "responseBodyIdleTimeoutMs": 99,
        "connectionWriteTimeoutMs": 3_600_001
    });
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("response progress timeouts must stay within finite bounds");
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("responseBodyIdleTimeoutMs")));
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("connectionWriteTimeoutMs")));
}

#[test]
fn schema_rejects_invalid_request_body_progress_timeouts() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["limits"] = json!({
        "requestBodyStartTimeoutMs": 99,
        "requestBodyIdleTimeoutMs": 3_600_001
    });
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("request Body progress timeouts must stay within finite bounds");
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("requestBodyStartTimeoutMs")));
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("requestBodyIdleTimeoutMs")));
}

#[test]
fn schema_rejects_invalid_http1_keep_alive_idle_timeout() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["limits"] = json!({"http1KeepAliveIdleTimeoutMs": 99});
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("HTTP/1 Keep-Alive idle timeout must stay within finite bounds");
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("http1KeepAliveIdleTimeoutMs")));
}

#[test]
fn schema_rejects_invalid_http1_pipeline_depth() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["limits"] = json!({"http1MaxPipelineDepth": 0});
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("zero HTTP/1 Pipeline depth must fail schema validation");
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("http1MaxPipelineDepth")));
}

#[test]
fn schema_rejects_connection_maximum_age_outside_the_finite_range() {
    for value in [99_u64, 86_400_001_u64] {
        let directory = TempDir::new().expect("create temp directory");
        let mut config = base_config();
        config["limits"] = json!({"maxConnectionAgeMs": value});
        let path = write_config(directory.path(), &config);

        let error = load_and_compile_webserver_config(path)
            .expect_err("connection maximum age outside the finite range must fail");
        assert!(error
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.path.contains("maxConnectionAgeMs")));
    }
}

#[test]
fn upstream_address_policy_rejects_private_literals_and_broad_cidrs() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["resolvers"] = json!([{
        "id": "system-dns",
        "servers": ["8.8.8.8"],
        "timeoutMs": 2_000,
        "maximumAnswers": 16,
        "maxConcurrentQueries": 64
    }]);
    config["upstreams"] = json!([{
        "id": "private-upstream",
        "resolverRef": "missing-resolver",
        "addressPolicy": {"allowedCidrs": ["0.0.0.0/0"]},
        "targets": [{"url": "http://127.0.0.1:8080"}]
    }]);
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("unsafe resolver and address policy must fail compilation");
    assert!(error.diagnostics().iter().any(|diagnostic| {
        diagnostic.path.contains("/resolvers/0/servers")
            && diagnostic.message.contains("not implemented")
    }));
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("resolverRef")));
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("allowedCidrs")));
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("literal upstream IP is forbidden")));
}

#[test]
fn upstream_address_policy_allows_explicit_narrow_loopback_cidr() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["resolvers"] = json!([{
        "id": "system-dns",
        "timeoutMs": 2_000,
        "maximumAnswers": 16,
        "maxConcurrentQueries": 64
    }]);
    config["upstreams"] = json!([{
        "id": "local-upstream",
        "resolverRef": "system-dns",
        "addressPolicy": {"allowedCidrs": ["127.0.0.1/32"]},
        "idleConnectionTimeoutMs": 1_000,
        "targets": [{"url": "http://127.0.0.1:8080"}]
    }]);
    let path = write_config(directory.path(), &config);

    load_and_compile_webserver_config(path).expect("explicit narrow loopback policy must compile");
}

#[test]
fn schema_rejects_unbounded_resolver_controls() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["resolvers"] = json!([{
        "id": "system-dns",
        "maxConcurrentQueries": 1_025
    }]);
    config["upstreams"] = json!([{
        "id": "invalid-upstream",
        "addressPolicy": {"allowedCidrs": ["127.0.0.1/32"]},
        "idleConnectionTimeoutMs": 99,
        "targets": [{"url": "https://example.com"}]
    }]);
    let path = write_config(directory.path(), &config);

    let error =
        load_and_compile_webserver_config(path).expect_err("unbounded resolver controls must fail");
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("maxConcurrentQueries")));
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("idleConnectionTimeoutMs")));
}

#[test]
fn configuration_rejects_malformed_allowed_cidr() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["upstreams"] = json!([{
        "id": "invalid-upstream",
        "addressPolicy": {"allowedCidrs": ["not-a-cidr"]},
        "targets": [{"url": "https://example.com"}]
    }]);
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("malformed allowed CIDR must fail deserialization");
    assert!(matches!(error, WebServerConfigError::Json { .. }));
}

#[test]
fn schema_rejects_invalid_http2_keep_alive_policy() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["limits"] = json!({
        "http2KeepAliveIntervalMs": 999,
        "http2KeepAliveTimeoutMs": 500
    });
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("unsafe HTTP/2 Keep-Alive interval must fail Schema validation");
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("http2KeepAliveIntervalMs")));

    config["limits"] = json!({
        "http2KeepAliveIntervalMs": 1000,
        "http2KeepAliveTimeoutMs": 1001
    });
    let path = write_config(directory.path(), &config);
    let error = load_and_compile_webserver_config(path)
        .expect_err("HTTP/2 Keep-Alive timeout beyond interval must fail semantic validation");
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.message.contains("must not exceed its interval")));
}

#[test]
fn semantic_validation_rejects_incoherent_uri_query_budgets() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["limits"] = json!({
        "maxUriPathBytes": 100,
        "maxDecodedPathBytes": 101,
        "maxQueryStringBytes": 0,
        "maxQueryParameters": 1,
        "maxQueryComponentBytes": 1
    });
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("URI and Query budgets must remain coherent");
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("maxDecodedPathBytes must not exceed maxUriPathBytes")));
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("query string, parameter, and component budgets")));
}

#[test]
fn semantic_validation_requires_canonical_route_paths() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["virtualHosts"][0]["routes"][0]["match"]["path"] = json!("/api/../status");
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("non-canonical route path must not enter compiled indexes");
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("route path must be canonical; use /status")));
}

#[test]
fn canonical_route_paths_allow_decoded_reserved_path_characters() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["virtualHosts"][0]["routes"][0]["match"]["path"] = json!("/a?b#c%d");
    let path = write_config(directory.path(), &config);

    load_and_compile_webserver_config(path)
        .expect("reserved characters are data in a canonical Path, not URI delimiters");
}

#[test]
fn semantic_validation_rejects_unbounded_http2_per_connection_budgets() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["limits"] = json!({
        "maxConnections": 1_000_000,
        "maxRequestHeaderBytes": 8_192,
        "http2MaxConcurrentStreams": 10_000,
        "http2MaxSendBufferBytes": 16_777_216,
        "http2MaxHeaderListBytes": 1_048_576
    });
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("aggregate HTTP/2 connection budgets must remain bounded");
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("send-buffer budget must not exceed 64 MiB")));
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("header-list budget must not exceed 64 MiB")));
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("connection header-window budget must not exceed 1 GiB")));
}

#[test]
fn semantic_validation_rejects_unbounded_http2_abuse_budgets() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["limits"] = json!({
        "http2MaxConcurrentStreams": 10_000,
        "http2MaxSendBufferBytes": 1_024,
        "http2MaxHeaderListBytes": 1_024,
        "http2MaxFrameBytes": 16_384,
        "http2MaxFramesPerWindow": 100,
        "http2MaxNewStreamsPerWindow": 101,
        "http2MaxResetFramesPerWindow": 101,
        "http2MaxEncodedHeaderBlockBytes": 1_048_576
    });
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("HTTP/2 abuse budgets must remain bounded");
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("http2MaxNewStreamsPerWindow must not exceed http2MaxFramesPerWindow")));
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("http2MaxResetFramesPerWindow must not exceed http2MaxFramesPerWindow")));
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("encoded-header budget must not exceed 64 MiB")));
}

#[test]
fn semantic_validation_rejects_unbounded_process_request_budgets() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["limits"] = json!({
        "maxConnections": 2_000,
        "maxConcurrentRequests": 2_000,
        "maxRequestHeaderBytes": 8_192,
        "http2MaxConcurrentStreams": 1,
        "http2MaxSendBufferBytes": 1_048_576,
        "http2MaxHeaderListBytes": 1_048_576,
        "http2MaxEncodedHeaderBlockBytes": 1_048_576
    });
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("process request and connection budgets must remain bounded");
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("active HTTP/2 header-list budget must not exceed 1 GiB")));
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("active HTTP/2 send-buffer budget must not exceed 1 GiB")));
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("encoded-header connection budget must not exceed 1 GiB")));
}

#[test]
fn semantic_validation_rejects_incoherent_http1_field_budgets() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["limits"] = json!({
        "maxRequestHeaderBytes": 8_192,
        "maxRequestLineBytes": 8_192,
        "maxRequestMethodBytes": 32,
        "maxRequestTargetBytes": 8_193,
        "maxHeaderNameBytes": 256,
        "maxHeaderValueBytes": 8_193
    });
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("individual HTTP/1 budgets must fit their enclosing budget");
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("maxRequestTargetBytes must not exceed maxRequestLineBytes")));
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("maxHeaderValueBytes must not exceed maxRequestHeaderBytes")));
}

#[test]
fn semantic_validation_rejects_incoherent_trailer_budgets() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["limits"] = json!({
        "maxTrailerBytes": 0,
        "maxTrailers": 1
    });
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path)
        .expect_err("trailer count cannot be enabled without a byte budget");
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("must both be zero or both be positive")));
}

#[test]
fn validation_rejects_invalid_server_names() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["virtualHosts"][0]["serverNames"][0] = json!("invalid host name");
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path).expect_err("invalid host name must fail");
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("serverNames")));
}

#[test]
fn semantic_validation_rejects_duplicate_ids_and_missing_references() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["resources"][1]["id"] = json!("exact-response");
    config["virtualHosts"][0]["routes"][0]["resourceRef"] = json!("missing-resource");
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path).expect_err("invalid references must fail");
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.message.contains("duplicate id")));
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.message.contains("unknown resource")));
}

#[test]
fn semantic_validation_rejects_static_root_escape() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["resources"]
        .as_array_mut()
        .expect("resources array")
        .push(json!({
            "id": "unsafe-static",
            "type": "static",
            "root": "../outside",
            "followSymlinks": false
        }));
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path).expect_err("unsafe path must fail");
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.ends_with("/root")));
}

#[test]
fn compilation_rejects_missing_tls_files() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["listeners"][0]["protocols"] = json!(["http1", "http2"]);
    config["listeners"][0]["tlsPolicyRef"] = json!("public-tls");
    config["certificates"] = json!([{
        "id": "public-cert",
        "serverNames": ["example.com", "*.example.net"],
        "source": {
            "type": "protected-file",
            "certificateFile": "missing-cert.pem",
            "privateKeyFile": "missing-key.pem"
        }
    }]);
    config["tlsPolicies"] = json!([{
        "id": "public-tls",
        "certificateRef": "public-cert",
        "minimumVersion": "tls1.2",
        "maximumVersion": "tls1.3",
        "alpn": ["h2", "http/1.1"]
    }]);
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path).expect_err("missing TLS files must fail");
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.path.contains("certificateFile")));
}

#[test]
fn multiple_certificate_tls_policy_covers_all_listener_hosts() {
    let directory = TempDir::new().expect("create temp directory");
    for file in ["primary.pem", "primary.key", "wildcard.pem", "wildcard.key"] {
        fs::write(directory.path().join(file), b"test material").expect("write TLS material");
    }
    let mut config = base_config();
    config["listeners"][0]["tlsPolicyRef"] = json!("public-tls");
    config["certificates"] = json!([
        {
            "id": "primary-cert",
            "serverNames": ["example.com"],
            "source": {
                "type": "protected-file",
                "certificateFile": "primary.pem",
                "privateKeyFile": "primary.key"
            }
        },
        {
            "id": "wildcard-cert",
            "serverNames": ["*.example.net"],
            "source": {
                "type": "protected-file",
                "certificateFile": "wildcard.pem",
                "privateKeyFile": "wildcard.key"
            }
        }
    ]);
    config["tlsPolicies"] = json!([{
        "id": "public-tls",
        "certificateRefs": ["primary-cert", "wildcard-cert"],
        "minimumVersion": "tls1.2",
        "maximumVersion": "tls1.3",
        "alpn": ["http/1.1"]
    }]);

    let path = write_config(directory.path(), &config);
    let compiled = load_and_compile_webserver_config(path).expect("compile multi-certificate TLS");
    let policy = compiled
        .tls_policy("public-tls")
        .expect("compiled TLS policy");
    assert_eq!(
        policy.certificate_refs().collect::<Vec<_>>(),
        ["primary-cert", "wildcard-cert"]
    );
}

#[test]
fn schema_rejects_ambiguous_tls_certificate_reference_shapes() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["tlsPolicies"] = json!([{
        "id": "invalid-tls",
        "certificateRef": "first",
        "certificateRefs": ["second"]
    }]);
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path).expect_err("ambiguous shape must fail");
    assert!(matches!(error, WebServerConfigError::Validation { .. }));
}

#[test]
fn semantic_validation_rejects_normalized_certificate_name_duplicates() {
    let directory = TempDir::new().expect("create temp directory");
    fs::write(directory.path().join("cert.pem"), b"test").expect("write certificate");
    fs::write(directory.path().join("key.pem"), b"test").expect("write key");
    let mut config = base_config();
    config["certificates"] = json!([{
        "id": "duplicate-name-cert",
        "serverNames": ["EXAMPLE.com", "example.com."],
        "source": {
            "type": "protected-file",
            "certificateFile": "cert.pem",
            "privateKeyFile": "key.pem"
        }
    }]);
    let path = write_config(directory.path(), &config);

    let error =
        load_and_compile_webserver_config(path).expect_err("normalized duplicate must fail");
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.message.contains("after DNS normalization")));
}

#[test]
fn semantic_validation_rejects_duplicate_sni_ownership_in_one_policy() {
    let directory = TempDir::new().expect("create temp directory");
    for file in ["first.pem", "first.key", "second.pem", "second.key"] {
        fs::write(directory.path().join(file), b"test").expect("write TLS material");
    }
    let mut config = base_config();
    config["listeners"][0]["tlsPolicyRef"] = json!("public-tls");
    config["certificates"] = json!([
        {
            "id": "first-cert",
            "serverNames": ["example.com", "*.example.net"],
            "source": {
                "type": "protected-file",
                "certificateFile": "first.pem",
                "privateKeyFile": "first.key"
            }
        },
        {
            "id": "second-cert",
            "serverNames": ["EXAMPLE.COM"],
            "source": {
                "type": "protected-file",
                "certificateFile": "second.pem",
                "privateKeyFile": "second.key"
            }
        }
    ]);
    config["tlsPolicies"] = json!([{
        "id": "public-tls",
        "certificateRefs": ["first-cert", "second-cert"],
        "minimumVersion": "tls1.2",
        "maximumVersion": "tls1.3",
        "alpn": ["http/1.1"]
    }]);
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path).expect_err("duplicate SNI owner must fail");
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("both declare server name example.com")));
}

#[test]
fn semantic_validation_rejects_listener_socket_conflicts() {
    let directory = TempDir::new().expect("create temp directory");
    let mut config = base_config();
    config["listeners"]
        .as_array_mut()
        .expect("listeners array")
        .push(json!({
            "id": "duplicate-socket",
            "bind": "127.0.0.1",
            "port": 18080,
            "protocols": ["http1"]
        }));
    let path = write_config(directory.path(), &config);

    let error = load_and_compile_webserver_config(path).expect_err("socket conflict must fail");
    assert!(error.diagnostics().iter().any(|diagnostic| diagnostic
        .message
        .contains("already owns this bind and port")));
}

#[test]
fn loader_rejects_configuration_larger_than_one_megabyte() {
    let directory = TempDir::new().expect("create temp directory");
    let path = directory.path().join("oversized.json");
    fs::write(&path, vec![b' '; 1024 * 1024 + 1]).expect("write oversized config");

    let error = load_and_compile_webserver_config(path).expect_err("oversized config must fail");
    assert!(matches!(error, WebServerConfigError::TooLarge { .. }));
}
