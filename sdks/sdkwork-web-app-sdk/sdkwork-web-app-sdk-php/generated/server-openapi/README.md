# sdkwork-web-app-sdk (PHP)

Generated SDKWork v3 dual-token transport SDK.

## Installation

```bash
composer require sdkwork/web-app-sdk
```

## Quick Start

```php
<?php

use SDKWork\Web\AppSdk\SdkworkAppClient;
use SDKWork\Web\AppSdk\SdkConfig;


$config = new SdkConfig(baseUrl: 'http://localhost:3800');
$client = new SdkworkAppClient($config);
$$params = ['page' => 1, 'pageSize' => 2];
$result = $client->certificate->certificatesList($params);


var_dump($result);
```

## Authentication

```text
Authorization: Bearer <authToken>
Access-Token: <accessToken>
```


## Configuration (Non-Auth)

```php
<?php

use SDKWork\Web\AppSdk\SdkworkAppClient;
use SDKWork\Web\AppSdk\SdkConfig;

$config = new SdkConfig(baseUrl: 'http://localhost:3800');
$client = new SdkworkAppClient($config);

// Set custom headers
$client->setHeader('X-Custom-Header', 'value');
```

## API Modules

- `$client->site` - site API
- `$client->domain` - domain API
- `$client->deployment` - deployment API
- `$client->envVariable` - env_variable API
- `$client->certificate` - certificate API
- `$client->monitor` - monitor API

## Usage Examples

### site

```php
<?php

// 获取站点列表
$params = ['page' => 1, 'pageSize' => 2, 'status' => 0, 'siteType' => 1, 'keyword' => 'keyword'];
$result = $client->site->sitesList($params);
var_dump($result);
```

### domain

```php
<?php

// 获取站点域名列表
$siteId = '1';
$params = ['page' => 1, 'pageSize' => 2];
$result = $client->domain->sitesDomainsList($siteId, $params);
var_dump($result);
```

### deployment

```php
<?php

// 获取部署历史
$siteId = '1';
$params = ['page' => 1, 'pageSize' => 2, 'status' => 0];
$result = $client->deployment->sitesDeploymentsList($siteId, $params);
var_dump($result);
```

### env_variable

```php
<?php

// 获取环境变量列表
$siteId = '1';
$params = ['environment' => 'environment'];
$result = $client->envVariable->sitesEnvVariablesList($siteId, $params);
var_dump($result);
```

### certificate

```php
<?php

// 获取证书列表
$params = ['page' => 1, 'pageSize' => 2];
$result = $client->certificate->certificatesList($params);
var_dump($result);
```

### monitor

```php
<?php

// 获取健康检查配置
$siteId = '1';
$result = $client->monitor->sitesHealthChecksList($siteId);
var_dump($result);
```

## Error Handling

```php
<?php

use SDKWork\Web\AppSdk\SdkworkAppClient;
use SDKWork\Web\AppSdk\SdkConfig;


$config = new SdkConfig(baseUrl: 'http://localhost:3800');
$client = new SdkworkAppClient($config);

try {
    $params = ['page' => 1, 'pageSize' => 2];
    $client->certificate->certificatesList($params);
} catch (\Throwable $e) {
    echo "Error: {$e->getMessage()}\n";
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

> Set `PHP_RELEASE_TAG` (or `SDKWORK_RELEASE_TAG`) for Composer/Packagist tag-based release.

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
