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

Current evidence on 2026-07-15: SQLite lifecycle and drift are verified and automated. PostgreSQL test code compiles, but execution remains pending because this workstation has neither an explicit disposable PostgreSQL URL nor an available Docker service. The requirement remains `in-progress` until PostgreSQL and repository transaction/parity evidence pass.
