# Rust Request Data-Plane Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-16
Requirements: REQ-2026-0003, REQ-2026-0005, REQ-2026-0006, REQ-2026-0007, REQ-2026-0008, REQ-2026-0009, REQ-2026-0010, REQ-2026-0011, REQ-2026-0012, REQ-2026-0013, REQ-2026-0014, REQ-2026-0015, REQ-2026-0016, REQ-2026-0017, REQ-2026-0018, REQ-2026-0019, REQ-2026-0020, REQ-2026-0021, REQ-2026-0022
Decisions: ADR-20260715-rust-webserver-data-plane; ADR-20260716-canonical-uri-dual-representation (proposed, human review required)
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
7. Hash the exact bounded source bytes with SHA-256 and expose the compiled revision through one immutable runtime generation.
8. In opt-in Watch mode, detect a changed file, build and validate the entire candidate, construct its upstream clients, and compare restart-only topology.
9. Atomically publish the candidate through `ArcSwap`; invalid or restart-only candidates retain the active generation.

Each request loads one generation before routing and uses that same generation through handler completion. Reload serialization is isolated to a reload-only Tokio mutex and is never acquired on the request path. Old generations are reclaimed when requests that borrowed them finish.

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
4. The gateway preserves raw Path and Query, validates their finite budgets, and produces one bounded canonical Path.
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

The bounded ingress profile configures Hyper directly. HTTP/1 uses explicit parser-buffer bytes, header count, and a Tokio-timer-backed complete-header deadline. HTTP/2 uses explicit concurrent-stream, header-list, pending-reset, local-error-reset, stream send-buffer, fixed flow-control windows, and frame-size limits.

An incremental HTTP/1 Framing Guard wraps the accepted stream after Rustls and before Hyper. It bounds and validates request-line structure, method tokens, visible-ASCII request targets, Header/Trailer field names and values, Chunked input, and Trailers; rejects original-wire TE/CL ambiguity and duplicate lengths; then passes unchanged valid bytes to Hyper. A line buffer cannot grow beyond the smaller applicable total/individual ceiling. One connection-local Atomic counter bounds complete request heads observed by the Guard but not yet submitted to the paired Service; dispatch releases a slot synchronously without retaining a Body or creating a queue. TLS ALPN `h2` streams bypass the Guard and Pipeline counter.

An incremental HTTP/2 Wire Guard wraps decrypted ALPN `h2` streams before Hyper. It validates the client Preface and Frame metadata, applies fixed-window Frame/new-Stream/`RST_STREAM` budgets, and bounds encoded Header Block bytes and `CONTINUATION` count without retaining payloads. Cross-stream Continuation, interleaving, invalid client HEADERS ids, invalid reset shape, and configured Frame oversize fail closed at connection scope. H2 remains the owner of HPACK, flow control, SETTINGS, RFC stream/connection errors, and GOAWAY. The selected Hyper server builder advertises configured concurrent Streams, Frame size, and decoded Header List size; it does not expose HPACK dynamic-table sizing, so H2 0.4.15 retains its pinned 4,096-byte default and the application exposes no fake field.

The same Hyper/H2 connection engine owns Keep-Alive PING and ACK matching. It sends one PING after `http2KeepAliveIntervalMs` without an inbound Frame, including when no Stream is active. Missing ACK at `http2KeepAliveTimeoutMs` invokes H2 `abrupt_shutdown(NO_ERROR)`, flushes GOAWAY, and closes. SDKWork adds no second PING parser or timer task. Responsive application-idle clients remain connected until the independent total connection-age deadline.

