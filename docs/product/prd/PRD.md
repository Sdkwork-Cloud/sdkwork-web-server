# SDKWork Web Server PRD

Status: active
Owner: SDKWork maintainers
Application: sdkwork-web
Updated: 2026-07-15
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md, NGINX_SPEC.md, SECURITY_SPEC.md, PERFORMANCE_SPEC.md, PAGINATION_SPEC.md, DEPLOYMENT_SPEC.md, CONFIG_SPEC.md, RUNTIME_DIRECTORY_SPEC.md, OBSERVABILITY_SPEC.md

## Document Map

- [PRD-webserver-app-config.md](PRD-webserver-app-config.md) - per-application Web Server configuration contract and default profiles.
- [PRD-runtime-core.md](PRD-runtime-core.md) - protocol correctness, connection lifecycle, static serving, proxying, DNS, caching, and bounded runtime behavior.
- [PRD-production-operations.md](PRD-production-operations.md) - process lifecycle, overload control, observability, deployment, high availability, upgrades, and commercial operations.
- [PRD-nginx-compatibility.md](PRD-nginx-compatibility.md) - Nginx HTTP compatibility scope, import/export behavior, and conformance requirements.
- [PRD-https-and-certificates.md](PRD-https-and-certificates.md) - HTTPS, TLS, certificate lifecycle, ACME, SNI, key security, and cluster distribution.
- [TECH_ARCHITECTURE.md](../../architecture/tech/TECH_ARCHITECTURE.md) - current technical architecture; it must be revised before implementation to reflect this PRD.

## 1. Background And Problem

Nginx is the operational baseline for static content, reverse proxying, TLS termination, virtual hosts, routing, load balancing, caching, and zero-downtime configuration reload. SDKWork applications currently lack one governed, application-owned Web Server contract that can be executed consistently by a Rust runtime and translated to or imported from Nginx.

The existing SDKWork Web Server implementation is primarily a management control plane. The target product is broader: a Rust-native Web Server with a production data plane, a versioned control plane, Nginx-compatible HTTP behavior, and SDKWork-native application configuration. It must support standalone and cloud deployments without allowing configuration drift, fake validation, silent compatibility degradation, unbounded memory growth, or tenant-wide secret exposure.

Every Web application under `apps/<app-root>/` needs a predictable configuration contract for listeners, ports, HTTPS, domains, certificates, path resources, static bundles, proxies, upstreams, policies, logging, deployment, and rollback. Application identity and release metadata must remain separate from runtime traffic configuration.

## 2. Target Users

- Application developers who need to serve static sites, SPAs, APIs, and hybrid applications without writing ad hoc server configuration.
- Platform engineers who need a common Web Server contract across SDKWork application roots.
- Site and tenant operators who manage domains, certificates, deployments, routing, and rollbacks.
- SRE and security teams who require bounded resource usage, observable behavior, auditable changes, and fail-closed security.
- Nginx operators migrating existing `nginx.conf` and site files to the SDKWork Rust data plane.
- Integrators who need generated SDKs and standard API contracts for configuration automation.

## 3. Product Vision And Principles

SDKWork Web Server is a Rust-native, high-performance HTTP/HTTPS server and reverse proxy with an Nginx HTTP compatibility profile and a first-class SDKWork application configuration model.

Product principles:

- Rust is the default execution engine; Nginx is a supported compatibility, migration, and rendering target.
- Supported Nginx semantics must be behaviorally compatible; unsupported directives must produce explicit diagnostics and must never be silently ignored.
- HTTPS is a first-class production capability, not an optional deployment afterthought.
- Configuration is typed, versioned, validated, immutable after publication, and atomically activated.
- Request and response bodies are streamed; queues, caches, connections, configurations, and background work are bounded.
- Cloud data planes are stateless and horizontally scalable; configuration and certificate distribution are node-scoped and revisioned.
- SDKWork standards govern API envelopes, pagination, IAM, security, observability, deployment, database lifecycle, and SDK consumption.
- No product workflow may report validation, deployment, reload, certificate issuance, or rollback success before the corresponding effect is verified.

## 4. Goals And Non-Goals

### 4.1 Goals

