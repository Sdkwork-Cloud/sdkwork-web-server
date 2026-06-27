# sdkwork-web-backend-sdk (Go)

Generated SDKWork v3 dual-token transport SDK.

## Installation

```bash
go get github.com/sdkwork/sdkwork-web-backend-sdk
```

## Quick Start

```go
package main

import (
    "fmt"
    "github.com/sdkwork/sdkwork-web-backend-sdk"
    sdkhttp "github.com/sdkwork/sdkwork-web-backend-sdk/http"

)

func main() {
    cfg := sdkhttp.NewDefaultConfig("http://localhost:3800")
    client := github.com/sdkwork/sdkwork-web-backend-sdk.NewSdkworkBackendClientWithConfig(cfg)
    client.SetAuthToken("your-auth-token")
client.SetAccessToken("your-access-token")
    
    // Use the SDK
    result, err := client.Nginx.StatusRetrieve()
    if err != nil {
        panic(err)
    }
    fmt.Println(result)
}
```

## Authentication

```text
Authorization: Bearer <authToken>
Access-Token: <accessToken>
```


## Configuration (Non-Auth)

```go
cfg := sdkhttp.NewDefaultConfig("http://localhost:3800")
client := github.com/sdkwork/sdkwork-web-backend-sdk.NewSdkworkBackendClientWithConfig(cfg)

// Set custom headers
client.SetHeader("X-Custom-Header", "value")
```

## API Modules

- `client.Nginx` - nginx API
- `client.Server` - server API
- `client.Agent` - agent API
- `client.Audit` - audit API

## Usage Examples

### nginx

```go
// 获取 Nginx 状态
result, err := client.Nginx.StatusRetrieve()
if err != nil {
    panic(err)
}
fmt.Println(result)
```

### server

```go
// 获取服务器列表
params := map[string]interface{}{
    "page": 1,
    "pageSize": 2,
}
result, err := client.Server.ServersList(params)
if err != nil {
    panic(err)
}
fmt.Println(result)
```

### agent

```go
// 拉取 nginx 配置与证书 bundle
params := map[string]interface{}{
    "ifSyncVersion": "ifSyncVersion",
}
result, err := client.Agent.Sync(params)
if err != nil {
    panic(err)
}
fmt.Println(result)
```

### audit

```go
// 获取审计日志列表
params := map[string]interface{}{
    "page": 1,
    "pageSize": 2,
    "targetType": "targetType",
    "action": "action",
    "operatorId": "operatorId",
    "startDate": "startDate",
    "endDate": "endDate",
}
result, err := client.Audit.LogsList(params)
if err != nil {
    panic(err)
}
fmt.Println(result)
```

## Error Handling

```go
_, err := client.Nginx.StatusRetrieve()
if err != nil {
    // Handle error
    fmt.Println("Error:", err)
    return
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

> Set `GO_RELEASE_TAG` (or `SDKWORK_RELEASE_TAG`) and push tag if needed.

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