The listener owns every accepted Hyper connection Future in a bounded `JoinSet`. The accept loop uses non-queuing process/listener permit acquisition before it creates a task, TLS state, or HTTP state, so accept floods cannot bypass the active task bound. One connection-owned `maxConnectionAgeMs` Timer starts after the Acceptor chain completes and never resets on traffic. Age expiry calls Hyper `graceful_shutdown`, which disables HTTP/1 reuse or emits H2 `GOAWAY(NO_ERROR)`. The Future remains polled for at most `drainTimeoutMs`; expiry drops the complete transport/Acceptor chain and releases its permit. Process shutdown signals the same tasks, stops acceptance, drains them concurrently, aborts remaining tasks at the shared deadline, and joins all task results. No raw GOAWAY injection, socket-error approximation, per-Stream age task, or detached accepted-connection task exists.

The proxy bridge preserves `http_body::Frame` values in both directions. Request Data frames are counted while Trailer frames are checked against the same finite Trailer budget and forbidden-field policy. Reqwest receives the guarded Body directly instead of a data-only stream. Reqwest responses are converted back to an HTTP Body without calling `bytes_stream`, so valid response Trailer frames remain visible to Hyper. Client hop-specific `TE` is removed and a canonical `TE: trailers` is generated for the upstream; validated `Trailer` declarations remain end-to-end metadata. HTTP/1 downstream emission follows Hyper's `TE: trailers` recipient signal. Undeclared cross-protocol Trailer synthesis, complete gRPC behavior, broader HTTP/2 HPACK malformed-input conformance, client source accounting, and live certificate-map rotation require later focused requirements.

An early final upstream response changes the proxy request lifecycle from active to paused before the handler exposes the response downstream. Paused request Bodies return `Pending` without reading another client Frame. Because Hyper's upstream HTTP/1 driver may drop its request Body before resolving the response Future, an incomplete Body transfers its existing inner Body object into one deferred ownership slot. The guarded response owns that slot until Hyper processes response completion or downstream cancellation; response-wrapper Drop cancels the upstream producer and releases the client Body. HTTP/1 then carries explicit `Connection: close`; H2 completes the response before `RST_STREAM(NO_ERROR)` and keeps other Streams usable. This two-phase handoff preserves response status, Headers, Body, and Trailer validation without buffering the request.

The URI layer keeps the original Path and Query for raw proxy fidelity and produces exactly one canonical Path after bounded validation. Canonical route/static identity performs one percent-decoding pass, merges repeated slashes, resolves dot segments, preserves significant trailing slash, and rejects traversal above root, decoded control/NUL/backslash, and invalid UTF-8. Authored route paths are validated directly as canonical Path values and are never percent-decoded again, so decoded `?`, `#`, and `%` remain matchable Path data. A proxy without URI rewriting retains the raw request URI; `stripPrefix` uses the canonical Path and the original Query. This behavior is implemented under REQ-2026-0018 but remains draft until human review accepts proposed ADR-20260716 and its documented Windows Nginx security differences.

The admitted response Body owns a reusable idle Timer and the request permit. Pending producer polls register the Hyper task Waker; meaningful Data/Trailer progress resets the deadline, while empty Data does not. A separate accepted-stream wrapper sits outside TLS and both Wire Guards and times continuously Pending `AsyncWrite` write/flush/shutdown calls. This separation covers both a stalled producer and a slow-reading downstream without collecting response data.

HTTP/1 connection handling enables Hyper write-side half-close so EOF after a complete request does not suppress the response. The Handler accepts only one exact HTTP/1.1 `Expect: 100-continue`; known fixed-length overflow returns `413` before Body polling, other expectations return `417`, and the proxy removes `Expect` after the listener has completed the client negotiation. Transfer-Encoding is accepted only for HTTP/1.1 and remains subject to the original-wire Framing Guard. Hyper service readiness serializes Pipeline dispatch, its parser buffer bounds unread bytes, and `http1MaxPipelineDepth` independently bounds fully read request heads awaiting dispatch. An over-depth connection closes because a parser-level synthetic response could violate existing Pipeline response order.

## 6. Concurrency And Memory

