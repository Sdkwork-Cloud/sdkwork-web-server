# SDKWork Web PC Management Client

Browser/desktop management surface for **SDKWork Web Server** (`sdkwork-web`). The backend API server is production-ready; this client root is reserved for the management UI.

## Architecture

Follows `sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md` and [TECH_ARCHITECTURE.md](../../docs/architecture/TECH_ARCHITECTURE.md):

```text
apps/sdkwork-web-pc/
  packages/sdkwork-web-pc-core/           # SDK bootstrap + TokenManager
  packages/sdkwork-web-pc-shell/          # routes, layout, providers
  packages/sdkwork-web-pc-sites/          # app-api: sites
  packages/sdkwork-web-pc-domains/        # app-api: domains
  packages/sdkwork-web-pc-certificates/   # app-api: certificates
  packages/sdkwork-web-pc-deployments/    # app-api: deployments
  packages/sdkwork-web-pc-console-nginx/  # backend-api: nginx configs
  packages/sdkwork-web-pc-console-servers/# backend-api: servers + agents
  packages/sdkwork-web-pc-web/            # Vite browser host
```

## Prerequisites

1. Generate TypeScript SDK from `sdks/sdkwork-web-app-sdk` and `sdks/sdkwork-web-backend-sdk` via `sdkwork-sdk-generator` (`sdkgen`). Do not hand-edit generated output.
2. Wire IAM login per `IAM_LOGIN_INTEGRATION_SPEC.md` using `sdkwork.app.config.json` backend profile.

## Status

| Package | Status |
| --- | --- |
| Component specs | Defined (`specs/component.spec.json` per package) |
| Generated SDK consumers | Pending SDK package publish |
| UI implementation | Pending |

## Commands

```powershell
pnpm verify
```
