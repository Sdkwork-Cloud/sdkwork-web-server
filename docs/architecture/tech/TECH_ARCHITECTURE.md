# SDKWork Web Server Technical Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-22
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md, RUST_CODE_SPEC.md, WEB_FRAMEWORK_SPEC.md, WEB_BACKEND_SPEC.md, DATABASE_FRAMEWORK_SPEC.md, CONFIG_SPEC.md, SECURITY_SPEC.md, DEPLOYMENT_SPEC.md, NGINX_SPEC.md

## Document Map

- [TECH-cloud-site-delivery-data-plane.md](TECH-cloud-site-delivery-data-plane.md) - in-progress;
  descriptor v1 ingestion and immutable domain/path/Variant/Mount indexes are implemented, while
  generated-SDK Drive/Knowledgebase provider adapters, activation-time Provider validation, and the
  transport-neutral runtime-set delivery executor run through the dedicated
  `sdkwork-web-server-website-delivery-edge-runtime`. Authenticated conditional cloud assignment pull and phased
  node observations now run through the generated Web Internal SDK; Deploy producer integration,
  true upstream SDK streaming, provider-aware cache consistency, TLS snapshot activation, and
  commercial runtime evidence remain open. Node-local dual-slot runtime-set recovery,
  owner-authenticated Drive/Knowledgebase event
  ingress, dual-slot per-stream checkpoints, ordering/gap handling, and generated-SDK reconciliation
  are implemented.

- [TECH-runtime-data-plane.md](TECH-runtime-data-plane.md) - target and implementation status for the Rust HTTP/HTTPS request data plane.
- [TECH-standards-alignment.md](TECH-standards-alignment.md) - pointer to the repository standards-alignment matrix.
- [ADR-20260715-rust-webserver-data-plane.md](../decisions/ADR-20260715-rust-webserver-data-plane.md) - accepted data-plane component and technology decision.
- [PRD.md](../../product/prd/PRD.md) - product behavior and commercial release authority.

## 1. Architecture Overview

SDKWork Web Server is a Rust-native HTTP/HTTPS server with separate request, management, and
host-operations planes.

Current implemented baseline:

- app-api and backend-api management surfaces;
- site, domain, deployment, certificate, Nginx, health-check, audit, environment, and Web Node workflows;
- SQLx persistence through the SDKWork database framework;
- ACME/self-signed certificate services;
- external Nginx artifact materialization and Web Node Daemon synchronization;
- durable bounded Web Node Daemon desired/observed apply checkpoints with crash replay;
- generated Rust backend SDK heartbeat/sync transport with AgentToken and bounded responses;
- machine-validated Web Server configuration and deterministic virtual-host/route compilation;
- bounded immutable website runtime-set compilation/activation and generated Drive/Knowledgebase
  Rust Internal SDK adapters behind transport-neutral resource/static/Wiki provider ports;
- immutable provider registry plus a runtime-set delivery executor for STATIC/explicit SPA
  fallback/WIKI outcomes with compiled scope, canonical URL reverse mapping, exact Range evidence,
  force-HTTPS, consumer-owned deadlines, and bounded streams;
- an independent single-tenant website delivery edge runtime that watches a verified runtime-set,
  constructs both generated Provider SDKs with secret-file ingress credentials, validates Provider
  resources before activation, registers both adapters, and maps typed outcomes and incremental
  chunks to public HTTP responses;
- a dedicated tenant-fleet Kubernetes topology whose per-fleet Website Service, Node-specific
  provider-event Services, Pod selectors, NetworkPolicy, and disruption budget cannot select Nodes
  assigned to another tenant scope or deliver a signed callback to the wrong Node;
- a production `cloud` runtime assignment source that uses only the generated Web Internal Rust
  SDK, a protected Web Node token file, strict node/environment/hash validation, conditional pulls,
  durable last-known-good recovery, and resumable `RECEIVED -> VALIDATED -> STAGED -> ACTIVE` or
  terminal `REJECTED` observations;
- a node-local dual-slot website runtime-set recovery store that preserves the highest valid
  generation across restart, rejects scope/hash conflicts, and never substitutes for authenticated
  Deploy distribution;
- a separate loopback provider-event listener with subscription-bound Drive/Knowledgebase HMAC
  verification, nine strict owner event contracts, bounded stream-sharded processing, dual-slot
  checkpoints, gap/conflict uncertainty, and current-runtime Provider reconciliation;
- bounded HTTP/1, HTTP/2, TLS, static, redirect, reverse-proxy, WebSocket, health, retry, admission,
  pressure, DNS, and observability controls;
- standalone and cloud development topology plans plus standalone/cloud production deployment
  templates.

The host synchronization process is named **Web Node Daemon** in all new
runtime and operational surfaces. The canonical packaged/development entry
point is `sdkwork-web-node-daemon`; `sdkwork-web-agent` is retained only as a
v3 compatibility binary. The v3 Agent API and generated DTO names remain wire
compatibility identifiers and are not new product terminology.

Commercial release approval remains separate from implementation. The PRD owns outstanding native
capacity, long-duration soak, managed PostgreSQL/PITR, external image publication, staged rollout,
and production monitoring evidence.

## 2. Technology Choices

