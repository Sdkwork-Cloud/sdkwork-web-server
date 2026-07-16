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

## Extension Points

Add configuration fields first to the root JSON Schema, then mirror them in focused models and semantic validation. Unsupported fields must not be accepted before their runtime behavior and tests exist.

## Verification

```powershell
cargo test -p sdkwork-webserver-core
```
