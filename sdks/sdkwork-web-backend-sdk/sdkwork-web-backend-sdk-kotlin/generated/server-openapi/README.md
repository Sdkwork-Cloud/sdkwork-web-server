# sdkwork-web-backend-sdk (Kotlin)

Generated SDKWork v3 dual-token transport SDK.

## Installation

Add to your `build.gradle.kts`:

```kotlin
implementation("com.sdkwork:sdkwork-web-backend-sdk:1.0.0")
```

Or with Gradle Groovy:

```groovy
implementation 'com.sdkwork:sdkwork-web-backend-sdk:1.0.0'
```

## Quick Start

```kotlin
import com.sdkwork.web.backend.sdk.SdkworkBackendClient
import com.sdkwork.web.backend.sdk.*
import com.sdkwork.common.core.SdkConfig
import kotlinx.coroutines.runBlocking

fun main() = runBlocking {
    val config = SdkConfig(baseUrl = "http://localhost:3800")
    val client = SdkworkBackendClient(config)
    client.setAuthToken("your-auth-token")
client.setAccessToken("your-access-token")

    // Use the SDK
    val result = client.nginx.statusRetrieve()
    println(result)
}
```

## Authentication

```text
Authorization: Bearer <authToken>
Access-Token: <accessToken>
```


## Configuration (Non-Auth)

```kotlin
val config = SdkConfig(baseUrl = "http://localhost:3800")
val client = SdkworkBackendClient(config)
```

## API Modules

- `client.nginx` - nginx API
- `client.server` - server API
- `client.agent` - agent API
- `client.audit` - audit API

## Usage Examples

### nginx

```kotlin
// 获取 Nginx 状态
val result = client.nginx.statusRetrieve()
println(result)
```

### server

```kotlin
// 获取服务器列表
val params = linkedMapOf<String, Any>(
    "page" to 1,
    "pageSize" to 2
)
val result = client.server.serversList(params)
println(result)
```

### agent

```kotlin
// 拉取 nginx 配置与证书 bundle
val params = linkedMapOf<String, Any>(
    "ifSyncVersion" to "ifsyncversion"
)
val result = client.agent.sync(params)
println(result)
```

### audit

```kotlin
// 获取审计日志列表
val params = linkedMapOf<String, Any>(
    "page" to 1,
    "pageSize" to 2,
    "targetType" to "targettype",
    "action" to "action",
    "operatorId" to "1",
    "startDate" to "2026-04-10T00:00:00Z",
    "endDate" to "2026-04-10T00:00:00Z"
)
val result = client.audit.logsList(params)
println(result)
```

## Error Handling

```kotlin
import kotlinx.coroutines.runBlocking

fun main() = runBlocking {
    try {
        val result = client.nginx.statusRetrieve()
        println(result)
    } catch (e: Exception) {
        println("Error: ${e.message}")
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

> Configure Gradle publishing credentials and optional `GRADLE_PUBLISH_TASK`.

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