- Configuration and route indexes are immutable after compile.
- The request path performs one lock-free `ArcSwap::load_full`; reload never mutates a live route or upstream collection.
- A candidate is fully compiled and all Reqwest upstream clients are built before the active pointer changes.
- Configuration input is read through a `MAX_CONFIG_BYTES + 1` bounded reader, closing the metadata/read replacement allocation race.
- HTTP/1 line memory starts at no more than the request-line budget and grows only to the smaller applicable total and individual field ceiling; rejected wire bytes are not copied into diagnostics.
- TLS PEM reads are bounded to 1 MiB per certificate or private-key file before Rustls parsing.
- TLS startup validates every leaf certificate's current validity, SAN coverage, and private-key match before building immutable Exact and single-label Wildcard hash indexes; a handshake performs no scan across the policy certificate collection.
- Request handlers do not hold locks across `.await`.
- Proxy request Frames use only atomic lifecycle reads and `AtomicWaker` registration. The early-response deferred-Body slot uses one non-nested standard mutex only for ownership transfer/release; it is never touched per Frame, never held across `.await`, and destructors run after the guard is released.
- One process-wide Semaphore bounds admitted requests across listeners with `try_acquire_owned`, so overload creates no waiter queue. The permit is owned by the response Body until end/error/drop rather than by the shorter Handler Future.
- Each admitted response has one reusable producer-idle Timer; each connection lazily allocates at most one reusable write deadline after the first Pending operation. Neither timeout adds a response queue or payload copy.
- Proxy bodies are polled as bounded Data/Trailer frames; no `to_bytes`, `collect`, or data-only conversion is permitted on the proxy path.
- Non-proxy request bodies are stream-discarded and counted before action execution; fixed, Chunked, and HTTP/2 no-length bodies share one application Body budget without Body-sized collection.
- Static files use the established async service and its bounded OS/file behavior.
- Connection, body, header, timeout, route, host, upstream, target, and config byte limits are validated before serving.
- HTTP/2 concurrent decoded-header, encoded-header, and send-buffer products are each capped at 64 MiB per connection. The Wire Guard holds constant parser state and no payload or per-Stream collection; protocol-limit changes are Restart-only and cannot partially replace a live listener generation.
- Active HTTP/2 decoded Header List and send-buffer products and the connection-level encoded Header Block product are each capped at 1 GiB globally. These configuration ceilings supplement, but do not replace, future RSS/cgroup-aware admission and load/soak evidence.
- The global connection/header-window product is capped at 1 GiB; Chunk Size lines and Trailer collections have separate finite limits.
- Listener tasks are supervised. A bind or TLS bootstrap failure prevents readiness and terminates the requested data-plane operation.
- Listener and accepted-connection tasks are supervised. Shutdown and maximum-age retirement use finite drain deadlines; no detached connection or listener task remains after process exit.

The foundation establishes bounded behavior but does not yet satisfy the parent 100,000-connection or 24-hour soak targets until dedicated load and memory evidence exists.

## 7. Operation Modes

| Mode | Database | Behavior |
| --- | --- | --- |
| Default management mode | Required by current control plane | Existing app-api/backend-api and service health behavior. |
| `db-migrate` | Required | Existing database migration-only behavior. |
| `validate` | Not used | Validate and compile one Web Server app configuration, print redacted summary, exit non-zero on any blocker. |
| `data-plane` | Not used | Start only configured HTTP/HTTPS application listeners, optionally Watch the config when `deployment.reload.mode=watch`, and drain on shutdown. |
| Future combined mode | Management optional after startup policy | Start isolated management and request listeners with separate readiness and failure policies. |

## 8. Implementation Status