- Serve HTTP and HTTPS traffic directly from the Rust data plane.
- Support static sites, SPAs, reverse proxies, API gateways, and hybrid applications through reusable profiles.
- Define a complete `sdkwork.webserver.app` configuration for every Web application root.
- Support Nginx OSS HTTP Core behavior required by common production sites.
- Import supported Nginx configuration into a normalized model and render normalized configuration back to Nginx.
- Support TLS 1.2 and TLS 1.3, SNI, multiple certificates, ACME, managed certificate rotation, and zero-downtime TLS reload.
- Provide deterministic validation, planning, publication, canary rollout, status observation, and rollback.
- Support PostgreSQL as the cloud and server-grade standalone default, plus an explicitly selected SQLite single-node standalone profile with equivalent supported business behavior.
- Meet measurable throughput, latency, reload, memory, availability, security, and recovery targets.
- Operate as a self-contained Web Server when the management control plane and database are temporarily unavailable.
- Provide strict HTTP parsing, deterministic overload behavior, graceful process lifecycle, and safe operating-system integration.
- Provide app-api and backend-api SDKs for every supported control-plane workflow.

### 4.2 Non-Goals

- V1 does not promise compatibility with arbitrary Nginx third-party dynamic modules.
- V1 does not implement OpenResty/Lua, Perl modules, Nginx mail proxy, or Nginx Plus proprietary features.
- V1 does not treat arbitrary raw Nginx text as executable by the Rust engine.
- V1 does not store certificate private keys, tokens, passwords, or credentials in `sdkwork.app.config.json` or committed Web Server configuration.
- V1 does not use SQLite as a shared cloud-cluster database.
- V1 does not allow per-application code to bypass the Web Server compiler with handwritten runtime routes.
- V1 does not guarantee byte-for-byte identical Nginx-generated error pages; protocol behavior and configured routing semantics are the compatibility target.
- V1 is a reverse proxy, not an unrestricted forward proxy, and does not expose the HTTP `CONNECT` method as an open tunnel.
- V1 does not embed PHP, CGI, FastCGI, uWSGI, SCGI, application language runtimes, or arbitrary native code. Dynamic applications run behind supported upstream protocols.
- V1 static roots are read-only and do not provide WebDAV or arbitrary client-driven filesystem mutation.
- V1 does not require the request data plane to query PostgreSQL, SQLite, Redis, ACME, DNS control APIs, or the management control plane for every request.

## 5. Scope

| Capability | Product requirement |
| --- | --- |
| Rust data plane | HTTP/1.0 compatibility, HTTP/1.1, HTTP/2, strict parsing, streaming, keep-alive, graceful drain, static files, proxying, load balancing, and bounded backpressure. |
| HTTPS | TLS 1.2/1.3, SNI, certificate selection, HTTP-to-HTTPS redirect, ACME, import, renewal, revocation, rotation, OCSP where supported, and secure key handling. |
| Application configuration | One governed Web Server configuration per Web app root, referenced by its SDKWork app manifest. |
| Virtual hosting | Multiple listeners, exact/wildcard/regex domains, default server, canonical domain, and per-host policies. |
| Path resources | Exact/prefix/regex matching, static roots, SPA fallback, redirects, fixed responses, reverse proxy, rewrites, method/header/query/source matching. |
| Upstreams | Weighted targets, common load-balancing algorithms, keepalive, active/passive health checks, retry budgets, circuit breaking, and discovery extension. |
| Nginx compatibility | Import, normalize, validate, explain, diff, render, and conformance-test the supported HTTP profile. |
| Configuration lifecycle | Draft, validate, plan, immutable revision, publish, canary, activate, observe, rollback, and audit. |
| Cluster operation | Node-scoped assignments, signed delta snapshots, fencing, offline recovery, version convergence, and no tenant-wide secret broadcast. |
| Control plane | IAM-protected APIs, SDKs, pagination, idempotency, optimistic concurrency, asynchronous operations, audit, and operational status. |
| Persistence | PostgreSQL cloud and default server authority, explicit SQLite single-node standalone support, portable contracts, migration governance, transaction safety, and drift detection. |
| Operations | Health/readiness, structured logs, metrics, traces, alerts, quotas, rate limits, backup/restore, rolling upgrades, and incident runbooks. |
| Runtime lifecycle | Deterministic bootstrap, config test/dump/explain, non-root operation, atomic reload, graceful shutdown, overload shedding, service-manager integration, and zero-downtime executable upgrade. |

Detailed requirements live in the three PRD shards linked above.

## 6. User Scenarios

### 6.1 Create A Static SPA

An application developer selects the `static-spa` profile. The generated configuration listens on a safe non-privileged development port, serves the app build artifact, applies immutable caching to fingerprinted assets, uses `index.html` fallback for client routes, rejects path traversal, and can be promoted unchanged to a cloud listener.

