# Standards Alignment

SDKWork Web Server (`sdkwork-web-server`) alignment status against `sdkwork-specs`.

Updated: 2026-06-29

## Framework Integration

| Framework | Status | Evidence |
| --- | --- | --- |
| `sdkwork-web-framework` | Integrated | `sdkwork-routes-webserver-*` dual-token/agent-token manifests, IAM resolver, `service_router` bootstrap |
| `sdkwork-database` | Integrated | `database/` contract assets, `sdkwork-webserver-database-host`, full `pnpm db:*` lifecycle |
| `sdkwork-utils-rust` | Integrated | API envelope, ProblemDetail, AES-GCM, SHA-256, env parsing, slugify, serde_int64 |
| `sdkwork-id-core` (via `sdkwork-database-id`) | Integrated | Snowflake PKs, UUID v4 resource IDs, prefixed agent tokens (`wagent_`) |
| `sdkwork-discovery` | Not required | HTTP application ingress only; adopt when RPC services and dynamic resolution are introduced |
| `sdkwork-drive` | Not required | No file-upload operations in current API surface; adopt when upload features are added |

## Production Readiness

| Layer | Status | Notes |
| --- | --- | --- |
| OpenAPI authorities | Complete | App + backend YAML, materialized JSON, route manifests, SDK assembly |
| API envelope | Complete | `SdkWorkApiResponse` success + `ProblemDetail` errors on all L2+ routes |
| Service layer | Complete | `WebAppApi` + `WebBackendApi` on `WebService` |
| Repository SQLx | Complete | All `web_*` tables via `WebRepositoryPort` |
| HTTP routes | Complete | 22 app + 13 backend operations aligned with OpenAPI paths |
| Runtime bootstrap | Complete | DB lifecycle, ACME issuer, edge runtime, readiness probe |
| ACME / certificates | Complete | Let's Encrypt + self-signed, AES-GCM key storage, renewal worker |
| Edge runtime | Complete | nginx deploy/validate/reload, cert bundle materialization |
| Edge agent | Complete | heartbeat + conditional sync (`ifSyncVersion`) |
| IAM security | Complete | Production fail-closed; dual-token + agent-token auth modes |
| Deployments | Complete | Docker + Kubernetes under `deployments/` |
| Packaging / CI | Complete | `sdkwork.workflow.json`, `.github/workflows/package.yml` |

## Certificate Stack

| Component | Choice |
| --- | --- |
| ACME client | `instant-acme` |
| Production CA | Let's Encrypt (HTTP-01) |
| Development CA | `rcgen` self-signed (`certType=3`) |
| Private key storage | AES-256-GCM (`SDKWORK_WEB_SECRET_ENCRYPTION_KEY`) |
| Auto renewal | `sdkwork-webserver-certificate-worker` + `renewal_status` state machine |
| Cert distribution | Agent sync + stable `sv1:` fingerprint + `ifSyncVersion` conditional pull |

## Verification

```powershell
pnpm verify
```

`pnpm verify` runs formatting, tests, API materialization, app composition, API envelope, repository docs, topology, database framework, and cloud gateway validation.

## Optional Enhancements (post-launch)

These are not blockers for backend production deployment:

- PC management UI under `apps/sdkwork-web-pc/` (requires published TypeScript SDK packages)
- Per-node certificate delta queue and push notification
- External KMS for certificate encryption keys
- Per-tenant Let's Encrypt account persistence
- `sdkwork-discovery` when RPC split-services topology is adopted
- `sdkwork-drive` when file-upload API operations are introduced
