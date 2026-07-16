# Runtime Configuration Templates

Application: sdkwork-web
Status: active
Owner: SDKWork maintainers
Specs: CONFIG_SPEC.md, ENVIRONMENT_SPEC.md, RUNTIME_DIRECTORY_SPEC.md, DEPLOYMENT_SPEC.md

## Purpose

This directory contains source-controlled, non-secret configuration templates and development examples. Production runtime configuration is administrator-managed under the canonical SDKWork runtime directories or injected by deployment infrastructure.

## Directory Index

| Path | Purpose |
| --- | --- |
| `topology/` | Safe standalone/cloud environment topology values. |
| `acme/`, `agent/`, `worker/` | Non-secret environment examples for focused processes. |
| `examples/sdkwork.webserver.config.json` | Valid application Web Server configuration example. |
| `sdkwork-api-cloud-gateway.web.*.toml` | Platform gateway integration templates. |

## Forbidden Content

- Production passwords, tokens, database URLs containing credentials, API keys, private keys, or certificate secret material.
- Mutable runtime snapshots, databases, logs, caches, PID files, or node assignments.
- Environment-specific absolute developer machine paths.

The example static asset under `examples/public/` is verification content only. Certificate file references in application configs must resolve through protected runtime paths and are not committed here.

## Reload Contract

`deployment.reload.mode` is `disabled` by default. `watch` polls the selected configuration file at `pollIntervalMs` and publishes a candidate only after bounded reading, Schema/Serde/semantic compilation, static-path resolution, upstream-client construction, and restart-only topology comparison all pass.

Watch reload supports virtual hosts, routes, static resources, fixed responses, redirects, upstream definitions, observability policy, and request-body limits. Listener ids/binds/ports/protocols, TLS policies or material, connection and request admission limits, request/response-progress/write/drain timeouts, and the reload policy require process restart. Invalid candidates retain the last active generation.

## Protocol Resource Limits

`limits` defines finite HTTP/1 total parser bytes, request-line/method/request-target bytes, individual Header/Trailer name/value bytes, header count, complete-header timeout, Chunk Size line, Trailer count/bytes, and HTTP/2 stream/header/reset/send-buffer/Frame-churn/encoded-block budgets. HTTP/2 concurrent decoded-header, encoded-header, and send-buffer products may not exceed 64 MiB per connection. The global HTTP/1 connection/header-window product may not exceed 1 GiB. These values configure the Listener's protocol stack and are Restart-only because existing accepted connections keep their original state.

Cross-protocol Handler limits separately bound raw URI Path bytes, once-decoded Path bytes, Path segments, Query string bytes, Query parameter count, and each Query name/value component. Validation is allocation-free, rejects malformed percent escapes and decoded NUL/control/backslash, and runs before every action. Budget errors return `414`; representation errors return `400`. H1 closes and H2 remains Stream-scoped. These fields are Watch-reloadable because each request reads one immutable generation.

For HTTP/2, `http2AbuseWindowMs` defines one fixed per-connection window. `http2MaxFramesPerWindow`, `http2MaxNewStreamsPerWindow`, and `http2MaxResetFramesPerWindow` bound metadata churn; `http2MaxContinuationFrames` and `http2MaxEncodedHeaderBlockBytes` bound a fragmented Header Block before HPACK decoding. The guard retains no Frame Payload, Header value, Body, timer task, or per-Stream collection. `http2MaxFrameBytes` is both the guard ceiling and advertised `SETTINGS_MAX_FRAME_SIZE`; concurrent Streams and decoded Header List size are also advertised. HPACK dynamic-table sizing is not a configuration field because the selected Hyper server builder does not expose it; H2 0.4.15 retains its finite 4,096-byte default.

`http2KeepAliveIntervalMs` defaults to 60 seconds and accepts 1 second through 1 hour. Hyper sends a PING after that period without any inbound H2 Frame, including on a connection with no active Stream. `http2KeepAliveTimeoutMs` defaults to 20 seconds, accepts 100 milliseconds through 1 minute, and cannot exceed the interval. Missing ACK causes Hyper/H2 to send `GOAWAY(NO_ERROR)` and close; an ACK keeps a healthy idle connection alive. Both fields are Restart-only and do not affect HTTP/1.

`maxConcurrentRequests` defaults to 4,096 and bounds admitted requests across all listeners in one process. Admission never queues: saturation returns fixed `503 Service Unavailable` plus `Retry-After: 1`; HTTP/1 also receives `Connection: close`. A permit remains attached to the response Body through streaming completion, H2 reset, error, or cancellation/drop. Active H2 Header List and send-buffer products and the cross-connection encoded Header Block product are each capped at 1 GiB by semantic validation. The field is Restart-only because the process Semaphore is immutable after startup.

