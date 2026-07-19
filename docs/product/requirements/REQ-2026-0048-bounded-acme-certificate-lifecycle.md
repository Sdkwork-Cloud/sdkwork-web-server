# REQ-2026-0048 Bounded ACME And Certificate Lifecycle

```yaml
id: REQ-2026-0048
title: Make ACME issuance and certificate activation bounded, truthful, and fail-closed
owner: sdkwork-web-server
status: accepted
source: acme-certificate-commercial-readiness
problem: The ACME adapter reads process environment directly on the active bootstrap path, leaves HTTP-01 challenges in memory and on disk, has no whole-operation deadline, reports certificate metadata that can differ from the actual leaf certificate, fingerprints PEM text instead of leaf DER, and writes the certificate and private key independently. These behaviors can leak resources, accept unsafe paths, report false evidence, or expose a mismatched live bundle after partial failure.
goals:
  - Inject validated typed ACME configuration from the deployable runtime bootstrap while preserving legacy Rust compatibility entrypoints.
  - Bound directory URL, contact email, renewal window, webroot, secret material, operation duration, provider response bytes, active challenges, token bytes, key-authorization bytes, authorization count, PEM bytes, and certificate names.
  - Remove HTTP-01 memory and filesystem state on success, provider failure, timeout, or future cancellation.
  - Derive validity and SHA-256 fingerprint from the actual leaf X.509 certificate DER.
  - Stage certificate and private-key material together and restore the prior bundle when activation fails.
  - Keep blocking certificate filesystem activation off async service executor threads.
non_goals:
  - Claiming real public Let's Encrypt issuance without an externally reachable HTTP-01 environment and provider evidence.
  - Persisting ACME account credentials, adding DNS-01/wildcard issuance, or introducing a database migration.
  - Cross-process locking, versioned current-symlink activation, database/filesystem distributed transactions, or multi-node served-fingerprint convergence.
  - Certificate revocation, OCSP stapling, delegated credentials, HSM/KMS integration, or automated trust-store distribution.
users:
  - operators issuing and renewing tenant HTTPS certificates
  - edge agents materializing certificate bundles
acceptance_criteria:
  - Active runtime bootstrap constructs and injects AcmeConfig and CertificateIssuer; the shared ACME crate's legacy from_env APIs are not used by the active path.
  - Production-like means test, staging, or production; those profiles require explicit contact email and certificate encryption key and cannot use development fallback values.
  - ACME directory URLs use HTTPS, contact email and webroot are syntactically bounded, renewal is 1..90 days, derived encryption keys are exactly 32 bytes, and total issuance timeout is configurable from 10000..600000 ms with a 180000 ms default.
  - Every ACME HTTP response body is streamed through an application-owned 2-MiB ceiling; oversized Content-Length, size hints, arithmetic overflow, and streamed excess fail before unbounded collection.
  - One CertificateIssuer admits at most eight concurrent issuance operations with try-acquire semantics and no waiter queue; capacity exhaustion fails immediately and permit release restores capacity.
  - At most 64 challenges exist in process; tokens are 1..256 base64url bytes, key authorization is 1..2048 safe ASCII bytes, and at most 8 authorizations are processed for one order.
  - Challenge files are atomically staged below .well-known/acme-challenge, path traversal is rejected, and an RAII lease removes memory/file state on every completion or cancellation path.
  - Self-signed certificate parameters explicitly set validity; returned notBefore/notAfter are RFC3339 values parsed from the actual leaf certificate.
  - ACME and self-signed fingerprints are lowercase SHA-256 of leaf DER, not PEM or the complete chain text.
  - Certificate names and PEM material are bounded and parsed before mutation; the chain contains certificates only, the key contains exactly one supported private key, and Rustls verifies the leaf/key pair before the bundle is staged.
  - Replacement activation restores the previous bundle when the staged generation cannot be activated and leaves no shared fixed temporary path.
  - Certificate activation has one non-queuing permit per EdgeRuntime before spawn_blocking and a fixed-memory process-wide eight-operation atomic limit; exhausted capacity fails immediately instead of retaining certificate/private-key copies in an unbounded blocking-task queue.
  - Unit tests parse X.509 evidence, prove challenge bounds and cleanup, reject path traversal/oversize material, prove replacement, and exercise rollback.
non_functional_requirements:
  security: No private key, challenge authorization, provider payload, or raw secret is logged; path-derived values remain confined to configured roots; production-like fallback secrets fail closed.
  performance: One issuer has at most eight concurrent operations with no waiter queue, each edge runtime has one non-queuing activation operation, the process has at most eight direct activations, challenge memory is O(64 * bounded-entry-size), one issuance has one finite total deadline and at most eight challenge leases, every provider response is at most 2 MiB, and certificate material has fixed byte ceilings.
  reliability: Cancellation releases challenge state, failed validation does not mutate the active bundle, and failed activation restores the prior complete bundle.
affected_surfaces:
  - webserver-acme-service
  - webserver-edge-runtime
  - webserver-runtime-bootstrap
  - webserver-business-service
trace:
  specs:
    - CONFIG_SPEC.md
    - SECURITY_SPEC.md
    - PERFORMANCE_SPEC.md
    - DEPLOYMENT_SPEC.md
    - RUST_CODE_SPEC.md
    - TEST_SPEC.md
  components:
    - crates/sdkwork-webserver-acme-service
    - crates/sdkwork-webserver-edge-runtime
    - crates/sdkwork-intelligence-webserver-repository-sqlx
    - crates/sdkwork-intelligence-webserver-service
verification:
  - cargo test -p sdkwork-webserver-acme-service
  - cargo test -p sdkwork-webserver-edge-runtime
  - cargo clippy -p sdkwork-webserver-acme-service --all-targets -- -D warnings
  - cargo clippy -p sdkwork-webserver-edge-runtime --all-targets -- -D warnings
  - cargo clippy -p sdkwork-intelligence-webserver-repository-sqlx --all-targets -- -D warnings
  - cargo fmt --all -- --check
  - git diff --check
  - pnpm.cmd verify
```

