# SDKWork Web Server PC Management Client

Planned browser management surface for **SDKWork Web Server** (`sdkwork-web-server`). This root currently owns component contracts only and is not a runnable or releasable application surface.

## Architecture

Follows `sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md` and [TECH_ARCHITECTURE.md](../../docs/architecture/tech/TECH_ARCHITECTURE.md):

```text
apps/sdkwork-web-server-pc/
  packages/sdkwork-web-server-pc-core/            # SDK bootstrap and session ownership
  packages/sdkwork-web-server-pc-shell/           # routes, layout, providers
  packages/sdkwork-web-server-pc-sites/           # app API: sites
  packages/sdkwork-web-server-pc-domains/         # app API: domains
  packages/sdkwork-web-server-pc-certificates/    # app API: certificates
  packages/sdkwork-web-server-pc-deployments/     # app API: deployments
  packages/sdkwork-web-server-pc-console-nginx/   # backend API: Nginx configs
  packages/sdkwork-web-server-pc-console-servers/ # backend API: servers and agents
  packages/sdkwork-web-server-pc-web/             # planned Vite browser host
```

## Prerequisites

1. Consume the repository-owned generated TypeScript packages under `sdks/`; do not hand-edit generated output or replace SDK calls with raw HTTP.
2. Implement IAM login per `IAM_LOGIN_INTEGRATION_SPEC.md` using the application manifest identity and runtime configuration from `etc/`.
3. Add authored package manifests and source only when PC implementation is explicitly scheduled; component specs do not declare runtime readiness.

## Status

| Package | Status |
| --- | --- |
| Component contracts | Defined (`specs/component.spec.json` per package) |
| Authored package manifests and source | Not implemented |
| Runnable browser host | Not implemented |
| Release packaging | Disabled until implementation and verification exist |

## Commands

```powershell
pnpm verify
```
