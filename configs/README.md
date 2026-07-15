# Runtime Configuration Templates

Application: sdkwork-web
Status: active
Owner: SDKWork maintainers
Specs: CONFIG_SPEC.md, ENVIRONMENT_SPEC.md, RUNTIME_DIRECTORY_SPEC.md, DEPLOYMENT_SPEC.md

## Purpose

This directory contains source-controlled, non-secret configuration templates and development examples. Production runtime configuration is administrator-managed under the canonical SDKWork runtime directories or injected by deployment infrastructure.

## Directory Index

| Path | Purpose |
| --- | --- |
| `topology/` | Safe standalone/cloud environment topology values. |
| `acme/`, `agent/`, `worker/` | Non-secret environment examples for focused processes. |
| `examples/sdkwork.webserver.config.json` | Valid application Web Server configuration example. |
| `sdkwork-api-cloud-gateway.web.*.toml` | Platform gateway integration templates. |

## Forbidden Content

- Production passwords, tokens, database URLs containing credentials, API keys, private keys, or certificate secret material.
- Mutable runtime snapshots, databases, logs, caches, PID files, or node assignments.
- Environment-specific absolute developer machine paths.

The example static asset under `examples/public/` is verification content only. Certificate file references in application configs must resolve through protected runtime paths and are not committed here.

## Verification

```powershell
cargo run -p sdkwork-web-standalone-gateway -- validate configs/examples/sdkwork.webserver.config.json
pnpm check:repository-docs
```

