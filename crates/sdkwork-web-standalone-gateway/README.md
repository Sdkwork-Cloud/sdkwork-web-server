# sdkwork-web-standalone-gateway

Domain: platform
Capability: webserver
Package type: Rust standalone gateway
Status: active

## Public API

- `build_router`: existing management app-api/backend-api composition.
- `run_database_migrate_only`: database migration operation.
- `run_data_plane_until`: database-independent HTTP/HTTPS application data plane with explicit shutdown.
- `run_data_plane_from_config_until`: database-independent data plane with opt-in same-topology configuration Watch and failed-candidate retention.
- Binary operations: `serve-management`, `db-migrate`, `validate`, and `data-plane`.

## Required SDK Surface

This runtime does not consume generated HTTP SDKs. It mounts repository-owned management route crates and executes application Web Server traffic from the compiled config port.

## Configuration

Management mode uses SDKWork typed server/database environment configuration. `validate` and `data-plane` accept an explicit config argument or `SDKWORK_WEB_SERVER_CONFIG_FILE`.

`data-plane` honors `deployment.reload`: `disabled` keeps the startup generation; `watch` publishes verified same-topology generations through a lock-free request-path pointer. Listener, TLS, connection-admission, timeout, drain, or Watch-policy changes require restart.

## Deployment Profile And Runtime Target Behavior

The current executable is the standalone server gateway. The data-plane-only operation does not initialize PostgreSQL or SQLite. Cloud node-scoped snapshot consumption remains a separate requirement.

## Security

