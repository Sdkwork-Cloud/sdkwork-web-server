# sdkwork-web-backend-sdk (C#)

Generated SDKWork v3 dual-token transport SDK.

## Installation

```bash
dotnet add package SDKWork.Web.BackendSdk
```

Or add to your `.csproj`:

```xml
<PackageReference Include="SDKWork.Web.BackendSdk" Version="1.0.0" />
```

## Quick Start

```csharp
using SDKWork.Web.BackendSdk.Models;
using SDKWork.Web.BackendSdk;
using SDKwork.Common.Core;

var config = new SdkConfig("http://localhost:3800");
var client = new SdkworkBackendClient(config);
client.SetAuthToken("your-auth-token");
client.SetAccessToken("your-access-token");

var result = await client.Nginx.StatusRetrieveAsync();
Console.WriteLine(result);
```

## Authentication

```text
Authorization: Bearer <authToken>
Access-Token: <accessToken>
```


## Configuration (Non-Auth)

```csharp
var config = new SdkConfig("http://localhost:3800");
var client = new SdkworkBackendClient(config);

// Set custom headers
client.SetHeader("X-Custom-Header", "value");
```

## API Modules

- `client.Nginx` - nginx API
- `client.Server` - server API
- `client.Agent` - agent API
- `client.Audit` - audit API

## Usage Examples

### nginx

```csharp
// 获取 Nginx 状态
var result = await client.Nginx.StatusRetrieveAsync();
Console.WriteLine(result);
```

### server

```csharp
// 获取服务器列表
var query = new Dictionary<string, object>
{
    ["page"] = 1,
    ["pageSize"] = 2,
};
var result = await client.Server.ServersListAsync(query);
Console.WriteLine(result);
```

### agent

```csharp
// 拉取 nginx 配置与证书 bundle
var query = new Dictionary<string, object>
{
    ["ifSyncVersion"] = "ifsyncversion",
};
var result = await client.Agent.SyncAsync(query);
Console.WriteLine(result);
```

### audit

```csharp
// 获取审计日志列表
var query = new Dictionary<string, object>
{
    ["page"] = 1,
    ["pageSize"] = 2,
    ["targetType"] = "targettype",
    ["action"] = "action",
    ["operatorId"] = "1",
    ["startDate"] = "2026-04-10T00:00:00Z",
    ["endDate"] = "2026-04-10T00:00:00Z",
};
var result = await client.Audit.LogsListAsync(query);
Console.WriteLine(result);
```

## Error Handling

```csharp
try
{
    await client.Nginx.StatusRetrieveAsync();
}
catch (HttpRequestException ex)
{
    Console.WriteLine($"Error: {ex.Message}");
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

> Configure NuGet registry credentials before release publish.

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
