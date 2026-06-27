# sdkwork-web-backend-sdk

Generated SDKWork v3 dual-token transport SDK.

## Installation

```bash
npm install @sdkwork/web-backend-sdk
# or
yarn add @sdkwork/web-backend-sdk
# or
pnpm add @sdkwork/web-backend-sdk
```

## Quick Start

```typescript
import { SdkworkBackendClient } from '@sdkwork/web-backend-sdk';

const client = new SdkworkBackendClient({
  baseUrl: 'http://localhost:3800',
  timeout: 30000,
});

// Authentication
client.setAuthToken('your-auth-token');
client.setAccessToken('your-access-token');

// Use the SDK
const result = await client.nginx.status.retrieve();
```

## Authentication

```text
Authorization: Bearer <authToken>
Access-Token: <accessToken>
```


## Configuration (Non-Auth)

```typescript
import { SdkworkBackendClient } from '@sdkwork/web-backend-sdk';

const client = new SdkworkBackendClient({
  baseUrl: 'http://localhost:3800',
  timeout: 30000, // Request timeout in ms
  headers: {      // Custom headers
    'X-Custom-Header': 'value',
  },
});
```

## API Modules

- `client.nginx` - nginx API
- `client.server` - server API
- `client.agent` - agent API
- `client.audit` - audit API

## Usage Examples

### nginx

```typescript
// 获取 Nginx 状态
const result = await client.nginx.status.retrieve();
```

### server

```typescript
// 获取服务器列表
const params = {
  page: 1,
  pageSize: 2,
};
const result = await client.server.list(params);
```

### agent

```typescript
// 拉取 nginx 配置与证书 bundle
const params = {
  ifSyncVersion: 'ifSyncVersion',
};
const result = await client.agent.sync(params);
```

### audit

```typescript
// 获取审计日志列表
const params = {
  page: 1,
  pageSize: 2,
  targetType: 'targetType',
  action: 'action',
  operatorId: 'operatorId',
  startDate: 'startDate',
  endDate: 'endDate',
};
const result = await client.audit.auditLogs.list(params);
```

## Error Handling

```typescript
import { SdkworkBackendClient, NetworkError, TimeoutError, AuthenticationError } from '@sdkwork/web-backend-sdk';

try {
  const result = await client.nginx.status.retrieve();
} catch (error) {
  if (error instanceof AuthenticationError) {
    console.error('Authentication failed:', error.message);
  } else if (error instanceof TimeoutError) {
    console.error('Request timed out:', error.message);
  } else if (error instanceof NetworkError) {
    console.error('Network error:', error.message);
  } else {
    throw error;
  }
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

> Configure npm registry credentials before release publish.

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
