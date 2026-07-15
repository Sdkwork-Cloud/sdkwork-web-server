# SDKWork Web Server Runtime Core PRD

Status: active
Owner: SDKWork maintainers
Application: sdkwork-web
Updated: 2026-07-15
Parent: [PRD.md](PRD.md)
Specs: NGINX_SPEC.md, SECURITY_SPEC.md, PERFORMANCE_SPEC.md, CONFIG_SPEC.md, TEST_SPEC.md

## 1. Purpose

Define the minimum runtime behavior required for SDKWork Web Server to operate as a complete, modern HTTP Web Server rather than only a management API, configuration renderer, or Nginx controller.

The complete runtime is a Rust data plane that can boot from a verified local snapshot, bind real HTTP/HTTPS listeners, route requests, serve static content, reverse proxy upstreams, enforce policy, remain bounded under hostile load, reload without corrupting active traffic, and continue serving the last verified configuration during control-plane or database outages.

## 2. Definition Of A Complete V1 HTTP Web Server

V1 is complete only when one packaged runtime can perform all P0 capabilities without requiring an external Nginx process:

| Capability | V1 level | Completion condition |
| --- | --- | --- |
| HTTP/1.0 compatibility and HTTP/1.1 | P0 | Strict parsing, framing, keep-alive, pipelining-safe response order, streaming, timeouts, and conformance tests. |
| HTTP/2 | P0 | TLS ALPN, bounded HPACK and streams, flow control, GOAWAY drain, abuse defenses, and conformance tests. |
| HTTPS | P0 | TLS 1.2/1.3, SNI, valid certificate selection, atomic rotation, and served-handshake proof. |
| Virtual hosts and routes | P0 | Deterministic Nginx-profile server and location selection. |
| Static Web content | P0 | Safe files, indexes, SPA fallback, conditional requests, ranges, MIME, compression variants, and cache policy. |
| Reverse proxy | P0 | Streaming HTTP upstreams, WebSocket, SSE, DNS, pools, deadlines, retries, health-aware balancing, and upstream TLS. |
| Runtime lifecycle | P0 | Validate, start, readiness, reload, drain, stop, status, local recovery, and service/container integration. |
| Resource governance | P0 | Hierarchical bounded memory, connections, descriptors, queues, buffers, disk, CPU-expensive work, and overload shedding. |
| Operations | P0 | Structured logs, metrics, trace correlation, health/readiness, audit for changes, and support diagnostics. |
| Proxy cache and compression | P1 release gate | Correct eligibility, bounded storage, revalidation, stampede defense, poisoning defense, gzip/Brotli, and purge authorization. |
| gRPC reverse proxy | P1 | HTTP/2 request/response streaming, trailers, deadlines, cancellation, and health-aware upstream behavior. |
| HTTP/3, generic TCP/UDP, WASM/WAF | Future | Separate requirements, threat models, architecture decisions, and release gates. |

A runtime that only exposes app-api/backend-api, writes Nginx files, changes database status, or starts an Axum management listener does not satisfy this definition.

## 3. Runtime Plane Boundaries

The product has three explicit planes:

| Plane | Owns | Must not own |
| --- | --- | --- |
| Request data plane | Accept, TLS, HTTP parsing, routing, static files, proxying, policies, local metrics, and active immutable snapshot. | Synchronous database/control-plane calls required for every request. |
| Configuration and control plane | Authoring, validation orchestration, revisions, deployment, certificate lifecycle, node assignment, audit, and management APIs. | Mutable request-path routing state that bypasses snapshot activation. |
| Host operations plane | Process config, physical sockets, service account, runtime paths, resource ceilings, admin exposure, supervisor/orchestrator integration, and local recovery. | App-owned domains, routes, content, or tenant secrets outside node assignments. |

An application declares logical traffic intent. Host policy determines physical exposure and global ceilings. The compiler produces a canonical, signed, checksummed, immutable node snapshot. The data plane reads the snapshot through typed in-process structures and never executes authored JSON, Nginx text, database rows, or template expressions on the request hot path.

