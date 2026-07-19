# REQ-2026-0004 PostgreSQL And SQLite Lifecycle Parity

```yaml
id: REQ-2026-0004
title: Prove PostgreSQL and SQLite database lifecycle parity for the Web control plane
owner: SDKWork maintainers
status: in-progress
source: reliability
problem: The application declares PostgreSQL and SQLite support, but a manifest declaration and static DDL validation do not prove that a fresh database can initialize, seed idempotently, remain drift-clean, and preserve SDKWork explicit-id semantics on both engines.
goals:
  - Keep PostgreSQL as the default standalone/cloud control-plane engine.
  - Keep SQLite restricted to an explicit single-node profile.
  - Execute the same contract, baseline, seed history, idempotency, and drift policy on both engines.
  - Reject SQLite rowid auto-allocation for SDKWork business identifiers.
  - Provide repeatable empty-database integration tests for both engines.
non_goals:
  - Adding a database dependency to the HTTP/HTTPS request data plane.
  - Treating SQLite as a cloud, multi-writer, or high-availability database.
  - Backup/restore, PITR, online schema migration, and failover certification; those remain parent commercial-release gates.
users:
  - Platform operators
  - Site reliability engineers
  - Backend maintainers
acceptance_criteria:
  - SQLite fresh init, standard zh-CN seed, repeated seed, and drift analysis pass in an automated Cargo test.
  - PostgreSQL fresh init, standard zh-CN seed, repeated seed, and drift analysis pass against a disposable database in CI.
  - Both engine baselines use explicitly supplied int64 business ids and produce zero error-level drift.
  - Named unique indexes and partial-index predicates match the database contract on both engines.
  - PostgreSQL/SQLite repository integration tests cover transaction rollback, unique/idempotency conflicts, tenant filters, and store-level pagination before commercial acceptance.
  - Every public repository path that reads or writes PostgreSQL JSONB or TIMESTAMPTZ values uses an engine-compatible bind/projection boundary and has representative dual-engine integration evidence.
  - Production topology rejects SQLite for shared/cloud multi-replica deployment.
non_functional_requirements:
  security: Test tooling refuses to run PostgreSQL lifecycle tests against a schema that already contains Web business tables; credentials remain environment-owned.
  privacy: Database tests use synthetic data only and do not read production rows.
  performance: P0/P1 repository queries retain bounded SQL pagination and engine-appropriate indexes.
  reliability: Seed execution is checksum-tracked and idempotent; fresh bootstrap produces a clean drift report.
affected_surfaces:
  - backend
  - composition
trace:
  specs:
    - DATABASE_SPEC.md
    - DATABASE_FRAMEWORK_SPEC.md
    - PAGINATION_SPEC.md
    - TEST_SPEC.md
  components:
    - database/database.manifest.json
    - database/ddl/baseline/postgres/0001_web_baseline.sql
    - database/ddl/baseline/sqlite/0001_web_baseline.sql
    - crates/sdkwork-webserver-database-host
verification:
  - pnpm db:validate
  - pnpm db:test:sqlite
  - SDKWORK_WEB_POSTGRES_TEST_DATABASE_URL=<disposable-url> pnpm db:test:postgres
  - pnpm verify
```

## Evidence

Evidence collected on 2026-07-19 against isolated build targets and disposable PostgreSQL 16
containers:

- SQLite lifecycle, repeat seed, explicit-id, and zero-error drift verification passes.
- PostgreSQL lifecycle, repeat seed, explicit-id, and zero-error drift verification passes.
- The shared drift matcher recognizes redundant boolean grouping and PostgreSQL
  `x = ANY (ARRAY[...])` output as equivalent to contract `x IN (...)`; positive and
  negative matcher tests pass, and drift policy contains no ignore workaround.
- SQLite and PostgreSQL repository parity pass through the real `WebRepositoryPort` for
  tenant isolation, tenant-scoped uniqueness, filtered/store-level pagination, stable
  tied-row ordering, bounded `i32::MAX` deep pages, idempotent replay/conflict,
  concurrent idempotent create, and atomic rollback on forced insert failure.
- The same dual-engine scenario now executes site update/delete, domain primary/verify/delete,
  public and encrypted environment variables, health checks, tenant/global Nginx list/update/
  activation, certificate create/finalize/renewal/failure, server token authentication and
  heartbeat, agent configuration/certificate sync, and audit insert/list paths.
- SQLx `Any` JSONB/TIMESTAMPTZ boundaries use explicit engine-compatible write casts and text
  projections. PostgreSQL BOOLEAN columns use standard `TRUE`/`FALSE`, structured JSON token
  lookup replaces serialization-dependent `LIKE`, and database instant parsing covers SQLite
  RFC3339 plus PostgreSQL text projections.
- `cargo test -p sdkwork-database-drift` passes all 24 unit/integration tests and strict
  crate Clippy passes with warnings denied.
- `cargo test -p sdkwork-intelligence-webserver-repository-sqlx` passes; the explicit
  disposable PostgreSQL parity test passes when its ignored environment gate is enabled.
- Strict component-port validation, pagination validation, database-framework validation,
  and strict repository Clippy pass.

The requirement remains `in-progress` only because its bounded-query non-functional criterion is
not yet true for every growing collection. `list_env_variables` and `list_health_checks` still
return complete matching sets, and agent sync still constructs tenant-wide configuration and
certificate vectors. The breaking public pagination and agent delta-sync contract is tracked by
[REQ-2026-0045](REQ-2026-0045-bounded-control-plane-collections.md) and requires human review before
OpenAPI/SDK/agent implementation. Database engine parity is proven; commercial bounded-collection
parity is not claimed yet.