### 6.2 Publish A Domain With HTTPS

An operator binds a verified domain, selects a managed certificate policy, validates the complete listener/host/route/TLS plan, and starts an asynchronous publication. The system obtains or imports the certificate, distributes only the assigned certificate to each selected node, activates the new revision atomically, verifies public readiness over HTTPS, and records an audit trail.

### 6.3 Reverse Proxy An API

An operator defines an upstream pool and a path route. The server preserves standard forwarding and WebSocket headers, streams request and response bodies, applies timeouts and bounded retries, removes unhealthy targets, and exposes latency, saturation, retry, and error metrics.

### 6.4 Migrate An Nginx Site

An operator imports an Nginx site file. The compiler resolves includes within an approved root, reports unsupported directives with file and line information, shows the normalized configuration and compatibility grade, runs behavioral conformance checks, and publishes only after no blocking diagnostic remains.

### 6.5 Rotate A Certificate Without Downtime

The certificate worker renews a certificate before expiry, validates the chain and hostname coverage, encrypts and distributes the new material, switches TLS contexts atomically, keeps existing connections alive, verifies the new fingerprint, and retains a bounded rollback window.

### 6.6 Roll Back A Failed Revision

Health or SLO checks detect a failed canary. The control plane stops rollout, reactivates the last verified revision with a fencing token, confirms node convergence, and returns an asynchronous operation result. A database-only status change is not a rollback.

## 7. Functional Requirements

- `PRD-FR-001`: Every Web application root must own `sdkwork.app.config.json` and a referenced `sdkwork.webserver.config.json` at the standard app-local path.
- `PRD-FR-002`: The compiler must validate syntax, schema, references, listener conflicts, domain conflicts, route ambiguity, filesystem boundaries, TLS dependencies, upstream reachability policy, and resource budgets before publication.
- `PRD-FR-003`: The Rust engine must implement deterministic virtual-host and route selection compatible with the declared Nginx profile.
- `PRD-FR-004`: Production public listeners must support HTTPS and must not activate when required certificate material is unavailable, invalid, expired, mismatched, or unreadable.
- `PRD-FR-005`: Static resources must support index files, MIME types, conditional requests, byte ranges, precompressed variants, cache policy, SPA fallback, and safe path normalization.
- `PRD-FR-006`: Proxy resources must support streaming, WebSocket upgrades, forwarding headers, configurable buffering, timeouts, bounded retries, health-aware upstream selection, and cancellation propagation.
- `PRD-FR-007`: Long-running publication, rollback, certificate, import, and bulk operations must use SDKWork asynchronous command semantics.
- `PRD-FR-008`: Published revisions must be immutable, checksummed, auditable, atomically activated, and rollback-capable.
- `PRD-FR-009`: Nginx import must preserve source locations and produce an explicit compatibility diagnostic for every parsed directive.
- `PRD-FR-010`: Every node must receive only the applications, routes, certificates, and secrets assigned to that node.
- `PRD-FR-011`: Every list/search API must use store-level SDKWork pagination and return standard `pageInfo`; growing logs, events, revisions, operations, and nodes must use cursor/keyset pagination.
- `PRD-FR-012`: PostgreSQL and SQLite must pass the same business, transaction, migration, and API contract suite for supported deployment profiles.
- `PRD-FR-013`: The management client must expose configuration, validation diagnostics, revisions, diff, rollout, HTTPS, certificate, node, metrics, and audit workflows without hand-built HTTP.
- `PRD-FR-014`: HTTP/1.x and HTTP/2 parsing, framing, method, authority, header, body, connection, and response behavior must be standards-correct and fail closed against request smuggling, slow-client, and protocol-confusion attacks.
- `PRD-FR-015`: The data plane must start from a locally available verified snapshot and continue serving the last verified snapshot without a synchronous database or control-plane dependency in the request path.
- `PRD-FR-016`: Static serving must implement conditional requests, HEAD, byte ranges, safe path resolution, bounded metadata caching, efficient file transfer, and deterministic error handling.
- `PRD-FR-017`: Reverse proxying must implement DNS resolution, connection pooling, HTTP upgrade, flow-controlled streaming, deadlines, health-aware balancing, bounded buffering/spooling, retries, and upstream TLS verification.
- `PRD-FR-018`: A hierarchical resource governor must enforce process, application, listener, host, route, upstream, and client budgets before memory, descriptors, disk, CPU, or worker queues are exhausted.
- `PRD-FR-019`: The runtime must provide config validate/dump/explain, start, readiness, reload, drain, stop, status, and version operations with service-manager and container lifecycle integration.
- `PRD-FR-020`: Reload and executable upgrade must stage a complete candidate, preserve accepted healthy connections, fence concurrent transitions, prove the served revision, and restore the last verified generation on failure.
- `PRD-FR-021`: Administrative, health, readiness, metrics, profiling, and debug surfaces must use separately governed exposure policies and must never be implicitly exposed through an application virtual host.

