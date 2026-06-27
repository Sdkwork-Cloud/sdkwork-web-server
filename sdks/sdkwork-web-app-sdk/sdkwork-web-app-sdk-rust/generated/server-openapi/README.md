# sdkwork-web-app-sdk (Rust)

Generated SDKWork v3 dual-token transport SDK.

## Installation

```bash
cargo add sdkwork-web-app-sdk
```

## Quick Start

```rust
use sdkwork_web_app_sdk::{SdkworkAppClient, SdkworkConfig};
use std::collections::HashMap;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SdkworkAppClient::new(SdkworkConfig::new("http://localhost:3800"))?;
    client.set_auth_token("your-auth-token");
client.set_access_token("your-access-token");

    let mut query = HashMap::new();
    query.insert("page".to_string(), serde_json::json!(1));
    query.insert("pageSize".to_string(), serde_json::json!(2));
    let result = client.certificate().certificates_list(Some(&query)).await?;
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
let client = SdkworkAppClient::new(SdkworkConfig::new("http://localhost:3800"))?;
client.set_header("X-Custom-Header", "value");
```

## API Modules

- `client.site()` - site API
- `client.domain()` - domain API
- `client.deployment()` - deployment API
- `client.env_variable()` - env_variable API
- `client.certificate()` - certificate API
- `client.monitor()` - monitor API

## Usage Examples

### site

```rust
use std::collections::HashMap;
// 获取站点列表
let mut query = HashMap::new();
query.insert("page".to_string(), serde_json::json!(1));
query.insert("pageSize".to_string(), serde_json::json!(2));
query.insert("status".to_string(), serde_json::json!(0));
query.insert("siteType".to_string(), serde_json::json!(1));
query.insert("keyword".to_string(), serde_json::json!("keyword"));
let result = client.site().sites_list(Some(&query)).await?;
println!("{result:?}");
```

### domain

```rust
use std::collections::HashMap;
// 获取站点域名列表
let site_id = "1";
let mut query = HashMap::new();
query.insert("page".to_string(), serde_json::json!(1));
query.insert("pageSize".to_string(), serde_json::json!(2));
let result = client.domain().sites_domains_list(site_id, Some(&query)).await?;
println!("{result:?}");
```

### deployment

```rust
use std::collections::HashMap;
// 获取部署历史
let site_id = "1";
let mut query = HashMap::new();
query.insert("page".to_string(), serde_json::json!(1));
query.insert("pageSize".to_string(), serde_json::json!(2));
query.insert("status".to_string(), serde_json::json!(0));
let result = client.deployment().sites_deployments_list(site_id, Some(&query)).await?;
println!("{result:?}");
```

### env_variable

```rust
use std::collections::HashMap;
// 获取环境变量列表
let site_id = "1";
let mut query = HashMap::new();
query.insert("environment".to_string(), serde_json::json!("environment"));
let result = client.env_variable().sites_env_variables_list(site_id, Some(&query)).await?;
println!("{result:?}");
```

### certificate

```rust
use std::collections::HashMap;
// 获取证书列表
let mut query = HashMap::new();
query.insert("page".to_string(), serde_json::json!(1));
query.insert("pageSize".to_string(), serde_json::json!(2));
let result = client.certificate().certificates_list(Some(&query)).await?;
println!("{result:?}");
```

### monitor

```rust
// 获取健康检查配置
let site_id = "1";
let result = client.monitor().sites_health_checks_list(site_id).await?;
println!("{result:?}");
```

## Error Handling

```rust
use sdkwork_web_app_sdk::{SdkworkAppClient, SdkworkConfig};
use std::collections::HashMap;


let client = SdkworkAppClient::new(SdkworkConfig::new("http://localhost:3800"))?;

let outcome: Result<(), _> = async {
    let mut query = HashMap::new();
    query.insert("page".to_string(), serde_json::json!(1));
    query.insert("pageSize".to_string(), serde_json::json!(2));
    client.certificate().certificates_list(Some(&query)).await?;
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
