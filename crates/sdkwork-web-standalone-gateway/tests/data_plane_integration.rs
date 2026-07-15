use std::{fs, net::TcpListener, path::Path, time::Duration};

use axum::{
    body::Body,
    http::{Request, Response},
    routing::any,
    Router,
};
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};
use sdkwork_web_standalone_gateway::run_data_plane_until;
use sdkwork_webserver_core::load_and_compile_webserver_config;
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::oneshot,
    task::JoinHandle,
    time::timeout,
};

fn available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve an available port");
    listener.local_addr().expect("read local address").port()
}

fn write_config(directory: &Path, config: &Value) -> std::path::PathBuf {
    let path = directory.join("sdkwork.webserver.config.json");
    fs::write(
        &path,
        serde_json::to_vec_pretty(config).expect("serialize config"),
    )
    .expect("write config");
    path
}

fn base_config(port: u16, resources: Value, upstreams: Value, routes: Value) -> Value {
    json!({
        "schemaVersion": 1,
        "kind": "sdkwork.webserver.app",
        "appKey": "sdkwork-test-web",
        "limits": {
            "maxRequestBodyBytes": 1048576,
            "requestTimeoutMs": 5000,
            "drainTimeoutMs": 1000,
            "maxConnections": 128
        },
        "listeners": [{
            "id": "http",
            "bind": "127.0.0.1",
            "port": port,
            "protocols": ["http1"],
            "defaultVirtualHostRef": "test-host",
            "maxConnections": 64
        }],
        "resources": resources,
        "upstreams": upstreams,
        "virtualHosts": [{
            "id": "test-host",
            "listenerRefs": ["http"],
            "serverNames": ["test.localhost"],
            "routes": routes
        }]
    })
}

fn spawn_data_plane(
    config_path: &Path,
) -> (
    oneshot::Sender<()>,
    JoinHandle<Result<(), sdkwork_web_standalone_gateway::DataPlaneError>>,
) {
    let compiled =
        load_and_compile_webserver_config(config_path).expect("compile data-plane config");
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        run_data_plane_until(compiled, async move {
            let _ = shutdown_rx.await;
        })
        .await
    });
    (shutdown_tx, task)
}

async fn wait_for_http(client: &reqwest::Client, url: &str, host: &str) -> reqwest::Response {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        match client.get(url).header("host", host).send().await {
            Ok(response) => return response,
            Err(error) if tokio::time::Instant::now() < deadline => {
                let _ = error;
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            Err(error) => panic!("data plane did not become ready: {error}"),
        }
    }
}

async fn stop_data_plane(
    shutdown: oneshot::Sender<()>,
    task: JoinHandle<Result<(), sdkwork_web_standalone_gateway::DataPlaneError>>,
) {
    shutdown.send(()).expect("send shutdown");
    timeout(Duration::from_secs(3), task)
        .await
        .expect("data plane drains before deadline")
        .expect("data-plane task joins")
        .expect("data plane stops cleanly");
}

async fn raw_http_status(port: u16, request: &[u8]) -> u16 {
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("connect raw HTTP client");
    stream.write_all(request).await.expect("write raw request");
    let mut response = Vec::new();
    timeout(Duration::from_secs(2), stream.read_to_end(&mut response))
        .await
        .expect("raw response completes")
        .expect("read raw response");
    let status_line = std::str::from_utf8(&response)
        .expect("HTTP response is UTF-8 in status line")
        .lines()
        .next()
        .expect("status line");
    status_line
        .split_whitespace()
        .nth(1)
        .expect("status code")
        .parse()
        .expect("numeric status code")
}