## 8. Non-Functional Requirements

### 8.1 Performance And Memory

- A reference benchmark profile must compare the Rust data plane with the current stable Nginx OSS release on identical hardware, kernel, TLS, payload, connection, and upstream settings.
- For supported static and reverse-proxy scenarios, V1 throughput must reach at least 80% of the Nginx baseline and p95 added proxy latency must not exceed 5 ms under the declared reference load.
- The server must sustain at least 100,000 idle keep-alive connections on the documented 8-vCPU/16-GB reference node without OOM or loss of control-plane readiness.
- A 10,000-route configuration must validate and atomically activate within 1 second p99 after required resources are locally available.
- Request memory must be bounded by configured body/header windows and streaming buffers, not total request or response size.
- List API memory must be O(page size); agent synchronization memory must be O(delta size); caches, queues, connection pools, log buffers, and retry inventories must have explicit hard limits.
- The runtime must reserve a configurable emergency margin and begin admission control before allocator or operating-system exhaustion; reaching a configured limit must produce bounded rejection or shedding, not process OOM.
- Per-connection and per-stream memory budgets must include parser, header, flow-control, TLS, proxy, compression, and observability allocations rather than counting body buffers alone.
- Load and soak tests must demonstrate stable memory over 24 hours with no monotonic growth from reloads, ACME challenges, certificate rotations, disconnected clients, or failed upstreams.

### 8.2 Reliability And High Availability

- A production three-node data-plane deployment targets 99.99% monthly request availability, excluding approved maintenance windows and external upstream failure outside configured policy.
- Configuration reload and certificate rotation must not drop accepted healthy connections.
- A failed canary must stop automatically; the last verified revision must be restorable within 30 seconds after rollback authorization.
- Control-plane cloud recovery targets are RPO <= 5 minutes and RTO <= 15 minutes, backed by tested PostgreSQL backup and restore procedures.
- Node synchronization must be idempotent, resumable, checksummed, fenced, and safe under duplicate, delayed, reordered, or replayed messages.
- Worker claims must use leases with expiry and fencing so a process crash cannot leave permanent in-progress state.
- Control-plane, database, DNS control API, and certificate issuer outages must not interrupt already active valid traffic configuration.
- Graceful shutdown must stop new accepts, advertise HTTP/2 shutdown, drain eligible requests to a deadline, and then terminate remaining work deterministically.

### 8.3 Security And Privacy

- Production public traffic must use HTTPS; plaintext HTTP may exist only for explicit redirect, ACME HTTP-01, loopback development, or documented private health traffic.
- TLS 1.0 and 1.1 are forbidden. TLS 1.2 and 1.3 are supported; insecure ciphers, invalid chains, hostname mismatch, expired certificates, and unsafe fallback are rejected.
- Private keys must be encrypted at rest, redacted from API responses/logs/metrics, referenced through secret or KMS identities, and distributed only to authorized nodes.
- Domain ownership must be verified through an approved DNS or HTTP challenge before production certificate issuance or public activation.
- Filesystem resources must be confined to approved roots with symlink and traversal protection.
- Configuration size, route count, regex complexity, request headers, body size, connections, rate, and upstream retries must be quota-controlled per application and tenant.
- IAM, RBAC, optimistic concurrency, idempotency, audit, secure headers, input validation, and problem details follow the referenced SDKWork standards.

### 8.4 Observability And Operations

- Every request must have a server-owned trace identity and structured access/error logging with secret redaction.
- Metrics must cover requests, latency, response size, active connections, TLS handshakes, certificate expiry, route/upstream status, retries, cache, rate limits, pool saturation, config revision, rollout, and node convergence.
- Traces must cover listener, virtual host, route, policy, upstream attempt, and control-plane operation without recording raw secrets or unbounded labels.
- `/healthz`, `/readyz`, and `/metrics` must have explicit exposure policies and remain independent of business API routes.
- Container logging defaults to structured stdout/stderr; service-package logging follows the SDKWork runtime directory standard and must survive disk-full conditions through bounded buffering and an explicit drop/block policy.
- Production releases require operator, backup/restore, certificate incident, failed rollout, node divergence, and capacity runbooks.