- Rustls provides TLS 1.2/1.3 and configured ALPN.
- Configuration and TLS PEM readers are bounded before JSON or Rustls parsing.
- HTTPS startup builds an immutable Exact/Wildcard SNI index only after every leaf certificate is currently valid, its SAN covers the declared names, and its private key matches. Unknown or absent DNS SNI fails closed.
- Reload logs revision identifiers and classified failures only; it never logs candidate bytes or private-key material.
- The accept loop acquires process and listener connection permits with non-queuing `try_acquire_owned` before creating a task, TLS state, or HTTP protocol state. Rejected sockets create no task; the listener-owned `JoinSet` therefore has a hard active-task bound, and its biased loop reaps completed tasks before accepting more sockets.
- One process-wide non-queuing request Semaphore is shared across listeners. `maxConcurrentRequests` saturation returns bounded `503`/`Retry-After`; HTTP/1 requests connection close, while HTTP/2 rejects only the excess Stream.
- An admitted permit is held by a zero-copy response Body wrapper until streaming completion, error, H2 reset, or Body drop. Handler completion alone does not release proxy/static response capacity.
- The same Body wrapper enforces `responseBodyIdleTimeoutMs` without collecting Frames. Only non-empty Data or Trailer Frames reset progress; empty Data cannot keep a permit alive.
- One request Body wrapper enforces `requestBodyStartTimeoutMs` and `requestBodyIdleTimeoutMs` for fixed, redirect, static, and proxy actions without collecting or copying Frames. Empty Data cannot keep a request alive; expiry returns `408`, closes HTTP/1, and isolates HTTP/2 to the affected Stream.
- A post-TLS accepted-stream wrapper enforces `connectionWriteTimeoutMs` independently for continuously Pending write, flush, and shutdown operations. Reads and successful writes pass through unchanged.
- A protocol-aware Stream/Service pair enforces `http1KeepAliveIdleTimeoutMs` only between HTTP/1 requests. Response Body leases and pending-flush state prevent active uploads, streaming responses, and Pipeline work from being mistaken for idle; negotiated H2 bypasses the policy.
- The original-wire Guard and a paired Service share one connection-local atomic count for `http1MaxPipelineDepth`. Complete request heads consume a slot before Hyper sees them, synchronous dispatch releases it, excess closes the connection, and no request Body or response is queued by the guard. TLS H1 is covered after decryption; H2 bypasses the state.
- HTTP/1 parser bytes, request-line/method/request-target bytes, individual Header/Trailer name/value bytes, header count, and header-read time are explicit; HTTP/2 stream, decoded/encoded header, reset, flow-control, send-buffer, Frame-rate, new-Stream-rate, and Continuation values are bounded before serving.
- One allocation-free request URI scanner applies raw/decoded Path, segment, Query string, parameter, and name/value component budgets across H1/H2 before route selection. It rejects malformed percent escapes and decoded NUL/control/backslash; valid proxy Query bytes remain unchanged.
- A bounded original-wire Framing Guard runs after TLS and before Hyper, supports exactly Chunked input, validates Trailers, and rejects TE/CL plus duplicate Content-Length before normalization. Proxy requests are reframed by Reqwest after client framing headers are removed.
- A constant-memory HTTP/2 Wire Guard runs after TLS ALPN and before Hyper, validates Preface/Frame metadata, counts fixed-window Frame/new-Stream/`RST_STREAM` churn, and bounds encoded Header Blocks without retaining payloads or Header values. Wire violations close only the offending connection; H2 retains protocol and GOAWAY ownership.
- Hyper/H2 sends an idle PING at `http2KeepAliveIntervalMs`; missing ACK at `http2KeepAliveTimeoutMs` produces `GOAWAY(NO_ERROR)` and connection close. A compliant idle peer remains connected, HTTP/1 bypasses the policy, and SDKWork adds no parallel PING state machine or task.
- Every accepted connection has one `maxConnectionAgeMs` deadline. Expiry asks Hyper to stop HTTP/1 reuse or send H2 `GOAWAY(NO_ERROR)`, polls accepted work for at most `drainTimeoutMs`, then cancels the supervised connection task. Listener shutdown uses the same owned task registry; no accepted connection task is detached.
- Hyper advertises configured maximum concurrent Streams, Frame size, and decoded Header List size. HPACK dynamic-table sizing is not exposed by the selected server builder, so no unsupported application field exists and H2 0.4.15 retains its finite 4,096-byte default.
- Fixed, Chunked, proxy, non-proxy, and HTTP/2 bodies without Content-Length are stream-counted against the application Body limit; no Body-sized collection is introduced.
- Reverse-proxy request and response bodies preserve Data and Trailer frames. Trailer declarations and actual frames share finite count/byte limits and forbidden-field checks; only canonical `TE: trailers` is regenerated toward upstreams.
- HTTP/1 enables write-side half-close, accepts only HTTP/1.1 `Expect: 100-continue`, returns early `413` without `100` for known fixed-length overflow, rejects unsupported expectations with `417`, and removes `Expect` before proxying upstream. HTTP/1.0 default-host, Keep-Alive, ordered Pipeline, finite Pipeline read-ahead, and Transfer-Encoding rejection are raw-socket tested.
- Static traversal/symlink escape checks run before `tower-http` file service.
- Proxy bodies are streamed with a counted hard limit; redirects and protocol upgrades are not followed silently.
- Upstream early responses pause incomplete uploads without draining or collecting them. Complete response handoff cancels the upstream producer; HTTP/1 explicitly closes the downstream connection, while H2 uses `RST_STREAM(NO_ERROR)` and preserves the connection for other Streams.
- Hop-by-hop headers and untrusted forwarding identity are removed/replaced.
- One bounded asynchronous system-resolver runtime is shared per resolver profile. Non-queuing admission, finite lookup timeout, and `maximumAnswers + 1` retention prevent resolver waiter and answer growth; an upstream-specific immutable policy rechecks every answer before Reqwest receives it.
- Reverse proxy destinations default to public unicast. Explicit narrow CIDRs may authorize loopback/private/shared/link-local/ULA targets, but cloud metadata and hard special-use addresses always fail closed. Mixed permitted/forbidden answer sets fail as a whole, and the configured hostname remains Host/SNI/certificate authority.

Request/response progress, HTTP/1 Keep-Alive, H2 PING/ACK, maximum connection age, proxy early-response ownership, and DNS resolver limits now provide finite fallbacks for slow uploaders, quiescent producers, slow-reading peers, request-between-request idle sockets, healthy but stale connections, upstreams that answer before upload completion, and DNS rebinding attempts. Custom resolver transport, authoritative TTL/cache/stale behavior, CNAME inspection, upstream health/retry/balancing, WebSocket/SSE heartbeat, gRPC deadline behavior, and adaptive memory-pressure admission remain separate commercial gates.

## Extension Points

New route actions and listener capabilities must first exist in the root schema/core compiler, then gain a focused adapter and real integration tests. Business API behavior remains in route/service/repository crates.

## Verification

```powershell
cargo test -p sdkwork-web-standalone-gateway
cargo run -p sdkwork-web-standalone-gateway -- validate configs/examples/sdkwork.webserver.config.json
```
