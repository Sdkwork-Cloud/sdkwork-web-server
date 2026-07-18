# sdkwork-webserver-core

Domain: platform
Capability: webserver-config
Package type: Rust crate
Status: active

## Public API

The crate exports the `sdkwork.webserver.app` Serde model, bounded file loader, SHA-256 compiled revision, schema/semantic diagnostics, immutable compiled indexes, resource lookups, and deterministic listener/host/route selection.

It does not bind sockets, execute HTTP, access SQLx, or own management API contracts.

## Configuration

`specs/sdkwork.webserver.config.schema.json` at the repository root is the machine authority. Loading reads at most 1 MiB plus one detection byte, rejects unknown fields, and requires all semantic references and resource paths to validate before compilation. The revision hashes the exact bytes that passed validation.

## Deployment Profile And Runtime Target Behavior

The same compiler is used by standalone and future cloud data planes. It consumes local authored or published configuration and has no database bootstrap dependency.

## Security

- Static roots are relative to the configuration directory and cannot escape it.
- Certificate keys remain protected file references; bytes are not part of the model.
- TLS policies accept one legacy `certificateRef` or a bounded `certificateRefs` collection, never both; duplicate normalized SNI ownership and incomplete listener coverage fail compilation.
- Listener `trustedProxy` is optional and therefore trusts no forwarding identity by default. When present it requires 1..64 typed CIDRs, supports exactly `x-forwarded-for`, defaults to Nginx-compatible non-recursive selection, bounds the chain to 16 hops and 4 KiB by default, and cannot exceed 64 hops, 64 KiB, or the global Header field-value budget. Unknown fields/tokens, malformed CIDRs, duplicates, and incoherent limits fail compilation.
- Unsupported resolver, TLS, routing, or Nginx behavior fails explicitly.
- Route and host indexes are immutable after compile.
- HTTP/1 request-line/method/request-target, individual Header/Trailer field, total Header/Chunk/Trailer, and HTTP/2 stream/header/reset/Frame-churn/Continuation budgets are finite. Cross-field HTTP/1 global and HTTP/2 decoded-header/encoded-header/send-buffer per-connection products are validated, and any protocol-budget reload is classified as Restart-only by the runtime.
- Cross-protocol URI Path/decoded-Path/segment and Query string/parameter/component budgets are finite and coherently enabled. They are Handler-level policy and may change through atomic generation reload.
- The schema exposes only controls wired by the runtime. It does not define HPACK dynamic-table sizing because the selected Hyper server builder has no supported configuration path.
- `maxConcurrentRequests` has a finite default/range. Active HTTP/2 Header List/send-buffer products and the connection-level encoded Header Block product are globally capped; changing the process request gate is Restart-only.
- Request Body start/idle, response Body idle, and downstream connection write timeouts have finite defaults/ranges and are Restart-only so existing accepted connections never mix timeout generations.
- HTTP/1 Keep-Alive idle timeout has a finite Nginx-aligned default/range and is Restart-only; its runtime activity semantics remain outside this compiler crate.
- Maximum connection age has a finite one-hour Nginx-aligned default and 24-hour ceiling. It is Restart-only so accepted connections cannot mix retirement policy generations.
- Resolver profiles have finite timeout, retained-answer, and concurrent-query bounds. Upstream idle pool lifetime and allowed-CIDR collections are finite; unknown resolver references, custom DNS server lists, broad/public CIDRs, and unauthorized literal private addresses fail compilation.
- Address authorization defaults to public unicast. Only explicit narrow loopback/private/shared/link-local/ULA CIDRs are accepted, while metadata and hard special-use destinations remain forbidden even under a containing allowlist. IPv4-mapped, IPv4-compatible, well-known NAT64, and 6to4 addresses are evaluated through their embedded IPv4 policy.
- Optional upstream TLS policy is valid only for all-HTTPS target sets. System/custom/combined trust semantics, paired client identity files, TLS 1.2/1.3 version ordering, safe relative paths, file existence, canonical containment beneath the configuration directory, and finite CA-file collections are validated before runtime construction.
- Per-upstream in-flight request admission has a finite default/range. Passive-health failure threshold, ejection time, and bounded unique `5xx` status collection are Schema-validated.
- Optional per-upstream retry accepts 2..8 total attempts, a finite total timeout, and a non-empty unique fixed condition set. Attempts cannot exceed targets and total retry time cannot exceed the sum of per-attempt request ceilings. Runtime replay remains restricted to bodyless idempotent requests.
- Per-upstream `maxConnections` defaults to 256, accepts 1..100,000, and requires `maxIdleConnections <= maxConnections`. Optional target `maxConnections` accepts the same range, cannot exceed the upstream maximum, and requires unique target authorities when enabled so origin-pool ownership remains unambiguous.
- Per-upstream `maxResponseHeaderBytes` defaults to 65,536 and accepts 8,192..1,048,576; `maxResponseHeaders` defaults to 100 and accepts 1..1,024. Unknown aliases fail compilation. These fields bound upstream response parsing and the protocol-independent decoded Header contract; they are not Nginx `proxy_buffer_size` or `proxy_buffers` compatibility fields.
- `upstreams[].loadBalancing` is a strict `round-robin`, `least-connections`, `random-two-least-connections`, or `ip-hash` enum and defaults to `round-robin`, whose runtime contract is bounded smooth weighted selection. Random-two and IP-hash are typed SDKWork forms of the corresponding Nginx strategies; directive fragments and aliases fail. IP-hash consumes the listener-resolved effective client IP, which remains the direct peer unless an explicit trusted-proxy policy succeeds. Because Nginx forbids `slow_start` with `ip_hash`, any target `slowStartMs` in an IP-hash upstream fails semantic validation. Target `weight` defaults to 1 and accepts 1..1,000. Target `backup` defaults to false, is strictly boolean, and every upstream must retain at least one non-backup primary. These fields compile into the immutable runtime target set; invalid values fail rather than degrading to equal or ambiguous selection.
- Optional target `slowStartMs` accepts 100..3,600,000 milliseconds. Omission disables health-recovery weight ramping; zero, negative, fractional, string, boolean, out-of-range, alias, and unknown values fail compilation.
- Optional active health has bounded method, origin-form URI, interval/timeout, failure/recovery thresholds, success-status range, and response Body bytes. The process-wide concurrent-check limit is finite; authority replacement, timeout greater than interval, reversed status ranges, and unknown fields fail compilation.
- Optional `deployment.resourcePressure` has finite process-memory, byte-reserve, open-handle, handle-reserve, event-loop-lag, sample-count, operations-reserve, and sampling-failure controls. Semantic validation requires reserves below ceilings, effective recovery thresholds strictly below admission thresholds after reserve truncation, and operations capacity below `maxConcurrentRequests`; the complete policy is Restart-only.
- No connector waiter queue, shared-zone/cluster connection or active-load policy, non-idempotent/body replay, hedging, arbitrary/consistent-hash or sticky load balancing, body-content matcher, custom probe-header, persisted health, or cluster-health field is accepted before a runtime implementation exists.

## Extension Points

Add configuration fields first to the root JSON Schema, then mirror them in focused models and semantic validation. Unsupported fields must not be accepted before their runtime behavior and tests exist.

## Verification

```powershell
cargo test -p sdkwork-webserver-core
```