## 9. Success Metrics

- 100% of active Web app roots have valid app manifests and Web Server configurations.
- 100% of production domains serve a valid TLS 1.2/1.3 certificate and pass automated expiry/hostname/chain checks.
- 100% of published revisions have validation evidence, checksum, actor, operation, node convergence, and rollback target.
- Zero unsupported Nginx directives are silently accepted.
- Zero unbounded interactive list queries, tenant-wide full configuration syncs, or unbounded in-memory request bodies remain in production paths.
- PostgreSQL and SQLite compatibility suites pass for every release.
- Nginx conformance, HTTPS, load, soak, security, failover, and rollback gates pass before stable release.
- Protocol conformance, request-smuggling, slow-client, overload, descriptor exhaustion, disk-full, DNS failure, cache poisoning, and graceful executable-upgrade gates pass before stable release.
- Critical certificate expiry, invalid rollout, node divergence, and capacity saturation alerts meet documented detection and response targets.

## 10. Phases

### Phase 0 - Contracts And Truthfulness

- Finalize the Web Server app configuration schema, compatibility profile, HTTPS contract, requirements, ADRs, and verification matrix.
- Remove or reclassify existing production-complete claims until evidence satisfies this PRD.

### Phase 1 - Rust HTTP/HTTPS Foundation

- Deliver listeners, virtual hosts, static resources, reverse proxying, TLS, SNI, certificate import, safe reload, configuration compiler, immutable revisions, PostgreSQL standalone defaults, and explicit SQLite single-node behavior.
- Deliver strict HTTP/1.x and HTTP/2 framing, resource governor, DNS resolver, bounded streaming/spooling, process lifecycle, protected administration, and standalone operation from a local verified snapshot.

### Phase 2 - Nginx Compatibility And Managed Certificates

- Deliver supported Nginx import/render/conformance, ACME HTTP-01 and DNS-01 provider model, renewal, OCSP policy, upstream health, load balancing, cache, rate limits, and PostgreSQL cloud behavior.

### Phase 3 - Cluster And Commercial Operations

- Deliver node-scoped delta distribution, canary rollout, fencing, autoscaling evidence, backup/restore, SLO dashboards, alerting, audit completeness, quotas, billing/entitlement hooks, and operator runbooks.
- Deliver zero-downtime executable upgrade, multi-region failure evidence, deterministic overload behavior, capacity models, support bundles, and long-term compatibility policy.

### Phase 4 - Advanced Protocols And Extensions

- Evaluate HTTP/3, TCP/UDP stream proxying, WASM policy modules, WAF integrations, service discovery, and approved Nginx Plus-equivalent capabilities through separate requirements and ADRs.

## 11. Linked Requirements

