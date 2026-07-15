use std::{fs, path::Path};

use sdkwork_webserver_core::{
    load_and_compile_webserver_config, ResourceConfig, WebServerConfigError,
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

    let selected = compiled
        .select_route("public-http", "LOCALHOST:8080", "/healthz", "GET")
        .expect("select health route");
    assert_eq!(selected.virtual_host.id, "example-web");
    assert_eq!(selected.route.id, "health");
    assert!(matches!(selected.resource, ResourceConfig::Respond { .. }));
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