## 4. Bootstrap And Local Recovery

Startup stages are deterministic:

1. Resolve typed host runtime configuration and canonical runtime directories.
2. Validate service account, file ownership, secret providers, clocks, descriptors, memory ceilings, and required platform capabilities.
3. Load the requested immutable snapshot or the last verified local snapshot according to startup policy.
4. Verify schema version, signature, checksum, host assignment, resource references, certificate material, and compatibility profile.
5. Compile or map request-path indexes away from listener threads.
6. Bind and configure physical sockets without exposing traffic.
7. Build TLS, routing, upstream, cache, policy, logging, and metrics state completely.
8. Start accepts, prove listener behavior through local probes, and only then become ready.

Production must not silently start with a generated default site, self-signed certificate, empty route table, permissive proxy, or incomplete snapshot. If no acceptable snapshot is available, affected public listeners remain closed and diagnostics identify the exact blocker.

The last verified snapshot and assigned encrypted resources are retained locally within a bounded rollback policy. A temporary control-plane, PostgreSQL, SQLite, Redis, ACME, or DNS control API outage must not interrupt already active configuration.

## 5. HTTP/1.x Protocol Correctness

The runtime supports HTTP/1.0 compatibility and HTTP/1.1 with strict, incremental parsing:

- Request line, method token, request-target form, version, header names, header values, and line endings are validated before routing.
- Origin-form is the default for origin service. Absolute-form is accepted only under an explicit reverse-proxy compatibility policy. Authority-form/`CONNECT` is rejected unless a future reviewed tunnel capability enables it.
- `Host` is required and validated for HTTP/1.1. Duplicate or conflicting host authority is rejected.
- Conflicting `Content-Length`, invalid duplicate length, ambiguous `Transfer-Encoding`, `Transfer-Encoding` plus `Content-Length`, obsolete line folding, control characters, whitespace ambiguity, and malformed chunking fail closed to prevent request smuggling.
- Chunked bodies, trailers, `Expect: 100-continue`, keep-alive, close semantics, and HTTP/1.0 connection compatibility follow declared protocol behavior.
- Pipelined requests, when accepted, preserve response order and cannot create unbounded queued responses. The server may stop reading or reject additional pipeline depth at its configured bound.
- Method handling preserves registered and extension method tokens for proxy routes while static/managed routes return deterministic `405` and `Allow` behavior where applicable.
- `HEAD`, informational responses, `204`, `304`, and responses to `CONNECT` obey no-body and framing rules.

Parser limits apply before allocation: request-line bytes, header count, total header bytes, individual name/value bytes, chunk metadata, trailer count/bytes, URI bytes, and pipeline depth. Header, body-start, body-progress, total-request, keep-alive, and response-write timeouts protect against slow-client attacks.

## 6. HTTP/2 Protocol Correctness

HTTP/2 support includes:

- TLS ALPN `h2`; cleartext h2c is disabled on public production listeners and requires an explicit private profile.
- Valid connection preface, SETTINGS, stream states, frame sizes, pseudo-header ordering, authority, content length, and forbidden connection-specific headers.
- Bounded HPACK dynamic table, header list size, concurrent streams, pending resets, control frames, outbound queue, and per-connection/per-stream buffers.
- Bidirectional flow control tied to application and upstream consumption so a slow peer cannot force unbounded buffering.
- Priority input is bounded and may be normalized to an implementation-safe scheduling policy without starvation.
- PING, graceful GOAWAY, stream cancellation, trailers, extended `CONNECT` only for supported upgrades, and deterministic shutdown behavior.
- Defenses for rapid reset, continuation floods, empty-frame floods, oversized header compression work, stream churn, and connection-level CPU amplification.

HTTP/2 errors use the correct stream or connection scope. One malformed stream does not terminate unrelated streams unless the protocol requires a connection error. Metrics use bounded reason classifications.

## 7. Request And Response Semantics

