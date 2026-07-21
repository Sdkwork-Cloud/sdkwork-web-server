# REQ-2026-0060 Cloud Site Delivery Data Plane

```yaml
id: REQ-2026-0060
title: Deliver live Drive directory and Knowledgebase Wiki Sites from compiled runtime descriptors
owner: SDKWork Web Server maintainers
status: ready
source: platform
problem: The Web Server needs a bounded execution contract for cloud Sites without becoming a second writable publishing authority or requiring a release for every source change.
goals:
  - atomically consume website and TLS runtime snapshots
  - route domains, paths, Variants, and Mounts deterministically
  - stream eligible public content through typed Drive and Knowledgebase provider ports
  - preserve freshness, tenant isolation, last-known-good service, and commercial telemetry
non_goals:
  - own Site/domain/certificate business metadata in cloud mode
  - build or release source content per update
  - access source databases or object stores directly
affected_surfaces:
  - webserver-edge-runtime
  - webserver-acme-service
  - webserver-certificate-worker
  - runtime-bootstrap
  - observability
  - deployment
```

Specs: REQUIREMENTS_SPEC.md, ARCHITECTURE_DECISION_SPEC.md, API_SPEC.md, SDK_SPEC.md,
APP_SDK_INTEGRATION_SPEC.md, CONFIG_SPEC.md, DEPLOYMENT_SPEC.md, NGINX_SPEC.md,
SECURITY_SPEC.md, PRIVACY_SPEC.md, PERFORMANCE_SPEC.md, OBSERVABILITY_SPEC.md, TEST_SPEC.md

## Requirements

1. Validate and atomically activate a complete `WebsiteRuntimeDescriptor` with deterministic
   canonical hash and bounded indexes.
2. Validate and activate a separate node-scoped TLS snapshot; website and certificate changes shall
   not create revisions in each other's lifecycle.
3. Resolve exact/wildcard Host, longest Binding path, Variant precedence, and longest Mount path
   without scanning all tenants or Sites on the request path.
4. Implement STATIC, SPA, and WIKI handlers with root confinement, safe index/fallback behavior,
   streaming, conditional/range support, MIME and security policy.
5. Resolve content only through injected Drive/Knowledgebase generated SDK clients or typed service
   ports. Do not use raw HTTP/manual auth, provider DBs, arbitrary filesystem paths, or object keys.
6. Consume idempotent provider change events and use read-through revalidation so ordinary content
   changes do not require descriptor activation.
7. Prevent stale public cache from bypassing private/deleted/revoked transitions.
8. Reuse bounded ACME and certificate runtime primitives for typed Deploy jobs and report actual
   served observations.
9. Continue last-known-good snapshots during temporary control-plane failure and fail readiness when
   no valid assigned state exists.
10. Emit bounded observability and deduplicated usage facts without logging content or secrets.
11. Treat a Drive WebsiteRoot as an opaque provider resource whether its owner selected
    `SPACE_ROOT` or `FOLDER`. Mount `ROOT`/`ALIAS` translates URL paths only and shall never retarget
    the Drive source root from descriptor diagnostics.
12. Treat every Knowledgebase as potentially Wiki-capable but require provider validation of its one
    canonical WikiPublication and `ACTIVE` status on every activation/revalidation path. Multiple
    Site Resources referencing one provider UUID remain isolated by the full runtime/cache identity.
13. Declare and consume owner-generated `sdkwork-drive-internal-sdk` and
    `sdkwork-knowledgebase-internal-sdk` dependencies plus their AsyncAPI event authorities.
    Knowledgebase provider events flow directly to Web Server; Deploy is not a per-content-event
    relay. Provider endpoint/credentials are runtime configuration and never descriptor fields.
14. Retire writable Web Site/Domain/Deployment/Certificate app-api routes and `web_*` business
    authority after the approved Deploy single-writer cutover. Web persistence is limited to
    immutable snapshots, checkpoints, bounded cache metadata, observations, audit and usage spool.
15. Separate provider-wide generation, route page/static content version, navigation/search
    generation, and Deploy SiteRevision policy generation. Wiki private processing must not advance
    a public cache version or flush unrelated routes.

## Acceptance Criteria

- Descriptor schema/hash/signature/limit/referential checks and atomic activation tests pass.
- Host/SNI/path/IDNA/wildcard/redirect/Variant/Mount routing property tests pass.
- STATIC/SPA/WIKI provider contract and browser-to-resource E2E tests pass.
- Space-root/folder-root, reserved-root, Mount-vs-provider-root separation, canonical Wiki
  publication, inactive Wiki, and multi-Site provider reuse tests pass.
- Event replay/gap/out-of-order, public-to-private, negative-cache, stampede, and provider outage
  tests pass.
- Exact Drive/Knowledgebase AsyncAPI and generated internal SDK compatibility tests pass in
  standalone and cloud topology, including provider generation, route page version, move and
  priority revocation behavior.
- A single-writer migration test proves Web control-plane write routes/tables are non-authoritative
  and normal rollback cannot reactivate dual writers.
- TLS challenge/renewal/distribution/hot-load/SNI/last-valid tests pass with website snapshot
  independence.
- Tenant-qualified cache and provider authorization tests prove no cross-tenant disclosure.
- Load and soak evidence proves bounded memory, concurrency, queues, caches, rendering, descriptor
  count, certificate count, and telemetry labels.
- Last-known-good, rollout quorum, node drift, drain, rollback, and restart recovery drills pass.

## Trace

- PRD: `docs/product/prd/PRD-cloud-site-delivery-data-plane.md`
- Decision: `docs/architecture/decisions/ADR-20260721-compiled-website-runtime-descriptor.md`
- Architecture: `docs/architecture/tech/TECH-cloud-site-delivery-data-plane.md`
- Cross-repository authority: `sdkwork-deployments` REQ-2026-0001 and
  ADR-20260721-unified-cloud-site-publishing-control-plane
