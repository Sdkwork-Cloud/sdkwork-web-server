# sdkwork-web-backend-sdk (Dart)

Generated SDKWork v3 dual-token transport SDK.

## Installation

```bash
dart pub add sdkwork_web_backend_sdk
```

## Quick Start

```dart
import 'package:sdkwork_web_backend_sdk/sdkwork_web_backend_sdk.dart';

final client = SdkworkBackendClient(
  config: const SdkConfig(
    baseUrl: 'http://localhost:3800',
  ),
);
client.setAuthToken('your-auth-token');
client.setAccessToken('your-access-token');

// Use the SDK
final result = await client.nginx.statusRetrieve();
print(result);
```

## Authentication

```text
Authorization: Bearer <authToken>
Access-Token: <accessToken>
```


## Configuration (Non-Auth)

```dart
final client = SdkworkBackendClient.withBaseUrl(baseUrl: 'http://localhost:3800');
client.setHeader('X-Custom-Header', 'value');
```

## API Modules

- `client.nginx` - nginx API
- `client.server` - server API
- `client.agent` - agent API
- `client.audit` - audit API

## Usage Examples

### nginx

```dart
// 获取 Nginx 状态
final result = await client.nginx.statusRetrieve();
print(result);
```

### server

```dart
// 获取服务器列表
final params = <String, dynamic>{
  'page': 1,
  'pageSize': 2,
};
final result = await client.server.serversList(params);
print(result);
```

### agent

```dart
// 拉取 nginx 配置与证书 bundle
final params = <String, dynamic>{
  'ifSyncVersion': 'ifsyncversion',
};
final result = await client.agent.sync_(params);
print(result);
```

### audit

```dart
// 获取审计日志列表
final params = <String, dynamic>{
  'page': 1,
  'pageSize': 2,
  'targetType': 'targettype',
  'action': 'action',
  'operatorId': '1',
  'startDate': '2026-04-10T00:00:00Z',
  'endDate': '2026-04-10T00:00:00Z',
};
final result = await client.audit.logsList(params);
print(result);
```

## Error Handling

```dart
try {
  final result = await client.nginx.statusRetrieve();
  print(result);
} catch (error) {
  print('Error: $error');
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

> Ensure `dart pub publish --dry-run` passes before release publish.

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