- URI normalization, percent decoding, dot-segment handling, query preservation, route matching, filesystem mapping, and upstream URI construction are distinct phases.
- No phase decodes data twice. Encoded separators, invalid UTF-8 policy, NUL, traversal, and platform-specific separator forms are covered by negative tests.
- Request bodies are streamed with backpressure. In-memory aggregation is available only for explicitly bounded managed handlers.
- Cancellation propagates when clients disconnect, deadlines expire, routes are superseded, or shutdown reaches its cancellation phase.
- Responses validate status, header names/values, framing, content length, transfer encoding, trailers, and no-body status rules before bytes are committed.
- Standard `Date`, server identity disclosure, connection, cache, content type, content length, and security headers are controlled by versioned policy.
- Error responses are deterministic, bounded, content-negotiated where supported, and do not expose internal paths, upstream addresses, stack traces, or secrets.

## 8. Static Content Engine

Static serving supports:

- Approved filesystem roots and immutable packaged/Drive artifacts.
- Exact root/alias semantics, index resolution, `tryFiles`, SPA fallback, canonical redirects, and controlled directory listing.
- MIME and charset mapping with safe fallback, `nosniff`, configurable download disposition, and no content-type inference from untrusted query data.
- Strong or weak ETag policy, Last-Modified, RFC preconditions, `If-Range`, single and multiple byte ranges, HEAD, and correct `304`/`416` behavior.
- Precompressed gzip/Brotli asset selection using `Accept-Encoding` and `Vary`, without serving stale or mismatched sidecars.
- Efficient platform file transfer such as zero-copy/sendfile when safe and available, with a bounded asynchronous fallback.
- Bounded file descriptor and metadata caches keyed by canonical file identity with change detection and safe invalidation.

Canonicalization and authorization occur before opening the file. Symlink, hard-link where relevant, mount/reparse point, device, named pipe, alternate data stream, hidden file, dot file, case collision, TOCTOU, and replacement races use a documented fail-closed policy. Content roots are read-only to the serving process unless an explicitly separate managed write workflow owns them.

## 9. Reverse Proxy Engine

Proxying supports:

- HTTP/1.1 upstreams at P0 and HTTP/2/gRPC upstreams at P1.
- Correct upstream URI mapping, hop-by-hop header removal, Host policy, trusted proxy identity, WebSocket upgrade, SSE flushing, trailers, and cancellation.
- Full-duplex streaming where the protocol permits it, with bounded independent request and response flow control.
- Per-attempt connect, TLS handshake, response-header, read-progress, write-progress, idle, and total deadlines constrained by the request deadline.
- Bounded connection pools partitioned by origin, TLS identity, protocol, application, and policy; idle/lifetime limits prevent stale or cross-security-context reuse.
- Health-aware load balancing, target drain, maximum connections, queue bounds, circuit breaking, passive failure tracking, active checks, outlier ejection, and recovery.
- Retry and hedging only under an explicit idempotency/replay policy, attempt cap, retry budget, remaining deadline, and request-body commitment rules.

Buffering defaults are route/profile-specific. When request or response replay requires spooling, memory thresholds, individual file size, total app/process disk quota, file permissions, encryption policy, cleanup deadline, and disk-full behavior are mandatory. Temporary files use SDKWork runtime directories, random names, exclusive creation, and guaranteed bounded cleanup.

The runtime is a reverse proxy. It rejects arbitrary destination selection, open `CONNECT`, untrusted scheme/host interpolation, link-local/cloud metadata destinations, DNS rebinding, and private-address resolution unless an explicit SSRF policy authorizes the target.

## 10. DNS And Dynamic Upstream Resolution

