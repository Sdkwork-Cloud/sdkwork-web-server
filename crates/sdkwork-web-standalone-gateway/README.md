# sdkwork-web-standalone-gateway

Domain: platform
Capability: webserver
Package type: Rust standalone gateway
Status: active

## Public API

- `build_router`: existing management app-api/backend-api composition.
- `run_database_migrate_only`: database migration operation.
- `run_data_plane_until`: database-independent HTTP/HTTPS application data plane with explicit shutdown.
- Binary operations: `serve-management`, `db-migrate`, `validate`, and `data-plane`.

## Required SDK Surface

This runtime does not consume generated HTTP SDKs. It mounts repository-owned management route crates and executes application Web Server traffic from the compiled config port.

## Configuration

Management mode uses SDKWork typed server/database environment configuration. `validate` and `data-plane` accept an explicit config argument or `SDKWORK_WEB_SERVER_CONFIG_FILE`.

## Deployment Profile And Runtime Target Behavior

The current executable is the standalone server gateway. The data-plane-only operation does not initialize PostgreSQL or SQLite. Cloud node-scoped snapshot consumption remains a separate requirement.

## Security

- Rustls provides TLS 1.2/1.3 and configured ALPN.
- Connection permits are acquired through make-service readiness before per-connection tasks are created.
- Static traversal/symlink escape checks run before `tower-http` file service.
- Proxy bodies are streamed with a counted hard limit; redirects and protocol upgrades are not followed silently.
- Hop-by-hop headers and untrusted forwarding identity are removed/replaced.

## Extension Points

New route actions and listener capabilities must first exist in the root schema/core compiler, then gain a focused adapter and real integration tests. Business API behavior remains in route/service/repository crates.

## Verification

```powershell
cargo test -p sdkwork-web-standalone-gateway
cargo run -p sdkwork-web-standalone-gateway -- validate configs/examples/sdkwork.webserver.config.json
```

