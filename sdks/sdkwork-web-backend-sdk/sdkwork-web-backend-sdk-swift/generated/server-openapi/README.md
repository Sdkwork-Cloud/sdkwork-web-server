# sdkwork-web-backend-sdk (Swift)

Generated SDKWork v3 dual-token transport SDK.

## Installation

Add to `Package.swift`:

```swift
dependencies: [
    .package(url: "https://github.com/sdkwork/sdkwork-web-backend-sdk", from: "1.0.0")
]
```

## Quick Start

```swift
import BackendSDK
import SDKworkCommon

let config = SdkConfig(baseUrl: "http://localhost:3800")
let client = SdkworkBackendClient(config: config)
client.setAuthToken("your-auth-token")
client.setAccessToken("your-access-token")

// Use the SDK
let result = try await client.nginx.statusRetrieve()
print(result)
```

## Authentication

```text
Authorization: Bearer <authToken>
Access-Token: <accessToken>
```


## Configuration (Non-Auth)

```swift
let config = SdkConfig(baseUrl: "http://localhost:3800")
let client = SdkworkBackendClient(config: config)

// Set custom headers
client.setHeader("X-Custom-Header", value: "value")
```

## API Modules

- `client.nginx` - nginx API
- `client.server` - server API
- `client.agent` - agent API
- `client.audit` - audit API

## Usage Examples

### nginx

```swift
// 获取 Nginx 状态
let result = try await client.nginx.statusRetrieve()
print(result)
```

### server

```swift
// 获取服务器列表
let params: [String: Any] = [
    "page": 1,
    "pageSize": 2
]
let result = try await client.server.serversList(params: params)
print(result)
```

### agent

```swift
// 拉取 nginx 配置与证书 bundle
let params: [String: Any] = [
    "ifSyncVersion": "ifsyncversion"
]
let result = try await client.agent.sync(params: params)
print(result)
```

### audit

```swift
// 获取审计日志列表
let params: [String: Any] = [
    "page": 1,
    "pageSize": 2,
    "targetType": "targettype",
    "action": "action",
    "operatorId": "1",
    "startDate": "2026-04-10T00:00:00Z",
    "endDate": "2026-04-10T00:00:00Z"
]
let result = try await client.audit.logsList(params: params)
print(result)
```

## Error Handling

```swift
do {
    try await client.nginx.statusRetrieve()
} catch {
    print("Error: \(error)")
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

> Set `SWIFT_RELEASE_TAG` (or `SDKWORK_RELEASE_TAG`) for tag-based release.

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
