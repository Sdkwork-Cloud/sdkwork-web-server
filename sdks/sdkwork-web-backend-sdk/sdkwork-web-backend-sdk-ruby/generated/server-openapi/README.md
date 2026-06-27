# sdkwork-web-backend-sdk (Ruby)

Generated SDKWork v3 dual-token transport SDK.

## Installation

```bash
gem install sdkwork-web-backend-sdk
```

## Quick Start

```ruby
require 'sdkwork/backend_sdk'

config = Sdkwork::BackendSdk::SdkConfig.new(base_url: 'http://localhost:3800')
client = Sdkwork::BackendSdk::SdkworkBackendClient.new(config)
params = { 'page' => 1, 'pageSize' => 2 }
result = client.server.servers_list(params: params)


puts result.inspect
```

## Authentication

```text
Authorization: Bearer <authToken>
Access-Token: <accessToken>
```


## Configuration (Non-Auth)

```ruby
config = Sdkwork::BackendSdk::SdkConfig.new(base_url: 'http://localhost:3800')
client = Sdkwork::BackendSdk::SdkworkBackendClient.new(config)

# Set custom headers
client.set_header('X-Custom-Header', 'value')
```

## API Modules

- `client.nginx` - nginx API
- `client.server` - server API
- `client.agent` - agent API
- `client.audit` - audit API

## Usage Examples

### nginx

```ruby
# 获取 Nginx 状态
result = client.nginx.status_retrieve()
puts result.inspect
```

### server

```ruby
# 获取服务器列表
params = { 'page' => 1, 'pageSize' => 2 }
result = client.server.servers_list(params: params)
puts result.inspect
```

### agent

```ruby
# 拉取 nginx 配置与证书 bundle
params = { 'ifSyncVersion' => 'ifsyncversion' }
result = client.agent.sync(params: params)
puts result.inspect
```

### audit

```ruby
# 获取审计日志列表
params = { 'page' => 1, 'pageSize' => 2, 'targetType' => 'targettype', 'action' => 'action', 'operatorId' => '1', 'startDate' => '2026-04-10T00:00:00Z', 'endDate' => '2026-04-10T00:00:00Z' }
result = client.audit.logs_list(params: params)
puts result.inspect
```

## Error Handling

```ruby
begin
  params = { 'page' => 1, 'pageSize' => 2 }
  client.server.servers_list(params: params)
rescue StandardError => e
  warn("Error: #{e.message}")
end
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

> Configure RubyGems registry credentials before release publish.

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