`requestBodyStartTimeoutMs`, `requestBodyIdleTimeoutMs`, `responseBodyIdleTimeoutMs`, and `connectionWriteTimeoutMs` each default to 30 seconds and accept 100 milliseconds through 1 hour. A request that is not already end-of-stream gets a first meaningful-Frame deadline; non-empty Data or Trailer switches to and resets the request Body idle deadline. Empty Data resets neither request nor response progress. Request Body expiry maps to `408`, closes HTTP/1, and remains Stream-scoped for HTTP/2. The connection write deadline measures continuous Pending time for downstream write, flush, or shutdown and disarms on any Ready result. All four controls are Restart-only. They do not replace Keep-Alive idle, TLS handshake, upstream, WebSocket/SSE heartbeat, or gRPC deadline policies.

`http1KeepAliveIdleTimeoutMs` defaults to 75 seconds, matching the common Nginx `keepalive_timeout` default, and accepts 100 milliseconds through 1 hour. It starts only between HTTP/1 requests after active response Body ownership and pending writes finish. Uploads, streaming responses, ordered pipelines, TLS handshake, first-request Header reads, and H2 are not governed by this Timer. Expiry silently closes the idle connection and releases its connection permit. The field is Restart-only.

`maxConnectionAgeMs` defaults to 1 hour, maps to the Nginx `keepalive_time` connection-retirement concept, and accepts 100 milliseconds through 24 hours. One connection-owned Timer starts after transport acceptance regardless of request activity. At expiry Hyper disables HTTP/1 reuse or sends HTTP/2 `GOAWAY(NO_ERROR)`; accepted work may finish only within `drainTimeoutMs`, after which the supervised connection task is canceled and all Acceptor/permit state is released. This Restart-only field is not an idle, request, Body, PING, or TLS handshake timeout.

`http1MaxPipelineDepth` defaults to 16 and accepts 1 through 1,024. It counts complete HTTP/1 request heads that passed original-wire validation but have not yet entered Service dispatch. Dispatch releases one slot synchronously. Excess closes the connection without a synthetic response, request queue, or Body buffering; TLS applies it after decryption and H2 bypasses it. Nginx 1.26.2 has no equivalent request-count control, so this is an explicit SDKWork hardening difference. The field is Restart-only.

An incremental Wire Framing Guard runs after TLS decryption and before Hyper. It supports exactly `Transfer-Encoding: chunked`, validates Extensions and Trailers, rejects TE/CL in either order and every duplicate Content-Length, and applies the Body limit to fixed, Chunked, and HTTP/2 bodies without collecting them. Reverse-proxy forwarding removes inbound `Content-Length` and `Transfer-Encoding`; Reqwest generates fresh upstream framing. Request and response bodies retain Data and Trailer frames. `maxTrailerBytes` and `maxTrailers` apply to both directions and to `Trailer` declarations. The proxy regenerates hop-specific `TE: trailers` toward upstreams; HTTP/1 clients must advertise `TE: trailers` to receive response Trailers.

## Upstream DNS And Address Policy

Resolver profiles use the asynchronous system resolver with finite `timeoutMs` (100..30,000), `maximumAnswers` (1..64), and `maxConcurrentQueries` (1..1,024). Resolver admission never queues: saturation fails the new upstream connection immediately. At most `maximumAnswers + 1` socket addresses are retained so overflow is detected without collecting an unbounded answer set. The `servers` collection must remain empty because custom DNS transport is not implemented by this profile.

Each upstream may select a `resolverRef`, set `idleConnectionTimeoutMs` (100..3,600,000), and declare at most 64 `addressPolicy.allowedCidrs`. Only subnets wholly contained in loopback, RFC1918, RFC6598 shared, IPv4 link-local, IPv6 ULA, or IPv6 link-local ranges are valid authorizations. Public, broad, metadata, documentation, benchmark, multicast, unspecified, and reserved networks cannot be authorized. Literal IP targets are checked at compilation; every DNS answer is rechecked before connection, and one forbidden answer rejects the complete answer set. The configured hostname remains the HTTP authority and HTTPS SNI/certificate-verification name.

The current profile has no authoritative TTL/CNAME access, positive or negative application cache, stale-answer policy, custom DNS server transport, retry loop, active health check, or application-owned Happy Eyeballs policy. Healthy in-flight and pooled active connections are not terminated on DNS change; the finite idle pool timeout bounds unused address retention.

## HTTPS Certificate Selection

A TLS policy configures exactly one of `certificateRef` or `certificateRefs`. The singular form preserves the original single-certificate contract; the plural form accepts 1 through 100 certificate ids for one listener. Every referenced certificate declares one or more Exact or leading-Wildcard DNS names and resolves separate protected certificate/key files outside source control.

At startup the runtime reads each PEM through a 1 MiB bound, validates its leaf validity period and SAN coverage, verifies the private key, and builds immutable Exact/Wildcard SNI hash indexes. Exact names take precedence; a Wildcard covers exactly one left DNS label. Unknown or absent DNS SNI fails closed because no default-certificate policy exists in this profile. TLS policy, certificate-set, path, or content changes remain Restart-only and are not Watch-reloaded.

## Verification

```powershell
cargo run -p sdkwork-web-standalone-gateway -- validate configs/examples/sdkwork.webserver.config.json
pnpm check:repository-docs
```
