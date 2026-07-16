# ADR-20260716 Canonical URI Dual Representation

Status: proposed
Requirement: REQ-2026-0018
Owner: SDKWork maintainers
Date: 2026-07-16
Specs: REQUIREMENTS_SPEC.md, ARCHITECTURE_DECISION_SPEC.md, SECURITY_SPEC.md, NGINX_SPEC.md, TEST_SPEC.md

## Context

Nginx distinguishes `$request_uri`, which preserves the original request target and Query, from `$uri`, which is percent-decoded and normalized for location processing. SDKWork previously bounded and validated URI components but matched routes against the raw Path. That difference affects encoded separators, dot segments, repeated slashes, static filesystem identity, rewrite behavior, upstream URI selection, and cache keys.

A real local Nginx 1.26.2 probe shows `/a/../b` and `/a/%2e%2e/b` normalize to `/b`, `//a///b` to `/a/b`, `%2F` to `/`, and a traversal above root returns `400`. Windows Nginx also treats decoded backslash as a separator. SDKWork currently rejects decoded backslash as a request-smuggling and cross-platform filesystem hardening rule.

## Decision

- Preserve the original request Path and Query as the raw representation.
- Build one bounded canonical Path by performing exactly one percent-decoding pass, rejecting NUL/control/backslash and invalid UTF-8, merging repeated slashes, resolving `.` and `..`, and rejecting traversal above root.
- Route selection and static filesystem mapping consume only the canonical Path.
- Authored route paths are canonical Path values, not raw URI strings. Compilation rejects repeated slashes and dot segments but does not percent-decode them again; decoded `?`, `#`, and `%` remain ordinary Path data and can be matched exactly.
- Reverse proxy without URI rewrite (`stripPrefix=false`) preserves the original Path and Query.
- Reverse proxy with URI rewrite (`stripPrefix=true`) strips the canonical route prefix from the canonical Path, safely URL-encodes it, and preserves the original Query.
- Query parsing and normalization remain outside this decision; REQ-2026-0017 budgets and validates Query representation only.
- Decoded backslash remains a documented security hardening difference from the tested Windows Nginx build.

## Alternatives

- Match and proxy only the raw Path: rejected because encoded separator and dot-segment behavior diverges from Nginx location semantics and can create adapter disagreement.
- Replace the request URI globally with the canonical form: rejected because it destroys `$request_uri`-equivalent evidence and breaks proxy modes that must preserve the original request target.
- Reproduce Windows Nginx backslash normalization: rejected because it creates platform-dependent filesystem and routing identity and weakens the existing cross-platform fail-closed policy.
- Delegate normalization independently to static and proxy adapters: rejected because multiple decoding phases create traversal and request-smuggling risk.

## Consequences

- Canonical Path allocation is bounded by `maxDecodedPathBytes`; segment stack allocation is bounded by `maxPathSegments`.
- Route behavior changes for encoded slash, encoded/literal dot segments, and repeated slash inputs. This is compatibility-visible and requires human review before acceptance.
- Invalid UTF-8 and decoded backslash remain intentional security differences from some Nginx builds.
- Future rewrite, cache-key, logging, and observability work must name raw versus canonical URI explicitly.

## Verification

- Versioned Nginx 1.26.2 probe fixture and recorded comparison matrix in REQ-2026-0018.
- Core unit tests for decoding, merge, dot resolution, above-root rejection, bounds, and canonical route config.
- Real H1/TLS-H2 route tests for raw/canonical differences and Stream recovery.
- Static filesystem and reverse-proxy rewrite tests proving one canonical identity and preserved Query.
- Full workspace tests, Clippy, formatting, SDKWork validators, and documentation checks.

## Supersedes / Superseded By

This decision narrows the URI portion of ADR-20260715 without superseding its data-plane boundaries. It remains proposed until human review accepts the compatibility and security differences.
