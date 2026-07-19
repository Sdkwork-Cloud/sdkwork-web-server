# sdkwork-intelligence-webserver-repository-sqlx

Domain: platform
Capability: webserver-persistence
Package type: Rust SQLx repository crate
Status: active

## Public API

The crate exports `WebRepository`, `WebRuntime`, and
`bootstrap_web_runtime_from_env`. `WebRepository` implements the service-owned
`WebRepositoryPort`; SQL and row mapping remain private repository concerns.

## Required SDK Surface

No generated HTTP SDK is consumed. The repository depends on service ports,
SDKWork database/ID utilities, and approved certificate/edge provider ports.

## Configuration

Runtime bootstrap resolves the typed Web database profile, Snowflake node id,
and protected secret-encryption key. Production fails closed when required ID or
encryption configuration is absent.

## Deployment Profile And Runtime Target Behavior

The same repository contract runs against PostgreSQL and single-node SQLite.
List operations bind finite `LIMIT` and `OFFSET` values in SQL; no repository
list method collects an unbounded table and slices in memory.

## Security

Tenant-scoped methods bind `tenant_id` before rows are returned. Environment
variable values are encrypted before storage, and database errors are mapped at
the repository boundary without exposing query data through public contracts.

## Extension Points

Add persistence behavior through focused repository modules and the
service-owned port. Do not move SQL into handlers or route crates.

## Verification

```powershell
cargo test -p sdkwork-intelligence-webserver-repository-sqlx
node ..\sdkwork-specs\tools\check-pagination.mjs --workspace .
```