| Layer | Choice | Status |
| --- | --- | --- |
| Language/runtime | Rust 2021 + Tokio | Implemented |
| Management HTTP | Axum through `sdkwork-web-framework` | Implemented |
| Request HTTP | Axum/Hyper with explicit HTTP/1 and HTTP/2 guards | Implemented bounded baseline |
| Request TLS | Rustls with bounded certificate material | Implemented bounded baseline |
| Static content | Compiled route/static-file service | Implemented bounded baseline |
| Reverse proxy transport | Hyper/Rustls with streamed bodies and bounded retries | Implemented bounded baseline |
| App Web Server config | JSON Schema authority + Serde + semantic compiler | Implemented |
| Database | `sdkwork-database` + SQLx; PostgreSQL default, explicit single-node SQLite profile | Implemented; parity, recovery, and bounded primary/standby promotion verified; managed HA, client failover, fencing, and PITR remain open |
| IAM | `sdkwork-iam-web-adapter` for protected management surfaces | Implemented |
| Certificates | `instant-acme`, `rcgen`, encrypted persistence, Rustls activation | Implemented bounded baseline |

## 3. System Boundaries

```text
sdkwork-api-web-server-standalone-gateway
  |-- management bootstrap -> app/backend route crates -> service -> repository -> database
  |-- data-plane bootstrap -> compiled Web Server config -> legacy HTTP/HTTPS/static/proxy
  `-- host operations -> config, signals, readiness, drain, runtime paths

sdkwork-web-server-website-delivery-edge-runtime
  `-- management-disabled data-plane library -> host config + cloud assignment or local file
      -> delivery executor -> generated-SDK provider adapters -> public HTTP response

sdkwork-webserver-core
  `-- framework-independent environment and Web Server config/compiler logic

sdkwork-webserver-edge-runtime
  `-- existing external Nginx artifact operations only
```

The request path does not call management services or repositories. Management route crates continue to use `sdkwork-web-framework`; application traffic routes are configuration-owned Web Server behavior and do not create a second SDKWork business API authority.

## 4. Configuration And Contract Ownership

- `sdkwork.app.config.json` remains application identity and release authority.
- `specs/sdkwork.webserver.config.schema.json` is the local machine contract for application Web
  Server configuration; the app manifest remains identity/release metadata rather than runtime
  configuration authority.
- Host process configuration follows `CONFIG_SPEC.md` and `RUNTIME_DIRECTORY_SPEC.md`.
- Node synchronization publishes bounded immutable `sv1:` snapshots through the Agent contract;
  mutable management DTOs do not enter the request path.
- OpenAPI authorities are app-api, backend-api, and the application-ingress Web Internal API. The
  Web Node consumes the generated `sdkwork-web-internal-sdk`; the internal route crate consumes the
  local `WebInternalApi` service port and never its own generated client.

## 5. API, SDK, And Data Ownership

- Management success/error responses follow SDKWork envelopes and Problem Details.
- SDK families are `sdkwork-web-app-sdk`, `sdkwork-web-backend-sdk`, and the machine-to-machine
  `sdkwork-web-internal-sdk` used for runtime assignment publication, retrieval, and observations.
- Request data-plane traffic preserves the configured upstream or static Web protocol; it does not wrap arbitrary application responses in SDKWork management envelopes.
- PostgreSQL is cloud/default server authority. SQLite is limited to explicitly selected single-node standalone behavior.
- List/search repositories and SDKs remain subject to store-level SDKWork pagination.

## 6. Security, Privacy, And Resource Boundaries

- Protected management surfaces use SDKWork IAM and typed request context.
- Public application traffic uses explicit host/route policy and HTTPS requirements from the PRD.
- Private keys and credentials remain references to protected runtime sources and are never serialized into app config or logs.
- Static roots, upstream destinations, trusted proxy networks, headers, bodies, timeouts, connections, queues, and configuration size are validated and bounded.
- Request data-plane telemetry is redacted and low-cardinality.
- No lock may be held across asynchronous external I/O.

## 7. Deployment And Runtime Topology

- `standalone`: one packaged gateway runs the composed management and data plane; server-grade
  deployments default to PostgreSQL, with an explicit SQLite single-node development exception.
- `cloud`: the dedicated website delivery edge-runtime nodes consume node-scoped immutable
  configuration and secret assignments; management assemblies are hosted by the platform cloud
  gateway and the application standalone gateway is not started.
- `cloud.development` starts only the local Web Node Daemon client; application/API/database
  surfaces are explicit remote development URLs.
- `cloud.production` uses digest-bound Kubernetes templates with one Node Secret and recovery PVC
  per rendered edge-runtime StatefulSet plus a compiler-validated, hash-versioned immutable listener ConfigMap;
  trusted-proxy CIDRs are explicit deployment inputs and universal networks are rejected. At least
  two independently rendered Nodes are required for high availability. Published image existence
  is not claimed while release packages remain disabled.
- External Nginx remains an edge activation option and is not required for Rust request handling.

## 8. Architecture Decision Index

- [ADR-20260721 Compiled Website Runtime Descriptor](../decisions/ADR-20260721-compiled-website-runtime-descriptor.md) - proposed cloud data-plane input and authority boundary.

| ADR | Topic | Status |
| --- | --- | --- |
| ADR-20260716-canonical-uri-dual-representation | Raw request URI preservation and bounded canonical routing Path | proposed; human review required |
| ADR-20260715-rust-webserver-data-plane | Config authority, crate boundaries, HTTP/TLS/static/proxy stack | accepted |
| ADR-20260720-process-database-pool | One typed SDKWork lifecycle pool per process | accepted |
| ADR-20260623-acme-certificate-authority | ACME client, CA selection, key storage | accepted |
| ADR-20260623-cert-distribution-topology | Node synchronization and certificate distribution | accepted |

## 9. Verification

```powershell
cargo fmt -- --check
cargo test --workspace
node ..\sdkwork-specs\tools\check-application-layering.mjs --root .
node ..\sdkwork-specs\tools\check-rust-backend-composition.mjs --root .
pnpm check
```

Commercial completion additionally requires the protocol, Nginx, HTTPS, performance, OOM, soak, failure, upgrade, backup/restore, and cluster evidence named by the PRD.
