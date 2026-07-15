# Rust Request Data-Plane Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-15
Requirement: REQ-2026-0003
Decision: ADR-20260715-rust-webserver-data-plane
Specs: RUST_CODE_SPEC.md, CONFIG_SPEC.md, SECURITY_SPEC.md, DEPLOYMENT_SPEC.md, NGINX_SPEC.md, TEST_SPEC.md

## 1. Runtime Boundaries

```text
authored app config + host runtime config + resolved secrets
                         |
                         v
sdkwork-webserver-core: schema/model/semantic validation/compiler
                         |
                         v
                 immutable compiled app
                         |
          +--------------+--------------+
          |                             |
          v                             v
HTTP/HTTPS listeners            management listener
static/proxy/routes             app-api/backend-api
no database hot path            service/repository/database
```

The request data plane and management plane may share one packaged binary in the standalone profile, but they do not share bootstrap requirements. A data-plane-only operation can start from a verified local app configuration while the management database is unavailable.

## 2. Configuration Flow

1. Read the bounded configuration file from an explicit host runtime setting.
2. Validate the JSON document against `specs/sdkwork.webserver.config.schema.json`.
3. Deserialize with Serde models that reject unknown fields.
4. Run semantic validation for ids, references, listener conflicts, host ownership, route precedence, paths, TLS, upstreams, and budgets.
5. Canonicalize and compile immutable host/route indexes.
6. Resolve protected file and secret references at listener bootstrap, never while routing a request.
7. Expose the compiled revision to request handlers through `Arc` without in-place mutation.

Dynamic publication later builds a full candidate and swaps one immutable `Arc` generation. It does not mutate live route collections.

## 3. Crate Responsibilities

| Crate | Added responsibility | Forbidden responsibility |
| --- | --- | --- |
| `sdkwork-webserver-core` | Config types, file loading, semantic errors, normalized domains/paths, route matching, compiled indexes, hard limit validation. | Axum handlers, sockets, TLS I/O, SQLx, management APIs, process control. |
| `sdkwork-web-standalone-gateway` | Operation dispatch, HTTP/HTTPS binds, static service adapters, proxy transport, request limits, graceful shutdown, management/data-plane composition. | Business rules, SQL queries, generated SDK ownership, raw credential parsing. |
| `sdkwork-webserver-edge-runtime` | Existing external Nginx artifact validation/materialization until renamed or superseded by a later reviewed boundary. | Rust request-path serving. |

The existing `sdkwork-webserver-edge-runtime` name predates the current naming standard. This requirement does not expand it; a separate migration must choose a responsibility-specific replacement without breaking current consumers.

## 4. Request Flow

1. Listener accepts within connection and handshake budgets.
2. Rustls negotiates TLS/ALPN for HTTPS listeners.
3. Hyper/Axum parses HTTP under configured header/body/time limits.
4. The gateway normalizes authority and path once.
5. The compiled core selects listener, virtual host, and route deterministically.
6. The action adapter serves a fixed response, redirect, static resource, or reverse proxy.
7. Backpressure and cancellation flow through the body stream.
8. Bounded structured telemetry records result, duration, bytes, and selected ids without secrets.

The foundation rejects unsupported regex/Nginx constructs during semantic validation. It never silently falls back to approximate behavior.

## 5. HTTP And TLS Stack

- Axum/Hyper provide HTTP/1.1 and HTTP/2 server behavior.
- `axum-server` integrates Tokio listeners and Rustls TLS configuration.
- TLS files are protected runtime references; key bytes are not represented in the app config model or logs.
- Rustls defaults are constrained to TLS 1.2/1.3. HTTP/2 is negotiated through ALPN.
- `tower-http` static services provide established conditional/range/file behavior and operate below an approved root.
- Reqwest/Rustls provides upstream HTTP/HTTPS pooling and streaming. Redirect following is disabled for proxy transport.

Advanced HTTP/1 parser hardening, HTTP/2 abuse limits, client source accounting, and dynamic certificate maps require later focused requirements even though the selected libraries provide the protocol foundation.

## 6. Concurrency And Memory

- Configuration and route indexes are immutable after compile.
- Request handlers do not hold locks across `.await`.
- Proxy bodies are converted to streams; no `to_bytes`/full collect is permitted on the proxy path.
- Static files use the established async service and its bounded OS/file behavior.
- Connection, body, header, timeout, route, host, upstream, target, and config byte limits are validated before serving.
- Listener tasks are supervised. A bind or TLS bootstrap failure prevents readiness and terminates the requested data-plane operation.
- Shutdown uses one cancellation signal and a finite drain deadline; no detached listener task remains after process exit.

The foundation establishes bounded behavior but does not yet satisfy the parent 100,000-connection or 24-hour soak targets until dedicated load and memory evidence exists.

## 7. Operation Modes

| Mode | Database | Behavior |
| --- | --- | --- |
| Default management mode | Required by current control plane | Existing app-api/backend-api and service health behavior. |
| `db-migrate` | Required | Existing database migration-only behavior. |
| `validate` | Not used | Validate and compile one Web Server app configuration, print redacted summary, exit non-zero on any blocker. |
| `data-plane` | Not used | Start only configured HTTP/HTTPS application listeners and drain on shutdown. |
| Future combined mode | Management optional after startup policy | Start isolated management and request listeners with separate readiness and failure policies. |

## 8. Implementation Status

| Capability | Status at document update |
| --- | --- |
| Product PRD and runtime requirements | Defined |
| Architecture decision | Accepted |
| Machine configuration schema | In progress |
| Core config model/compiler | In progress |
| HTTP listener and fixed/redirect routes | Planned in REQ-2026-0003 |
| Static and streaming proxy routes | Planned in REQ-2026-0003 |
| HTTPS listener | Planned in REQ-2026-0003 |
| Dynamic reload, complete Nginx profile, cache, cluster rollout | Not implemented; later requirements |

No planned row may be reported as implemented until its verification evidence passes.

