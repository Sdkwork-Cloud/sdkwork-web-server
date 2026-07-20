# SDKWork Standards Alignment

Application: `sdkwork-web-server`

Updated: 2026-07-20

This document records current implementation and evidence. It does not declare production release
approval. Normative requirements are owned by `../sdkwork-specs`.

## Framework And Capability Matrix

| Capability | Current state | Evidence |
| --- | --- | --- |
| `sdkwork-web-framework` | Integrated | App/backend route manifests, `WebRequestContext`, IAM resolver, framework `service_router`, health/readiness/metrics |
| `sdkwork-database` | Integrated | Database manifest and baselines, lifecycle host, one process-shared typed PostgreSQL/SQLite pool, repository parity tests |
| `sdkwork-utils-rust` | Integrated | API envelopes, pagination, crypto, SHA-256, validation, serde helpers, platform helpers |
| `sdkwork-id-core` | Integrated through database ID support | Snowflake internal IDs and UUID resource identities |
| Backend SDK | Integrated | Generated TypeScript/Rust family, AgentToken auth, bounded response reads, Node Daemon consumption without handwritten HTTP |
| `sdkwork-drive` | Gated, no current upload capability | No business upload/presign/provider ownership; contract test rejects future bypasses |
| `sdkwork-discovery` | Gated, no current RPC transport | No tonic/prost service; contract test requires RPC framework and discovery together if RPC is introduced |

## Architecture State

- OpenAPI YAML is authored under `apis/`; materialized JSON, route manifests, and generated SDK
  inputs are deterministic derivatives.
- App and backend operations use the SDKWork v3 success envelopes and Problem Details error shape.
- The standalone gateway composes framework management routes with the bounded HTTP/HTTPS data
  plane. Management routes call services through ports; SQLx stays in repository modules.
- Database bootstrap returns one SDKWork lifecycle-owned typed pool. PostgreSQL and SQLite compile
  the same repository implementation; production source contains no `AnyPool` bridge or second
  pool.
- The Web Node Daemon uses the application-root generated Rust backend SDK for heartbeat and sync,
  with typed AgentToken configuration, canonical envelope decoding, and finite response limits.
- Proxy orchestration, upstream selection/health, request-body controls, metrics, TLS, DNS, admission,
  and protocol guards are separated into focused private modules.

## API And SDK Guarantees

- Authored Agent routes explicitly declare `x-sdkwork-route-auth: agent-token` and require the
  `AgentToken` security scheme.
- Generated SDK methods return domain payloads after SDKWork v3 envelope unwrapping and reject
  nonzero business codes.
- Backend SDK generation defaults to TypeScript and Rust, retains generator control-plane manifests,
  removes stale owned files, and is idempotent on an unchanged contract.
- `sdk-manifest.json` and `specs/component.spec.json` agree on IAM SDK dependencies.
- Generated files under `generated/server-openapi` are generator-owned and are never hand-edited.

## Deployment And Release State

`standalone.production` is a host-package profile. `cloud.production` is a Kubernetes/container-image
profile with digest-bound templates, a bounded migration Job, StatefulSet identity, probes,
PodDisruptionBudget, non-root execution, read-only root filesystem, dropped capabilities, and
secret-manager references.

The four Linux server package declarations in `sdkwork.app.config.json` are disabled and carry
`releaseBuildDeferred: true`. Archive packaging, checksum, Sigstore, CycloneDX, x64/arm64 smoke,
database recovery, and HA workflow steps are implemented, but no container registry publication
authority or production release approval is declared. Docker/Kubernetes files are deployment
templates, not evidence that an image has been published or deployed.

## Verification

Primary local gates:

```powershell
pnpm check
pnpm verify
pnpm db:validate
pnpm topology:validate
node ..\sdkwork-specs\tools\deployctl.mjs validate --root . --profile cloud.production
node ..\sdkwork-specs\tools\deployctl.mjs validate --root . --profile standalone.production
node ..\sdkwork-github-workflow\scripts\sdkwork-workflow.mjs validate --config sdkwork.workflow.json
```

PostgreSQL lifecycle, recovery, and failover tests require Docker or an explicitly disposable
PostgreSQL endpoint. A local run that lacks that external authority must report the missing evidence;
it must not convert an ignored or unavailable PostgreSQL test into a production-readiness claim.