## Compatibility Boundary

The public `AcmeConfig::from_env` and `CertificateIssuer::from_env` Rust entrypoints remain available for compatibility, but the application bootstrap no longer uses them. New callers should construct `AcmeConfig` from typed runtime values and inject it through `CertificateIssuer::new` or `CertificateIssuer::new_with_operation_timeout_ms`. Invalid values that were previously defaulted or accepted now fail before provider or filesystem work because accepting an unsafe secret, path, timeout, or provider identity is not a valid compatibility behavior.

No OpenAPI or database schema changes are introduced. ACME account credential persistence and a durable desired/observed certificate activation state machine require separate human-reviewed contracts.

## Evidence Boundary

This requirement accepts deterministic local implementation and test evidence only. It does not convert [REQ-2026-0002](REQ-2026-0002-instant-acme-letsencrypt.md) to accepted because that requirement needs a real externally reachable HTTP-01 staging issuance and database lifecycle evidence. A public-CA certificate, provider rate-limit behavior, revocation, and renewal under real DNS/network conditions must be proven in a controlled staging environment before commercial GA.

## Implementation Evidence

- The active Repository runtime bootstrap now parses environment/profile overrides, applies the canonical test/staging/production security boundary, constructs `AcmeConfig`, and injects `CertificateIssuer`. The compatibility `from_env` methods remain available but are absent from the active application call path.
- `AcmeConfig` requires an HTTPS directory without userinfo, a bounded ASCII contact, a 1..90 day renewal window, a bounded webroot, and a 32-byte HKDF-derived encryption key. Production ACME profile selection is itself production-like even when the surrounding application environment is development.
- One issuer uses an eight-permit non-queuing semaphore. Each ACME provider response uses the platform TLS verifier and an application-owned streaming body adapter capped at 2 MiB; size hints, declared length, arithmetic overflow, and actual frames are checked before growth.
- HTTP-01 state reserves one of 64 bounded entries without holding a lock across async I/O. A generation-bound lease owns the memory entry and atomically staged webroot file, and cleanup is serialized against reuse of the same token so an old lease cannot delete a newer challenge.
- The whole account/order/authorization/challenge/finalize/certificate flow runs under one 10000..600000 ms deadline. At most eight authorizations and challenge leases are retained for an order.
- Self-signed validity is assigned to rcgen parameters and then read back from the generated X.509 leaf. Both provider and self-signed paths store RFC3339 validity and SHA-256 of leaf DER.
- Certificate activation rejects unsafe names, over-1-MiB chains, over-128-KiB keys, non-certificate chain items, multiple/non-key key items, and leaf/private-key mismatch through Rustls `CertifiedKey`. It admits one task per runtime before `spawn_blocking`, uses a fixed process-wide eight-operation atomic limit without a waiter queue, stages and syncs both files, applies Unix `0600` before activation, restores the prior directory after injected activation failure, and retains a backup path if restoration itself fails.
- Business-service activation encrypts the private key before filesystem mutation and runs blocking activation in `spawn_blocking`. New certificate bundles use the globally unique certificate record id instead of a lossy hostname transformation.

## Verification Evidence

- `cargo test -p sdkwork-webserver-acme-service` passes 13/13 tests covering typed bounds, encryption, X.509 evidence, response-body limits, issuance admission, challenge capacity/path/failure cleanup, and scoped cleanup.
- `cargo test -p sdkwork-webserver-edge-runtime` passes 11/11 tests covering non-queuing async activation admission, real certificate/key parsing, mismatch/size/path rejection, complete replacement, injected rollback, Nginx candidate validation, failure propagation, and child timeout.
- Strict all-target Clippy passes with `-D warnings` for ACME service, edge runtime, business service, and SQLx Repository.
- Strict component-port binding, application-layering, route-collision, and repository-document validators pass.
- `pnpm.cmd verify` passes the complete Rust workspace, 19 Node contract tests, API materialization consistency, repository composition, API envelope, topology, database framework, and cloud gateway validation. Its environment-gated PostgreSQL Repository and lifecycle tests remain ignored without an explicit disposable PostgreSQL URL; prior dual-engine evidence remains owned by REQ-2026-0004.
- `cargo fmt --all -- --check` and `git diff --check` pass.

## Remaining Boundary

The current `instant-acme` account credentials are still ephemeral per issuance. Repeated account creation can increase provider/rate-limit risk; encrypted account persistence, rotation, and recovery need an approved storage contract. Directory replacement prevents a partial certificate/private-key pair but is not a cross-process linearizable activation primitive. Multi-process agents and Nginx workers still need versioned bundle generations, a stable `current` reference, process coordination, served-fingerprint acknowledgement, rollback policy, and cluster reconciliation before high-availability commercial release.
