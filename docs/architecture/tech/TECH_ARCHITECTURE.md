# SDKWork Web Server Technical Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-15
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md, RUST_CODE_SPEC.md, WEB_FRAMEWORK_SPEC.md, WEB_BACKEND_SPEC.md, DATABASE_FRAMEWORK_SPEC.md, CONFIG_SPEC.md, SECURITY_SPEC.md, DEPLOYMENT_SPEC.md, NGINX_SPEC.md

## Document Map

- [TECH-runtime-data-plane.md](TECH-runtime-data-plane.md) - target and implementation status for the Rust HTTP/HTTPS request data plane.
- [TECH-standards-alignment.md](TECH-standards-alignment.md) - pointer to the repository standards-alignment matrix.
- [ADR-20260715-rust-webserver-data-plane.md](../decisions/ADR-20260715-rust-webserver-data-plane.md) - accepted data-plane component and technology decision.
- [PRD.md](../../product/prd/PRD.md) - product behavior and commercial release authority.

## 1. Architecture Overview

SDKWork Web Server is evolving from an HTTP management control plane into a Rust-native HTTP/HTTPS Web Server with separate request, management, and host-operations planes.

Current implemented baseline:

- app-api and backend-api management surfaces;
- site, domain, deployment, certificate, Nginx, health-check, audit, environment, and agent business workflows;
- SQLx persistence through the SDKWork database framework;
- ACME/self-signed certificate services;
- external Nginx artifact materialization and edge-agent synchronization;
- one standalone Axum management listener.

Target work in progress under `REQ-2026-0003`:

- machine-validated application Web Server configuration;
- independent Rust HTTP/HTTPS request listeners;
- virtual hosts and deterministic route matching;
- static resources, fixed responses, redirects, and streaming reverse proxy;
- data-plane-only bootstrap without database initialization;
- bounded resources and graceful process lifecycle.

The target rows are not implementation-complete until the linked requirement evidence passes.

## 2. Technology Choices

| Layer | Choice | Status |
| --- | --- | --- |
| Language/runtime | Rust 2021 + Tokio | Implemented |
| Management HTTP | Axum through `sdkwork-web-framework` | Implemented |
| Request HTTP | Axum/Hyper | In progress |
| Request TLS | `axum-server` + Rustls | In progress |
| Static content | `tower-http` file services behind compiled routing | In progress |
| Reverse proxy transport | Reqwest/Hyper with Rustls and streamed bodies | In progress |
| App Web Server config | JSON Schema authority + Serde + semantic compiler | In progress |
| Database | `sdkwork-database` + SQLx; PostgreSQL default, explicit single-node SQLite profile | Implemented in control plane, parity evidence incomplete |
| IAM | `sdkwork-iam-web-adapter` for protected management surfaces | Implemented |
| Certificates | `instant-acme`, `rcgen`, target Rustls activation | Issuance implemented; request-plane activation in progress |

## 3. System Boundaries

```text
sdkwork-web-standalone-gateway
  |-- management bootstrap -> app/backend route crates -> service -> repository -> database
  |-- data-plane bootstrap -> compiled Web Server config -> HTTP/HTTPS/static/proxy
  `-- host operations -> config, signals, readiness, drain, runtime paths

sdkwork-webserver-core
  `-- framework-independent environment and Web Server config/compiler logic

sdkwork-webserver-edge-runtime
  `-- existing external Nginx artifact operations only
```

The request path does not call management services or repositories. Management route crates continue to use `sdkwork-web-framework`; application traffic routes are configuration-owned Web Server behavior and do not create a second SDKWork business API authority.

## 4. Configuration And Contract Ownership

- `sdkwork.app.config.json` remains application identity and release authority.
- `specs/sdkwork.webserver.config.schema.json` is the local machine contract for app Web Server configuration until coordinated App Manifest standard changes approve `runtime.webServer.configRef`.
- Host process configuration follows `CONFIG_SPEC.md` and `RUNTIME_DIRECTORY_SPEC.md`.
- Published app/node snapshots are a later immutable distribution contract and are not represented by mutable database DTOs on the request path.
- OpenAPI remains authority for management app-api/backend-api only.

## 5. API, SDK, And Data Ownership

- Management success/error responses follow SDKWork envelopes and Problem Details.
- Existing SDK families remain `sdkwork-web-app-sdk` and `sdkwork-web-backend-sdk`.
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

- `standalone`: one packaged gateway can run management, data-plane-only, or future combined modes; server-grade deployments default to PostgreSQL, with an explicit SQLite single-node exception.
- `cloud`: request data-plane nodes consume node-scoped immutable configuration and secret assignments; PostgreSQL remains control-plane authority.
- The current topology manifest describes only HTTP management ingress and must be versioned before it advertises HTTPS/data-plane surfaces.
- Nginx can remain in front during migration but is not required by the target Rust request data plane.

## 8. Architecture Decision Index

| ADR | Topic | Status |
| --- | --- | --- |
| ADR-20260716-canonical-uri-dual-representation | Raw request URI preservation and bounded canonical routing Path | proposed; human review required |
| ADR-20260715-rust-webserver-data-plane | Config authority, crate boundaries, HTTP/TLS/static/proxy stack | accepted / implementation in progress |
| ADR-20260623-acme-certificate-authority | ACME client, CA selection, key storage | historical accepted; requires review against HTTPS PRD |
| ADR-20260623-cert-distribution-topology | Agent sync and certificate distribution | historical accepted; requires review against node-scoped distribution PRD |

## 9. Verification

```powershell
cargo fmt -- --check
cargo test --workspace
node ..\sdkwork-specs\tools\check-application-layering.mjs --root .
node ..\sdkwork-specs\tools\check-rust-backend-composition.mjs --root .
pnpm check
```

Commercial completion additionally requires the protocol, Nginx, HTTPS, performance, OOM, soak, failure, upgrade, backup/restore, and cluster evidence named by the PRD.