- DNS resolution is asynchronous and never blocks an event-loop worker.
- Positive and negative answers honor configurable TTL floors/ceilings and bounded stale-on-error behavior.
- Resolver concurrency, queries per name, answer count, CNAME depth, response bytes, cache entries, and cache bytes are bounded.
- IPv4/IPv6 selection, fallback, address rotation, and connection racing behavior are deterministic and observable.
- Address changes update new connection selection without terminating healthy in-flight requests.
- Empty, malformed, poisoned, private, loopback, multicast, link-local, or otherwise forbidden answers fail according to upstream SSRF policy.
- Service discovery is introduced only through an approved typed adapter; DNS names and control-plane target sets cannot race to create two authorities.

Resolver failure does not cause an unbounded retry storm. Each upstream declares whether a last valid answer may be used temporarily, how long, and what happens after it expires.

## 11. Compression

- Gzip and Brotli negotiation respects q-values, wildcard/identity semantics, MIME allowlists, minimum/maximum size, existing `Content-Encoding`, `Cache-Control: no-transform`, and `Vary`.
- Static precompressed content is preferred when valid. Dynamic compression uses bounded concurrency and CPU budgets and runs off latency-sensitive executor work when necessary.
- Secrets mixed with attacker-controlled reflected input are excluded from compression through policy to reduce compression side-channel risk.
- Compression buffers, encoder state, output expansion checks, and queue depth are included in memory governance.
- Unsupported encodings produce standards-correct fallback or `406` only when identity is explicitly unacceptable.

## 12. Proxy Cache

The bounded proxy cache supports:

- Canonical keys containing approved scheme, authority, normalized path/query, method, selected headers, upstream identity, and tenant/application scope.
- HTTP freshness, validators, `Age`, `Vary`, conditional revalidation, stale-while-revalidate, stale-if-error, and deterministic warning behavior where supported.
- Explicit handling of authorization, cookies, `Set-Cookie`, private/no-store/no-cache, partial content, redirects, errors, and unsafe methods.
- Collapsed forwarding and bounded per-key waiters to prevent cache stampedes.
- Memory and disk tiers with entry, object, byte, inode, write-rate, and eviction budgets.
- Authenticated purge/ban operations scoped to application and cache namespace.
- Atomic metadata/body publication so partial writes never become cache hits.

Cache poisoning, key confusion, unkeyed inputs, host confusion, range confusion, variant explosion, and cross-tenant disclosure are release-blocking security failures. V1 does not promise globally coherent cache contents across nodes; each policy declares local cache scope and invalidation expectations.

## 13. Hierarchical Resource Governor

Resource limits exist at process, application, listener, virtual host, route, upstream, and client/source scopes as applicable:

| Resource | Required controls |
| --- | --- |
| Memory | Global ceiling, emergency reserve, per-connection/stream/request estimates, cache/buffer/queue budgets, and admission threshold. |
| Connections | Accepted, active, idle, handshake, per-source, per-app, upstream, and pending accept limits. |
| File descriptors/handles | Listener, client, upstream, static file, cache, log, temp file, and reserve budgets. |
| CPU-expensive work | TLS handshakes, regex, compression, crypto, parsing, logging, config compilation, and health-check concurrency. |
| Queues | Accept, handshake, request, upstream wait, retry, log, metric export, cache fill, disk spool, and background operation bounds. |
| Disk | Cache, temporary spool, logs, snapshots, support bundles, and certificate rollback quotas. |
| Configuration | File bytes, include depth/count, apps, listeners, hosts, routes, regex, upstreams, certificates, snapshots, and activation concurrency. |

Admission control begins before the hard ceiling and preserves an emergency margin for health, readiness, diagnostics, drain, and rollback. The server returns bounded `429`, `503`, connection refusal, or protocol-appropriate resets according to policy; it must not continue allocating until the allocator or OS kills the process.

Limits cannot be bypassed by protocol upgrade, retries, internal redirects, compression, cache fills, high-cardinality observability, disconnected clients, or configuration reload. Accounting is released on every success, error, timeout, cancellation, panic boundary, and shutdown path.

## 14. Request-Path Concurrency Rules

