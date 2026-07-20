# sdkwork-web-app-sdk

Generated SDKWork v3 dual-token transport SDK.

## Installation

```bash
npm install @sdkwork/web-app-sdk
# or
yarn add @sdkwork/web-app-sdk
# or
pnpm add @sdkwork/web-app-sdk
```

## Quick Start

```typescript
import { SdkworkAppClient } from '@sdkwork/web-app-sdk';

const client = new SdkworkAppClient({
  baseUrl: 'http://localhost:3800',
  timeout: 30000,
});

// Authentication
client.setAuthToken('your-auth-token');
client.setAccessToken('your-access-token');

// Use the SDK
const params = {
  page: 1,
  page_size: 2,
};
const result = await client.certificate.list(params);
```

## Authentication

```text
Authorization: Bearer <authToken>
Access-Token: <accessToken>
```


## Configuration (Non-Auth)

```typescript
import { SdkworkAppClient } from '@sdkwork/web-app-sdk';

const client = new SdkworkAppClient({
  baseUrl: 'http://localhost:3800',
  timeout: 30000, // Request timeout in ms
  headers: {      // Custom headers
    'X-Custom-Header': 'value',
  },
});
```

## API Modules

- `client.site` - site API
- `client.domain` - domain API
- `client.deployment` - deployment API
- `client.envVariable` - env_variable API
- `client.certificate` - certificate API
- `client.monitor` - monitor API

## Usage Examples

### site

```typescript
// 获取站点列表
const params = {
  page: 1,
  page_size: 2,
  status: 0,
  siteType: 1,
  keyword: 'keyword',
};
const result = await client.site.list(params);
```

### domain

```typescript
// 获取站点域名列表
const siteId = '1';
const params = {
  page: 1,
  page_size: 2,
};
const result = await client.domain.sites.domains.list(siteId, params);
```

### deployment

```typescript
// 获取部署历史
const siteId = '1';
const params = {
  page: 1,
  page_size: 2,
  status: 0,
};
const result = await client.deployment.sites.deployments.list(siteId, params);
```

### env_variable

```typescript
// 获取环境变量列表
const siteId = '1';
const params = {
  environment: 'environment',
};
const result = await client.envVariable.sites.envVariables.list(siteId, params);
```

### certificate

```typescript
// 获取证书列表
const params = {
  page: 1,
  page_size: 2,
};
const result = await client.certificate.list(params);
```

### monitor

```typescript
// 获取健康检查配置
const siteId = '1';
const result = await client.monitor.sites.healthChecks.list(siteId);
```

## Error Handling

```typescript
import { SdkworkAppClient, NetworkError, TimeoutError, AuthenticationError } from '@sdkwork/web-app-sdk';

try {
  const params = {
    page: 1,
    page_size: 2,
  };
  const result = await client.certificate.list(params);
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