#[tokio::test]
async fn serves_fixed_and_static_routes_and_drains() {
    let directory = TempDir::new().expect("create temp directory");
    let public = directory.path().join("public");
    fs::create_dir(&public).expect("create public directory");
    fs::write(public.join("index.html"), "<h1>real static response</h1>")
        .expect("write static index");
    let port = available_port();
    let config = base_config(
        port,
        json!([
            {
                "id": "health-response",
                "type": "respond",
                "status": 200,
                "body": "ok\n"
            },
            {
                "id": "static-files",
                "type": "static",
                "root": "public",
                "indexFiles": ["index.html"],
                "followSymlinks": false
            }
        ]),
        json!([]),
        json!([
            {
                "id": "health",
                "match": {"pathType": "exact", "path": "/healthz"},
                "resourceRef": "health-response"
            },
            {
                "id": "static",
                "match": {"pathType": "prefix", "path": "/"},
                "resourceRef": "static-files"
            }
        ]),
    );
    let path = write_config(directory.path(), &config);
    let (shutdown, task) = spawn_data_plane(&path);
    let client = reqwest::Client::new();

    let health = wait_for_http(
        &client,
        &format!("http://127.0.0.1:{port}/healthz"),
        "test.localhost",
    )
    .await;
    assert_eq!(health.status(), reqwest::StatusCode::OK);
    assert_eq!(health.text().await.expect("read health body"), "ok\n");

    let index = client
        .get(format!("http://127.0.0.1:{port}/"))
        .header("host", "test.localhost")
        .send()
        .await
        .expect("request static index");
    assert_eq!(index.status(), reqwest::StatusCode::OK);
    assert!(index
        .text()
        .await
        .expect("read static body")
        .contains("real static response"));

    assert_eq!(
        raw_http_status(
            port,
            b"GET /%2e%2e/secret HTTP/1.1\r\nHost: test.localhost\r\nConnection: close\r\n\r\n",
        )
        .await,
        403
    );
    assert_eq!(
        raw_http_status(
            port,
            b"GET http://evil.example/ HTTP/1.1\r\nHost: test.localhost\r\nConnection: close\r\n\r\n",
        )
        .await,
        400
    );

    stop_data_plane(shutdown, task).await;
}

#[tokio::test]
async fn streams_proxy_body_and_enforces_body_limit() {
    let upstream_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind upstream");
    let upstream_address = upstream_listener.local_addr().expect("upstream address");
    let (upstream_shutdown_tx, upstream_shutdown_rx) = oneshot::channel();
    let upstream_task = tokio::spawn(async move {
        let app = Router::new().route(
            "/echo",
            any(|request: Request<Body>| async move {
                let (_, body) = request.into_parts();
                Response::new(body)
            }),
        );
        axum::serve(upstream_listener, app)
            .with_graceful_shutdown(async move {
                let _ = upstream_shutdown_rx.await;
            })
            .await
            .expect("serve upstream");
    });

    let directory = TempDir::new().expect("create temp directory");
    let port = available_port();
    let mut config = base_config(
        port,
        json!([{
            "id": "proxy",
            "type": "proxy",
            "upstreamRef": "echo-upstream",
            "stripPrefix": false
        }]),
        json!([{
            "id": "echo-upstream",
            "targets": [{"url": format!("http://{upstream_address}")}],
            "connectTimeoutMs": 1000,
            "requestTimeoutMs": 5000,
            "maxIdleConnections": 8
        }]),
        json!([{
            "id": "proxy-route",
            "match": {"pathType": "prefix", "path": "/"},
            "resourceRef": "proxy"
        }]),
    );
    config["limits"]["maxRequestBodyBytes"] = json!(4);
    let path = write_config(directory.path(), &config);
    let (shutdown, task) = spawn_data_plane(&path);
    let client = reqwest::Client::new();

    let ready = wait_for_http(
        &client,
        &format!("http://127.0.0.1:{port}/echo"),
        "test.localhost",
    )
    .await;
    assert_eq!(ready.status(), reqwest::StatusCode::OK);

    let echoed = client
        .post(format!("http://127.0.0.1:{port}/echo"))
        .header("host", "test.localhost")
        .body("four")
        .send()
        .await
        .expect("proxy bounded body");
    assert_eq!(echoed.status(), reqwest::StatusCode::OK);
    assert_eq!(echoed.text().await.expect("read echo"), "four");

    let rejected = client
        .post(format!("http://127.0.0.1:{port}/echo"))
        .header("host", "test.localhost")
        .body("exceeds")
        .send()
        .await
        .expect("request oversized body");
    assert_eq!(rejected.status(), reqwest::StatusCode::PAYLOAD_TOO_LARGE);

    stop_data_plane(shutdown, task).await;
    upstream_shutdown_tx.send(()).expect("stop upstream");
    timeout(Duration::from_secs(3), upstream_task)
        .await
        .expect("upstream drains")
        .expect("upstream task joins");
}