| Capability | Status at document update |
| --- | --- |
| Product PRD and bounded runtime foundation requirement | Defined; REQ-2026-0003 accepted |
| Architecture decision | Accepted |
| Machine configuration schema | Implemented and verified for the declared foundation profile |
| Core config model/compiler | Implemented; strict validation and immutable host/route indexes verified |
| HTTP listener and fixed/redirect routes | Implemented and real-socket tested |
| Static and streaming proxy routes | Implemented and real-socket tested; proxy bodies are not fully collected |
| HTTPS listener | Implemented and tested with Rustls TLS, HTTP/2 ALPN, immutable multi-certificate Exact/Wildcard SNI selection, leaf SAN/time/key activation checks, and fail-closed unknown/no-SNI behavior |
| Connection admission and graceful drain | Implemented and tested under connection saturation |
| Same-topology local configuration reload | Implemented with SHA-256 revisions, `ArcSwap`, failed-candidate retention, and concurrent real-socket tests |
| Bounded protocol ingress | Implemented for HTTP/1 header bytes/count/deadline, pre-Hyper original-wire Chunked/Trailer framing, all-action Body limits, and HTTP/2 stream/header/reset/flow-control/send-buffer settings; complete parser and abuse conformance remains pending |
| Bounded proxy Trailer fidelity | Implemented for declared HTTP/1 request/response Trailers and HTTP/2 trailing HEADERS with declaration/frame limits, forbidden-field validation, and no Body collection |
| HTTP/1 connection semantics | Implemented and raw-socket tested for HTTP/1.0 default host/Keep-Alive/ordered Pipeline, strict Expect/Continue, proxy Expect termination, TLS Continue, and complete/truncated half-close; complete differential conformance remains pending |
| HTTP/1 request-field budgets | Implemented in Schema/Core/Wire Guard for request line, method, target, Header/Trailer name and value; plain/TLS raw-socket and Restart-only reload evidence added |
| HTTP/2 abuse and drain budgets | Implemented in Schema/Core and a constant-memory decrypted Wire Guard; real TLS/H2 tests cover SETTINGS, Frame/new-Stream/reset churn, encoded blocks, H2 local-error GOAWAY, connection recovery, graceful drain, and Restart-only reload |
| Process request admission | Implemented as one non-queuing cross-listener Semaphore with response Body lifetime ownership; real HTTP/1 and TLS/H2 tests cover overload, completion, H2 reset cancellation, recovery, and Restart-only reload |
| Response progress timeouts | Implemented separately for meaningful response Body Frame gaps and downstream write/flush/shutdown Pending time; real HTTP/1, slow-reader, TLS/H2, recovery, and Restart-only reload tests added |
| Request Body progress timeouts | Implemented as one all-action zero-copy Body wrapper with distinct first-meaningful-Frame and later progress deadlines; real HTTP/1, proxy, TLS/H2, one-permit recovery, empty-Frame, and Restart-only reload tests added |
| HTTP/1 Keep-Alive idle timeout | Implemented as a protocol-aware connection Stream/Service activity pair; plain/TLS H1 idle, one-connection permit release, upload, long response, Pipeline, H2 bypass, recovery, and Restart-only tests pass |
| URI and Query component budgets | Accepted allocation-free cross-H1/H2 raw/decoded Path, segment, Query/parameter/component and percent-safety checks; real H1/H2/proxy/reload tests pass |
| Canonical URI dual representation | Implemented draft with canonical route/static/rewrite identity and raw no-rewrite proxy fidelity; real Nginx/H1/H2/static/proxy evidence exists, but proposed ADR-20260716 requires human review |
| HTTP/1 Pipeline depth | Accepted with one connection-local pending-head counter, plain/TLS over-depth close, one-connection recovery, H2 bypass, Restart-only reload, and Nginx 1.26.2 difference evidence |
| HTTP/2 Keep-Alive PING | Accepted with Hyper/H2 idle PING, finite ACK timeout, `GOAWAY(NO_ERROR)`, H1 isolation, one-connection recovery, Restart-only reload, and pinned Nginx evidence |
| Proxy early-response request lifecycle | Implemented with two-phase pause/cancel ownership, complete response preservation, HTTP/1 close and upstream-pool eviction, H2 `NO_ERROR` Stream cancellation/reuse, timeout/admission recovery, and pinned Nginx evidence |
| Bounded connection maximum age | Implemented with one connection-owned Timer, Hyper-aware HTTP/1/H2 graceful retirement, finite in-flight drain/cancellation, fresh-connection recovery, and Restart-only Watch tests |
| Bounded upstream DNS and SSRF policy | Implemented with shared asynchronous system-resolver profiles, non-queuing query admission, finite answer/timeout/idle-pool bounds, literal and per-resolution address checks, rebinding rejection, and real default-deny/explicit-local proxy evidence |
| Full repository verification | `pnpm verify` and strict full-workspace Clippy pass |
| Persisted rollback, TLS/listener hot handoff, executable upgrade, complete Nginx profile, cache, cluster rollout | Not implemented; later requirements |