- No blocking filesystem, DNS, database, KMS, certificate issuer, process execution, compression, or CPU-heavy regex work runs directly on an asynchronous event-loop worker.
- Shared structures use immutable snapshots, sharded/lock-free reads, or short bounded critical sections. A lock is never held across `.await`, network I/O, filesystem I/O, callback execution, or process control.
- Lock ordering and ownership are documented for mutable runtime registries. Reload, shutdown, certificate rotation, health updates, and metrics collection cannot create cyclic waits.
- Bounded channels declare capacity, producer behavior, consumer failure behavior, shutdown semantics, and what is dropped or rejected on saturation.
- Per-request tasks are cancellable and owned. Detached tasks, background retries, timers, watchers, and health checks have lifecycle supervision and bounded cardinality.
- Panics are contained at approved task/process boundaries, counted, and never converted into a successful response or activation.

## 15. Runtime Observability

Every served request can be correlated with protocol, listener, application, virtual host, route, policy, upstream attempt, response status, bytes, duration, active snapshot, and server-owned trace identity. Labels remain low-cardinality; raw host values, URIs, user IDs, certificate subjects, and arbitrary header values are not metric dimensions.

Runtime metrics include:

- Accepts, active/idle connections, HTTP/2 streams, handshakes, request phases, bytes, response classes, disconnects, timeouts, and protocol errors.
- Memory estimates and allocator/resident observations, descriptors, task counts, queue depth, cache/spool/log disk, and emergency reserve.
- Route/upstream latency, pool state, DNS freshness, health, retries, circuits, cache, compression, rate limits, and load shedding.
- Active snapshot, reload generation, retained snapshots, node convergence, and local recovery state.

Diagnostic dumps are bounded, redacted, authorized, rate-limited, and generated asynchronously. Profiling is disabled by default in public production and uses a separately protected operations surface.

## 16. Verification

Required suites include:

- HTTP/1.0/1.1 parser and semantic conformance, differential parsing, fuzzing, request smuggling, slowloris, malformed chunking, pipeline, disconnect, and timeout tests.
- HTTP/2 conformance, HPACK, flow control, rapid reset, frame flood, stream churn, GOAWAY, trailers, and graceful shutdown tests.
- Static precondition/range/path/security tests on Linux, Windows, and macOS tooling, plus supported Linux production filesystems.
- Proxy streaming, half-close, WebSocket, SSE, gRPC, retry, cancellation, pool isolation, DNS rebinding, upstream TLS, and failure tests.
- Cache RFC behavior, poisoning, variant explosion, stampede, purge authorization, disk-full, crash recovery, and eviction tests.
- Hierarchical memory/connection/descriptor/queue/disk limits, adversarial overload, OOM prevention, executor starvation, deadlock, cancellation leak, and 24-hour soak tests.
- Differential Nginx fixtures for the declared compatibility profile and protocol interoperability with supported clients and upstream servers.

Fuzz and property-test corpora are retained as regression evidence. Failures must reproduce with the exact runtime version, snapshot checksum, seed, platform, and resource profile.

## 17. Acceptance Criteria

- The packaged Rust runtime serves HTTP and HTTPS without an external Nginx process and without synchronous control-plane/database access on the request path.
- HTTP/1.x and HTTP/2 conformance and adversarial parser suites pass with no known request-smuggling ambiguity.
- Static, proxy, WebSocket, SSE, DNS, TLS, routing, compression, and cache behavior meets its declared profile.
- Every allocation-amplifying input has a pre-allocation limit, bounded queue, timeout, cancellation path, and saturation behavior.
- Process health and the last verified application traffic remain available during temporary control-plane, database, certificate issuer, and DNS control API outages.
- OOM, descriptor exhaustion, disk-full, slow-client, retry storm, reload storm, and cache stampede tests degrade predictably without losing the operations reserve.
- No request-path lock is held across asynchronous or external I/O, and concurrency/deadlock test evidence covers reload, shutdown, health, certificate rotation, and upstream churn.
- Performance, memory, compatibility, and availability targets in the parent PRD pass on the published reference profiles.

