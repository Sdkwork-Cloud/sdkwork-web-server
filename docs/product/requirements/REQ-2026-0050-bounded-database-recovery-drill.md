# REQ-2026-0050 Bounded Database Recovery Drill

```yaml
id: REQ-2026-0050
title: Prove bounded real SQLite and PostgreSQL backup restoration
owner: sdkwork-web-server
status: accepted
source: database-commercial-readiness
problem: Database lifecycle and Repository parity are verified, but the application has no executable evidence that either supported engine can restore a consistent schema and tenant-scoped business row from a real backup artifact.
goals:
  - Exercise a transactionally consistent SQLite backup and reopen the backup as an independent restored database.
  - Exercise PostgreSQL custom-format pg_dump and pg_restore against a fresh disposable PostgreSQL 16.9 instance.
  - Prove the restored generation is isolated from source writes that occur after the backup.
  - Bound the complete operation, readiness wait, subprocess duration, captured output, baseline bytes, backup bytes, process concurrency, and disposable resources.
  - Make the recovery drill mandatory in shared merge and release validation.
non_goals:
  - A production backup scheduler, retention service, object-store uploader, or encryption/KMS implementation.
  - PostgreSQL WAL archiving, PITR, replication, failover, managed-provider certification, or multi-region disaster recovery.
  - Claiming the product RPO/RTO or the parent production-operations PRD is complete.
  - Restoring or mutating any developer-owned or production database.
acceptance_criteria:
  - SQLite uses the SDKWork database adapter, a one-connection temporary file database, a full WAL checkpoint, bounded page-size estimation, and VACUUM INTO rather than an unsafe live file copy.
  - The SQLite source is changed after backup; the reopened backup passes integrity_check, contains the original tenant-scoped canary, and contains the expected Web table set.
  - PostgreSQL uses the same digest-pinned image as mandatory parity CI, one uniquely named --rm container, no host port, no volume, no container network, and test-only credentials.
  - PostgreSQL applies the tracked baseline, inserts a tenant-scoped canary, creates a custom-format no-owner/no-ACL dump, computes a SHA-256 checksum, mutates the source, restores into a distinct empty database, and observes the pre-mutation canary.
  - The entire drill is capped at ten minutes, readiness at sixty seconds, captured output at 64 KiB, baseline input at 2 MiB, backup artifacts at 64 MiB, and container concurrency at one; PostgreSQL is limited to 256 MiB memory, one CPU, 256 PIDs, and finite data/tmp tmpfs mounts.
  - Cleanup force-removes the disposable container in a finally path and temporary SQLite files are owned by a scoped temporary directory.
  - Root and shared workflow validation invoke the drill, and contract tests pin the real commands and bounds.
non_functional_requirements:
  security: No production URL or credential is accepted; the PostgreSQL port is not published, artifacts stay inside disposable storage, and logs contain no business payload or production secret.
  reliability: Restore verification checks independent generation data, schema completeness, SQLite integrity, PostgreSQL command failure, dump checksum shape, and cleanup.
  performance: Work is sequential, backup sizes and container memory/CPU/PIDs/tmpfs are finite, subprocesses are bounded, and no captured output can grow beyond 64 KiB.
affected_surfaces:
  - database-recovery-evidence
  - sqlite-standalone-verification
  - postgresql-release-verification
  - release-workflow-contract
trace:
  specs:
    - DATABASE_SPEC.md
    - DATABASE_FRAMEWORK_SPEC.md
    - TEST_SPEC.md
    - SECURITY_SPEC.md
    - PNPM_SCRIPT_SPEC.md
  components:
    - crates/sdkwork-webserver-database-host
    - scripts/database-recovery-verify.mjs
    - sdkwork.workflow.json
verification:
  - cargo test -p sdkwork-webserver-database-host --test sqlite_recovery sqlite_consistent_backup_restores_integrity_and_tenant_data -- --exact --nocapture
  - node --test tests/contract/database-recovery.contract.test.mjs
  - pnpm test:database:recovery
  - node ../sdkwork-github-workflow/scripts/sdkwork-workflow.mjs validate --config sdkwork.workflow.json
  - node ../sdkwork-specs/tools/check-pnpm-script-standard.mjs --root . --product-prefix web
  - pnpm verify
  - git diff --check
```

## Evidence Boundary

This requirement proves that application-owned schema and a tenant-scoped canary can be restored from real SQLite and PostgreSQL backup mechanisms in disposable local infrastructure. It does not prove scheduled production backup completion, encryption, immutable off-host retention, PostgreSQL WAL continuity, PITR, managed-provider compatibility, cross-region recovery, or any declared production RPO/RTO.

## Implementation Evidence

- `sqlite_recovery.rs` uses the approved SDKWork database pool and lifecycle orchestrator, enforces one connection and a 64-MiB page estimate, checkpoints WAL, writes an independent `VACUUM INTO` artifact, mutates the source, and verifies restored integrity, tenant data, and schema completeness.
- `database-recovery-verify.mjs` owns a single digest-pinned disposable PostgreSQL container without a published port, network, or volume and with finite memory, CPU, PID, data, and temporary-storage budgets. It applies the tracked bounded baseline and runs real `pg_dump`, `sha256sum`, `createdb`, `pg_restore`, and fail-closed SQL verification.
- The runner caps total duration, child duration, readiness, captured diagnostics, source baseline size, backup artifact size, and container count, and unconditionally attempts bounded container cleanup even when startup itself fails or times out.
- Root contract tests and `sdkwork.workflow.json` make the drill a non-optional merge/release check after lifecycle and Repository parity.

## Verification Evidence

- `pnpm test:database:recovery` passes against real SQLite and the digest-pinned PostgreSQL 16.9 Alpine image. SQLite restores its original tenant canary and passes `integrity_check`; PostgreSQL restores 10 Web tables and the pre-divergence canary from a 46,857-byte SHA-256-checksummed custom-format dump.
- The successful PostgreSQL run uses no published port or container network and passes with 256 MiB memory, one CPU, 256 PIDs, 128-MiB data tmpfs, and 80-MiB temporary tmpfs limits.
- `node --test tests/contract/database-recovery.contract.test.mjs` passes 2/2 and pins both real engine mechanisms, user selection, Alpine-compatible tooling, stdin semantics, resource bounds, cleanup, and mandatory workflow placement.
- `pnpm verify` passes with the new SQLite recovery test and 23 Node contract tests, followed by API materialization, SDKWork repository/agent/pnpm/database/topology checks, and cloud gateway validation.
- Shared workflow validation, pnpm-script validation, Rust formatting, and `git diff --check` pass. No `sdkwork-web-recovery-*` container remains after successful or failed runs.
