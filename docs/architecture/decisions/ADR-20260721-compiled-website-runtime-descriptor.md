# ADR-20260721 Compiled Website Runtime Descriptor

Status: proposed
Requirement: REQ-2026-0060
Owner: SDKWork Web Server maintainers
Date: 2026-07-21
Specs: ARCHITECTURE_DECISION_SPEC.md, SDK_SPEC.md, APP_SDK_INTEGRATION_SPEC.md,
CONFIG_SPEC.md, DEPLOYMENT_SPEC.md, NGINX_SPEC.md, SECURITY_SPEC.md, PERFORMANCE_SPEC.md,
OBSERVABILITY_SPEC.md

## Context

Cloud Sites combine multiple domains, client Variants, path Mounts, live Drive directory resources,
live Knowledgebase Wiki resources, delivery policy, and certificates. Querying normalized control
plane tables or source services to reconstruct routing on every request would couple availability,
increase latency, and make atomic rollback difficult. Persisting provider URLs/keys in a Web Server
configuration would leak source topology and create a second authority.

Web Server already has Rust HTTP/TLS, bounded ACME, certificate activation, static serving, proxy,
and atomic runtime foundations. The missing boundary is a safe compiled input and typed provider
resolution model.

## Decision

1. Web Server consumes an immutable, bounded, hash-addressed `WebsiteRuntimeDescriptor` compiled by
   Deploy from its normalized source of truth.
2. The descriptor includes Bindings, Variants, Variant rules, Resources, Mounts, delivery/security
   policy, limits, and observability policy. It contains stable IDs/references only and no secrets,
   SDK/base URLs, buckets, object keys, presigned URLs, or database connections.
3. A node validates and stages the entire descriptor, builds immutable indexes, then atomically
   swaps one current pointer. Partial maps and in-place mutation are forbidden.
4. TLS assignments use a separate immutable snapshot and atomic pointer because certificate
   rotation is operationally independent from Site configuration.
5. Ordinary Drive/Wiki content changes are provider lifecycle events. They invalidate/revalidate
   content caches but do not activate a website descriptor.
6. Content is opened only through owner-generated SDK clients or typed same-process service ports.
   The provider returns typed public eligibility/version/metadata and a bounded stream or Wiki
   representation.
7. Exact host, approved wildcard, longest Binding prefix, deterministic Variant precedence, and
   longest Mount prefix are pre-indexed and evaluated in that order.
8. Last-known-good website and TLS snapshots remain active during temporary control-plane failure.
   Desired/observed revision and served fingerprint are reported back to Deploy.
9. In cloud mode, legacy writable `web_site`, `web_domain`, `web_deployment`, and `web_certificate`
   state becomes a one-way compatibility/runtime projection or is retired through the approved
   migration. It is not a second business authority.
10. Existing ACME implementation choices remain reusable execution details. Upon acceptance, this
    decision narrows the cloud metadata/orchestration ownership portions of
    `ADR-20260623-acme-certificate-authority`; it does not discard its bounded Rust provider choice.
11. Drive `SPACE_ROOT`/`FOLDER` selection is resolved behind a stable WebsiteRoot by Drive; Web
    Server treats the reference as opaque. Mount `ROOT`/`ALIAS` remains URL translation and cannot
    retarget provider scope. Knowledgebase capability likewise never implies public access: the
    canonical WikiPublication must validate ACTIVE. One provider UUID may appear in multiple Site
    Resources, so cache/routing identity always includes the Site-local Resource and Mount.

## Alternatives

- Query Deploy/source databases per request: rejected for ownership, latency, availability, and
  tenant-isolation reasons.
- Copy normalized tables into every node and run joins: rejected because distribution and rollback
  would expose partial state and schema coupling.
- Embed origin URLs/secrets in descriptors: rejected because descriptors are distributed metadata
  and provider topology must remain private.
- Rebuild a frozen release for each content update: rejected because live directory/Wiki semantics
  and WYSIWYG freshness are product requirements.
- Let each provider register arbitrary executable handlers: rejected because runtime extension must
  remain typed, bounded, reviewed, and deterministic.

## Consequences

- A versioned descriptor schema, canonical serializer, compiler compatibility matrix, rollout
  protocol, and golden tests are required.
- Drive and Knowledgebase need stable provider service contracts and events.
- Web Server needs provider adapters, cache generation/invalidation, routing indexes, and per-resource
  circuit/timeout policy.
- The fleet can continue serving when Deploy is unavailable, but provider availability and stale
  policy become explicit SLO inputs.
- Web Server cloud management APIs/tables need migration rather than dual ownership.

## Verification

- deterministic compiler/consumer golden fixtures and version compatibility tests;
- schema/hash/signature/size/reference failure tests;
- routing property/fuzz and Nginx-profile conformance tests;
- provider eligibility/path/state/version/event contract tests;
- atomic website/TLS activation, last-known-good, drift, and rollback tests;
- cross-tenant cache/origin/security tests and bounded load/soak evidence.

## Supersedes / Superseded By

On acceptance, this ADR supersedes only the cloud control-plane ownership assumptions of older Web
Server management designs. Existing protocol safety and ACME runtime library decisions remain in
force unless a separate ADR replaces them.