No planned row may be reported as implemented until its verification evidence passes.

## 9. Verified Foundation And Remaining Boundary

The accepted foundation evidence covers configuration compilation, process bootstrap, real HTTP/HTTPS sockets, exact/default host selection, exact/prefix route precedence, fixed/redirect/static/proxy actions, streaming proxy transport, request-size rejection, finite timeouts, non-queuing connection admission, traversal and authority-confusion rejection, TLS 1.2/1.3 policy, HTTP/2 ALPN, finite shutdown drain, and same-topology local configuration reload with failed-candidate retention. Accepted REQ-2026-0006 separately adds static multi-certificate SNI selection and activation-time leaf SAN/time/key checks. Accepted REQ-2026-0007 adds bounded protocol budgets. Accepted REQ-2026-0008 adds safe original-wire Chunked/Trailer framing and all-action Body accounting. Accepted REQ-2026-0009 adds frame-preserving declared HTTP/1 and HTTP/2 request/response Trailer proxying. REQ-2026-0010 adds the focused HTTP/1 connection-state slice and records the first real Nginx 1.26.2 protocol comparison. REQ-2026-0011 adds pre-allocation request-line and individual field ceilings. REQ-2026-0012 adds constant-memory HTTP/2 Frame/Header Block abuse controls and real graceful GOAWAY drain evidence. REQ-2026-0013 adds non-queuing process request admission held through streaming response completion or cancellation. REQ-2026-0014 adds separate response producer-idle and downstream write-stall deadlines. REQ-2026-0015 adds distinct request Body start/progress deadlines, HTTP/1 close behavior, H2 Stream isolation, and proxy timeout classification. REQ-2026-0016 adds protocol-scoped HTTP/1 Keep-Alive idle reaping without misclassifying active uploads, responses, or pipelines. Accepted REQ-2026-0017 adds bounded URI and Query components. REQ-2026-0018 adds the implemented but not yet accepted dual raw/canonical URI behavior. Accepted REQ-2026-0019 adds the HTTP/1 pending Pipeline depth boundary. Accepted REQ-2026-0020 adds H2 PING/ACK failure detection. REQ-2026-0021 adds bounded early-response pause/cancel ownership with complete response preservation and protocol-scoped connection behavior. REQ-2026-0022 adds finite connection lifetime, protocol-aware graceful retirement, and owned connection-task drain. REQ-2026-0023 adds bounded asynchronous system resolution, explicit private-range authorization, per-answer rebinding defense, and finite idle pool lifetime.

This evidence does not establish the parent PRD's commercial release. No claim is made yet for persisted operator rollback, live certificate-map rotation and served-fingerprint convergence, listener socket handoff, executable upgrade, undeclared cross-protocol Trailer synthesis, full gRPC proxy conformance, complete HTTP/1 parser differential/fuzz and timeout conformance, exhaustive HTTP/2 malformed-frame/HPACK CPU/fuzz and Nginx differential conformance, WebSocket/SSE Upgrade, PCRE2 location semantics, Nginx import/render/differential conformance, custom DNS transport or authoritative TTL/cache/stale/CNAME behavior, upstream TLS policy, weighted balancing/health/retry/circuit breaking, cache/compression/rate limiting, production observability, 100,000 concurrent connections, 24-hour soak, chaos/failover, Kubernetes high availability, backup/restore, signed SBOM/provenance, or commercial support operations.