#[tokio::test]
async fn serves_https_with_rustls() {
    let directory = TempDir::new().expect("create temp directory");
    let mut params = CertificateParams::new(vec!["localhost".to_owned()]).expect("cert params");
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(DnType::CommonName, "localhost");
    let key = KeyPair::generate().expect("generate key");
    let certificate = params.self_signed(&key).expect("generate certificate");
    fs::write(directory.path().join("cert.pem"), certificate.pem()).expect("write cert");
    fs::write(directory.path().join("key.pem"), key.serialize_pem()).expect("write key");

    let port = available_port();
    let config = json!({
        "schemaVersion": 1,
        "kind": "sdkwork.webserver.app",
        "appKey": "sdkwork-https-test",
        "limits": {
            "requestTimeoutMs": 5000,
            "drainTimeoutMs": 1000,
            "maxConnections": 64
        },
        "listeners": [{
            "id": "https",
            "bind": "127.0.0.1",
            "port": port,
            "protocols": ["http1", "http2"],
            "tlsPolicyRef": "tls",
            "defaultVirtualHostRef": "https-host"
        }],
        "certificates": [{
            "id": "cert",
            "serverNames": ["localhost"],
            "source": {
                "type": "protected-file",
                "certificateFile": "cert.pem",
                "privateKeyFile": "key.pem"
            }
        }],
        "tlsPolicies": [{
            "id": "tls",
            "certificateRef": "cert",
            "minimumVersion": "tls1.2",
            "maximumVersion": "tls1.3",
            "alpn": ["h2", "http/1.1"]
        }],
        "resources": [{
            "id": "secure-response",
            "type": "respond",
            "status": 200,
            "body": "secure\n"
        }],
        "virtualHosts": [{
            "id": "https-host",
            "listenerRefs": ["https"],
            "serverNames": ["localhost"],
            "routes": [{
                "id": "secure",
                "match": {"pathType": "prefix", "path": "/"},
                "resourceRef": "secure-response"
            }]
        }]
    });
    let path = write_config(directory.path(), &config);
    let (shutdown, task) = spawn_data_plane(&path);
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("build TLS test client");
    let response = wait_for_http(&client, &format!("https://localhost:{port}/"), "localhost").await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(response.text().await.expect("read TLS body"), "secure\n");

    stop_data_plane(shutdown, task).await;
}

#[tokio::test]
async fn rejects_connections_over_limit_without_blocking_shutdown() {
    let directory = TempDir::new().expect("create temp directory");
    let port = available_port();
    let mut config = base_config(
        port,
        json!([{
            "id": "response",
            "type": "respond",
            "status": 200,
            "body": "ok"
        }]),
        json!([]),
        json!([{
            "id": "response-route",
            "match": {"pathType": "prefix", "path": "/"},
            "resourceRef": "response"
        }]),
    );
    config["limits"]["maxConnections"] = json!(1);
    config["listeners"][0]["maxConnections"] = json!(1);
    let path = write_config(directory.path(), &config);
    let (shutdown, task) = spawn_data_plane(&path);

    let mut first = loop {
        match tokio::net::TcpStream::connect(("127.0.0.1", port)).await {
            Ok(stream) => break stream,
            Err(_) => tokio::time::sleep(Duration::from_millis(25)).await,
        }
    };
    first
        .write_all(b"GET / HTTP/1.1\r\nHost: test.localhost\r\n")
        .await
        .expect("hold first connection open");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut second = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("TCP backlog accepts the overload connection");
    second
        .write_all(b"GET / HTTP/1.1\r\nHost: test.localhost\r\n\r\n")
        .await
        .expect("write overload request");
    let mut buffer = [0u8; 1];
    let result = timeout(Duration::from_secs(1), second.read(&mut buffer))
        .await
        .expect("overload connection is closed promptly");
    assert!(matches!(result, Ok(0) | Err(_)));

    stop_data_plane(shutdown, task).await;
    let _ = first.shutdown().await;
}