- [REQ-2026-0003 Rust Web Server data-plane foundation](../requirements/REQ-2026-0003-rust-webserver-data-plane-foundation.md) - accepted bounded foundation for configuration compilation and the first real HTTP/HTTPS data-plane slice; it does not satisfy the remaining Phase 1 or commercial release gates.
- [REQ-2026-0004 PostgreSQL and SQLite lifecycle parity](../requirements/REQ-2026-0004-database-engine-parity.md) - SQLite fresh-bootstrap and drift evidence is implemented; PostgreSQL execution and cross-engine repository transaction/parity evidence remain in progress.
- [REQ-2026-0005 Atomic same-topology configuration reload](../requirements/REQ-2026-0005-atomic-config-reload.md) - accepted local-file Watch and lock-free immutable generation publication; persisted rollback, TLS/listener handoff, executable upgrade, and cluster convergence remain separate gates.
- [REQ-2026-0006 Multi-certificate SNI selection](../requirements/REQ-2026-0006-multi-certificate-sni.md) - accepted static immutable Exact/Wildcard certificate selection and activation-time leaf/time/SAN/key checks; zero-downtime rotation remains a separate gate.
- [REQ-2026-0007 Bounded HTTP protocol ingress](../requirements/REQ-2026-0007-bounded-http-protocol-ingress.md) - accepted HTTP/1 header bytes/count/deadline and HTTP/2 stream/header/reset/buffer controls; its temporary no-Transfer-Encoding policy is superseded by REQ-2026-0008.
- [REQ-2026-0008 Safe HTTP/1 Chunked framing](../requirements/REQ-2026-0008-safe-http1-chunked-framing.md) - accepted original-wire framing guard, bounded Chunked/Trailer input, all-action Body budgets, TLS/ALPN separation, and proxy reframing; full HTTP conformance remains separate.
- [REQ-2026-0009 Bounded proxy Trailer fidelity](../requirements/REQ-2026-0009-bounded-proxy-trailer-fidelity.md) - accepted frame-preserving request/response Trailer proxying for declared HTTP/1 and HTTP/2 profiles with finite declaration/frame budgets; full gRPC and undeclared cross-protocol synthesis remain separate.
- [REQ-2026-0010 HTTP/1 connection semantics](../requirements/REQ-2026-0010-http1-connection-semantics.md) - implements HTTP/1.0 default-host/Keep-Alive/Pipeline behavior, strict Expect/Continue, TCP half-close, upstream Expect termination, TLS evidence, and an explicit Nginx 1.26.2 difference matrix; full HTTP/1 conformance remains separate.
- [REQ-2026-0011 Bounded HTTP/1 request fields](../requirements/REQ-2026-0011-bounded-http1-request-fields.md) - accepted original-wire request-line, method, request-target, Header/Trailer name, and Header/Trailer value allocation ceilings before Hyper parsing; query semantics and complete parser conformance remain separate.
- [REQ-2026-0012 Bounded HTTP/2 abuse and drain](../requirements/REQ-2026-0012-bounded-http2-abuse-and-drain.md) - accepted constant-memory per-connection Frame/Stream/Reset and encoded Header Block/CONTINUATION controls with real SETTINGS, recovery, H2 local-error GOAWAY, and graceful drain evidence; exhaustive HPACK/fuzz/Nginx differential work remains separate.
- [REQ-2026-0013 Process request admission](../requirements/REQ-2026-0013-process-request-admission.md) - accepted one process-wide non-queuing active-request gate whose permits remain held through streaming response Body completion or cancellation, with real HTTP/1 and TLS/H2 overload/recovery evidence; adaptive memory pressure and fairness remain separate.
- [REQ-2026-0014 Response progress timeouts](../requirements/REQ-2026-0014-response-progress-timeouts.md) - accepted distinct response Body producer-idle and downstream Socket write-stall deadlines with empty-Frame hardening, request-admission release, slow-reader handling, and real HTTP/1/TLS/H2 recovery evidence; request Body and Keep-Alive policies were delivered by later focused requirements.
- [REQ-2026-0015 Request Body progress timeouts](../requirements/REQ-2026-0015-request-body-progress-timeouts.md) - accepted all-action first-meaningful-Frame and subsequent progress deadlines with safe HTTP/1 close, H2 Stream isolation, one-permit recovery, and proxy error classification; HTTP/1 Keep-Alive idle was delivered separately by REQ-2026-0016.
- [REQ-2026-0016 HTTP/1 Keep-Alive idle timeout](../requirements/REQ-2026-0016-http1-keep-alive-idle-timeout.md) - accepted protocol-scoped request-between-request idle reaping with one-connection permit recovery and no interruption of uploads, streaming responses, ordered pipelines, or H2 on mixed ALPN listeners.
- [REQ-2026-0017 Bounded URI and Query components](../requirements/REQ-2026-0017-bounded-uri-query-components.md) - accepted allocation-free cross-H1/H2 Path/Query budgets, percent safety, proxy Query fidelity, H2 recovery, and atomic policy reload; canonical Nginx URI normalization remains separate.
- [REQ-2026-0018 Canonical URI normalization](../requirements/REQ-2026-0018-canonical-uri-normalization.md) - in-progress dual raw/canonical URI implementation for Nginx-style route and static identity plus canonical `stripPrefix` proxy rewriting; acceptance is blocked on human review of the proposed ADR and deliberate security differences.
- [REQ-2026-0019 Bounded HTTP/1 Pipeline depth](../requirements/REQ-2026-0019-bounded-http1-pipeline-depth.md) - accepted connection-local pending-request-head limit enforced before Hyper service dispatch, with plain/TLS close, recovery, H2 bypass, Restart-only reload, and explicit Nginx hardening-difference evidence.
- [REQ-2026-0020 HTTP/2 Keep-Alive PING timeout](../requirements/REQ-2026-0020-http2-keep-alive-ping-timeout.md) - accepted Hyper/H2-owned idle PING and ACK-timeout policy with `GOAWAY(NO_ERROR)`, connection recovery, H1 isolation, Restart-only reload, and pinned Nginx evidence; healthy-idle maximum age remains separate.
- [REQ-2026-0021 Proxy early-response request lifecycle](../requirements/REQ-2026-0021-proxy-early-response-request-lifecycle.md) - accepted two-phase pause/cancel ownership for streamed uploads, with complete response preservation, HTTP/1 connection close/pool eviction, H2 `NO_ERROR` Stream isolation/reuse, timeout/admission recovery, and pinned Nginx evidence.
- [REQ-2026-0022 Bounded connection maximum age](../requirements/REQ-2026-0022-bounded-connection-maximum-age.md) - accepted one Nginx-aligned finite HTTP/1/H2 connection lifetime, pre-task connection admission, Hyper-owned graceful retirement, bounded in-flight drain, supervised task cancellation, fresh-connection recovery, and Restart-only Watch behavior.
- [REQ-2026-0023 Bounded upstream DNS and SSRF policy](../requirements/REQ-2026-0023-bounded-upstream-dns-ssrf-policy.md) - accepted asynchronous system-resolution, answer/concurrency/timeout bounds, literal and resolved-address policy, DNS rebinding defense, and finite idle upstream pool lifetime; authoritative TTL/cache/custom transport and health-aware resolution remain separate gates.
- [REQ-2026-0024 Upstream TLS identity and pool isolation](../requirements/REQ-2026-0024-upstream-tls-identity-pool-isolation.md) - accepted system/custom/combined trust, mutual TLS identity, TLS-version bounds, fail-closed generation construction, and security-context pool isolation; pinning, revocation, and dynamic secret providers remain separate gates.
- [REQ-2026-0025 Bounded upstream admission and passive health](../requirements/REQ-2026-0025-bounded-upstream-admission-passive-health.md) - accepted non-queuing per-upstream request lifetime budget, passive target ejection, single half-open probe, recovery, and all-target overload behavior without hidden retries; physical connections are now covered by REQ-2026-0028, while retries and cluster health remain separate gates.
- [REQ-2026-0026 Supervised active upstream health](../requirements/REQ-2026-0026-supervised-active-upstream-health.md) - accepted bounded HTTP probes with one generation-owned scheduler, global concurrent-future ceiling, independent active/passive target state, recovery thresholds, and explicit Watch/shutdown cancellation; REQ-2026-0028 now shares physical capacity without treating local saturation as target failure, while metrics/admin, cluster health, retries, and advanced balancing remain separate gates.
- [REQ-2026-0027 Adaptive resource pressure admission](../requirements/REQ-2026-0027-adaptive-resource-pressure-admission.md) - accepted bounded Windows/Linux process resource sampling, strict reserve/hysteresis validation, total/business request partition, pre-task socket shedding, exact fixed health reserve, HTTPS/H2 isolation, recovery, Restart-only Watch behavior, and supervised shutdown; hard allocator caps, resource metrics/admin, distributed fairness, and load/soak proof remain separate gates.
- [REQ-2026-0028 Bounded upstream physical connections](../requirements/REQ-2026-0028-bounded-upstream-physical-connections.md) - generation-level aggregate `maxConnections` across connecting, TLS, active HTTP/1, multiplexed H2, and idle sockets; local saturation preserves passive/active health, and Watch/shutdown close retired idle pools. Per-target Nginx `max_conns`/shared-zone compatibility remains separate.
- [REQ-2026-0029 Bounded upstream response Headers](../requirements/REQ-2026-0029-bounded-upstream-response-headers.md) - accepted finite HTTP/1 parser and H2 Header List budgets plus allocation-free decoded occurrence accounting before forwarding; oversize responses fail as generic `502` target failures without Header/Body disclosure. Nginx proxy-buffer directive compatibility remains separate.
- [REQ-2026-0030 Weighted upstream selection](../requirements/REQ-2026-0030-weighted-upstream-selection.md) - accepted bounded relative target weights with equal-weight round robin, active/passive health exclusion, single half-open recovery, real dual-origin distribution, and Watch replacement; exact Nginx smooth scheduling, shared zones, and slow start remain separate.
- [REQ-2026-0031 Bounded WebSocket reverse proxy](../requirements/REQ-2026-0031-bounded-websocket-reverse-proxy.md) - accepted classic HTTP/1.1 WebSocket/WSS forwarding with strict upgrade negotiation, fixed bidirectional buffers, upstream admission and generation ownership, hard lifetime, shared shutdown drain budget, Watch continuity, and generic invalid-101 failure; RFC 8441, frame policy, heartbeat, and exact Nginx timeout directives remain separate.
- [REQ-2026-0032 Bounded management HTTP metrics](../requirements/REQ-2026-0032-bounded-management-http-metrics.md) - replaces the disconnected zero-value management `/metrics` registry with one registry shared by app-api, backend-api, and the scrape handler; unmatched routes collapse to one series, request/stage series and label bytes have hard ceilings, and overflow is counted.
- [REQ-2026-0033 Bounded data-plane operations metrics](../requirements/REQ-2026-0033-bounded-data-plane-operations-metrics.md) - adds a separate loopback-only host operations listener and a fixed-atomic runtime registry for connection/request lifetimes and rejection, upstream outcomes, aggregate target health, resource pressure, reload, and WebSocket tunnel state. REQ-2026-0034 extends this registry with bounded RED and capacity metrics; traces, exporters, authenticated remote operations, dashboards, alerts, and cluster aggregation remain separate gates.
- [REQ-2026-0034 Bounded data-plane RED and capacity metrics](../requirements/REQ-2026-0034-bounded-data-plane-red-capacity-metrics.md) - adds fixed request/upstream latency histograms, streaming Body/tunnel byte counters, normalized protocol and DNS outcomes, and current-generation upstream request/physical-connection capacity without dynamic labels, queues, payload retention, or invented idle-pool values.
- [REQ-2026-0035 Bounded safe upstream retries](../requirements/REQ-2026-0035-bounded-safe-upstream-retries.md) - adds opt-in Nginx-token-aligned sequential failover for Body-end-of-stream idempotent requests, with distinct eligible targets, one retained admission permit, finite attempt/total deadlines, fixed retry metrics, and no request payload buffering or replay.
- [REQ-2026-0036 Bounded target physical connections](../requirements/REQ-2026-0036-bounded-target-physical-connections.md) - adds optional unique-authority target socket ceilings beneath the aggregate upstream limit, with non-queuing connector admission, H1/H2/idle RAII ownership, target-isolation tests, and fixed aggregate telemetry.
- [REQ-2026-0037 Bounded backup upstream targets](../requirements/REQ-2026-0037-bounded-backup-upstream-targets.md) - adds an immutable primary/backup target tier, health-aware primary precedence, weighted backup fallback, half-open primary recovery, retry composition, and atomic Watch role replacement without request-level tier state.
- [REQ-2026-0002 instant-acme Let's Encrypt](../requirements/REQ-2026-0002-instant-acme-letsencrypt.md) - existing draft requirement; it must be revised to match the HTTPS shard before implementation continues.
- New implementation work must be decomposed into `REQ-*` records for configuration contracts, HTTP runtime core, process and resource lifecycle, Nginx compatibility, HTTPS/certificates, persistence, cluster distribution, observability, deployment, and conformance testing.
- Existing certificate and distribution ADRs must be reviewed because their current implementation-complete claims do not satisfy this PRD.

## 12. Release Acceptance

A stable commercial release is accepted only when:

- All P0/P1 requirements in this PRD and linked ready requirements have implementation and test evidence.
- Authoritative configuration schemas, OpenAPI, generated SDKs, runtime behavior, database contracts, and documentation agree.
- PostgreSQL and SQLite tests, Nginx conformance, HTTPS interoperability, security, performance, soak, chaos, rolling upgrade, backup/restore, and rollback tests pass.
- No fake success, silent directive ignore, unbounded memory path, plaintext private key exposure, cross-node secret over-distribution, or undocumented compatibility exception remains.
- Production deployment includes immutable artifacts, checksum, signature, SBOM, provenance, health probes, resource limits, autoscaling policy, disruption policy, secret manager integration, and operational runbooks.

## 13. Open Questions

- Whether HTTP/3 enters Phase 2 or remains Phase 4 depends on client demand and the selected Rust transport stack's production maturity.
- The initial DNS-01 provider set and credential ownership model require a separate integration requirement.
- WASM/WAF extension ABI, isolation limits, and compatibility policy require an ADR before plugin work begins.
- Nginx Plus-specific features require explicit product licensing and clean-room compatibility review before inclusion.
