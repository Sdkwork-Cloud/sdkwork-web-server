# sdkwork-web-backend-sdk (Rust)

Generated SDKWork v3 dual-token transport SDK.

## Installation

```bash
cargo add sdkwork-web-backend-sdk
```

## Quick Start

```rust
use sdkwork_web_backend_sdk::{SdkworkBackendClient, SdkworkConfig};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SdkworkBackendClient::new(SdkworkConfig::new("http://localhost:3800"))?;
    client.set_auth_token("your-auth-token");
client.set_access_token("your-access-token");

    let result = client.nginx().status_retrieve().await?;
    println!("{result:?}");
    Ok(())
}
```

## Authentication

```text
Authorization: Bearer <authToken>
Access-Token: <accessToken>
```


## Configuration (Non-Auth)

```rust
let client = SdkworkBackendClient::new(SdkworkConfig::new("http://localhost:3800"))?;
client.set_header("X-Custom-Header", "value");
```

## API Modules

- `client.nginx()` - nginx API
- `client.server()` - server API
- `client.agent()` - agent API
- `client.audit()` - audit API

## Usage Examples

### nginx

```rust
// 获取 Nginx 状态
let result = client.nginx().status_retrieve().await?;
println!("{result:?}");
```

### server

```rust
use std::collections::HashMap;
// 获取服务器列表
let mut query = HashMap::new();
query.insert("page".to_string(), serde_json::json!(1));
query.insert("pageSize".to_string(), serde_json::json!(2));
let result = client.server().servers_list(Some(&query)).await?;
println!("{result:?}");
```

### agent

```rust
use std::collections::HashMap;
// 拉取 nginx 配置与证书 bundle
let mut query = HashMap::new();
query.insert("ifSyncVersion".to_string(), serde_json::json!("ifsyncversion"));
let result = client.agent().sync(Some(&query)).await?;
println!("{result:?}");
```

### audit

```rust
use std::collections::HashMap;
// 获取审计日志列表
let mut query = HashMap::new();
query.insert("page".to_string(), serde_json::json!(1));
query.insert("pageSize".to_string(), serde_json::json!(2));
query.insert("targetType".to_string(), serde_json::json!("targettype"));
query.insert("action".to_string(), serde_json::json!("action"));
query.insert("operatorId".to_string(), serde_json::json!("1"));
query.insert("startDate".to_string(), serde_json::json!("2026-04-10T00:00:00Z"));
query.insert("endDate".to_string(), serde_json::json!("2026-04-10T00:00:00Z"));
let result = client.audit().logs_list(Some(&query)).await?;
println!("{result:?}");
```

## Error Handling

```rust
use sdkwork_web_backend_sdk::{SdkworkBackendClient, SdkworkConfig};


let client = SdkworkBackendClient::new(SdkworkConfig::new("http://localhost:3800"))?;

let outcome: Result<(), _> = async {
    client.nginx().status_retrieve().await?;
    Ok(())
}.await;

match outcome {
    Ok(()) => println!("request completed"),
    Err(error) => eprintln!("request failed: {error}"),
}
```

## Publishing

This SDK includes cross-platform publish scripts in `bin/`:
- `bin/publish-core.mjs`
- `bin/publish.sh`
- `bin/publish.ps1`

### Check

```bash
./bin/publish.sh --action check
```

### Publish

```bash
./bin/publish.sh --action publish --channel release
```

```powershell
.\bin\publish.ps1 --action publish --channel test --dry-run
```

> Set cargo registry credentials before `cargo publish` and use `--dry-run` first.

## License

MIT

## Regeneration Contract

- HTTP/OpenAPI generator-owned files are tracked in `.sdkwork/sdkwork-generator-manifest.json`.
- HTTP/OpenAPI generation also writes `.sdkwork/sdkwork-generator-changes.json` so automation can inspect created, updated, deleted, unchanged, scaffolded, and backed-up files plus the classified impact areas, verification plan, and execution decision for the latest generation.
- HTTP/OpenAPI apply mode also writes `.sdkwork/sdkwork-generator-report.json` with the full execution report, including `schemaVersion`, `generator`, stable artifact paths, and the execution handoff commands that match CLI `--json` output.
- CLI JSON output also includes an execution handoff with concrete next commands, including reviewed apply commands for dry-run flows.
- Put HTTP/OpenAPI hand-written wrappers, adapters, and orchestration in `custom/`.
- Files scaffolded under `custom/` are created once and preserved across HTTP/OpenAPI regenerations.
- If an HTTP/OpenAPI generated-owned file was modified locally, its previous content is copied to `.sdkwork/manual-backups/` before overwrite or removal.
- RPC SDK source workspaces use convention-first evidence by default: RPC SDK family naming, language workspace naming, `rpc/*.manifest.json`, proto source references, generated client source, and native package manifests.
- Use `sdkgen inspect --protocol rpc` to verify RPC convention evidence. Request persisted generator evidence only with `--emit-control-plane` for release, CI, audit, or migration workflows; evidence paths are derived by generator convention.
