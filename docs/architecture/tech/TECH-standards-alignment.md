> Migrated from `docs/standards-alignment.md` on 2026-06-24.
> Owner: SDKWork maintainers

# Standards Alignment

SDKWork Web Server standards alignment status for `sdkwork-web-server`.

## Integrated Frameworks

| Framework | Status | Evidence |
| --- | --- | --- |
| `sdkwork-web-framework` | Integrated | `sdkwork-router-webserver-*` web bootstrap, dual-token route manifests, auth context injection |
| `sdkwork-database` | Integrated | `database/` assets, `sdkwork-webserver-database-host`, `pnpm db:*` |
| `sdkwork-utils-rust` | Integrated | `sdkwork-webserver-core` env parsing, repository slugify |
| `sdkwork-discovery` | Deferred | V1 is HTTP-only unified-process; add when split-services RPC is required |

## Implementation Status

| Layer | Status | Notes |
| --- | --- | --- |
| OpenAPI authorities | Complete | App + backend YAML materialized to JSON, route manifests, SDK assembly |
| Service layer | Complete | `WebAppApi` + `WebBackendApi` on `WebService` |
| Repository SQLx | Complete | All `web_*` tables wired via `WebRepositoryPort` |
| HTTP routes | Complete | 22 app + 11 backend operations aligned with OpenAPI paths |
| Runtime bootstrap | Complete | `bootstrap_web_runtime_from_env()` with DB lifecycle + ACME issuer + edge runtime |
| ACME / certificates | Complete | instant-acme (LE), rcgen (self-signed), AES-GCM encrypt, renewal worker |
| Edge runtime | Complete | nginx deploy/validate/reload, cert bundle materialization |
| Edge agent | Complete | heartbeat + conditional sync (`ifSyncVersion`); local state + offline compensation |
| Certificate worker | Complete | `sdkwork-webserver-certificate-worker` scans `autoRenew` + `renewal_status` |
| Server registration | Complete | `servers.create` returns one-time `agentToken`; heartbeat updates status |
| Deployments | Complete | Docker + Kubernetes manifests under `deployments/` |

## Certificate Stack (ADR-20260623)

| Component | Choice | Status |
| --- | --- | --- |
| ACME client | `instant-acme` | Integrated |
| CA (production) | Let's Encrypt | Integrated (HTTP-01, staging default in dev) |
| Self-signed (dev) | `rcgen` (`certType=3`) | Integrated |
| Private key storage | AES-256-GCM (env key; KMS in prod) | Integrated |
| Auto renewal | Worker scan + `renewal_status` state machine | Integrated |
| Cert distribution | Agent sync + stable `sv1:` fingerprint + `ifSyncVersion` conditional pull | Phase 2a (ADR-20260623-cert-distribution-topology) |
| Incremental / push | Per-node delta queue + push notify | Phase 2b |

## Verification

```powershell
pnpm verify
cargo test --workspace
pnpm db:validate
pnpm topology:validate
pnpm api:materialize
node ../sdkwork-specs/tools/check-repository-docs-standard.mjs --root .
```

## Remaining Work (Phase 2+)

- Per-node cert delta queue and push notification (Phase 2b)
- PC app UI implementation after `sdkgen` for `sdks/sdkwork-web-*-sdk` (scaffold under `apps/sdkwork-web-pc/`)
- Client apps: H5 / Flutter management surfaces
- Generate and publish SDK client packages from `sdks/sdkwork-web-*-sdk`
- Production KMS integration for `SDKWORK_WEB_CERT_ENCRYPTION_KEY`
- Per-tenant Let's Encrypt account persistence
- ADR acceptance: nginx-agent, client-architecture

