# sdkwork-webserver-core

Domain: platform
Capability: webserver-config
Package type: Rust crate
Status: active

## Public API

The crate exports the `sdkwork.webserver.app` Serde model, bounded file loader, schema/semantic diagnostics, immutable compiled indexes, resource lookups, and deterministic listener/host/route selection.

It does not bind sockets, execute HTTP, access SQLx, or own management API contracts.

## Configuration

`specs/sdkwork.webserver.config.schema.json` at the repository root is the machine authority. Loading is limited to 1 MiB, rejects unknown fields, and requires all semantic references and resource paths to validate before compilation.

## Deployment Profile And Runtime Target Behavior

The same compiler is used by standalone and future cloud data planes. It consumes local authored or published configuration and has no database bootstrap dependency.

## Security

- Static roots are relative to the configuration directory and cannot escape it.
- Certificate keys remain protected file references; bytes are not part of the model.
- Unsupported resolver, TLS, routing, or Nginx behavior fails explicitly.
- Route and host indexes are immutable after compile.

## Extension Points

Add configuration fields first to the root JSON Schema, then mirror them in focused models and semantic validation. Unsupported fields must not be accepted before their runtime behavior and tests exist.

## Verification

```powershell
cargo test -p sdkwork-webserver-core
```

