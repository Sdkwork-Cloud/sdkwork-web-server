# SDKWork Web Server Application Configuration PRD

Status: active
Owner: SDKWork maintainers
Application: sdkwork-web
Updated: 2026-07-15
Parent: [PRD.md](PRD.md)
Specs: APP_MANIFEST_SPEC.md, APPLICATION_SPEC.md, CONFIG_SPEC.md, ENVIRONMENT_SPEC.md, NGINX_SPEC.md, SECURITY_SPEC.md

## 1. Purpose

Define the complete application-owned Web Server configuration contract for Web applications under `apps/<app-root>/`. The contract must cover common Nginx-class Web Server behavior while remaining deterministic, portable, safe, versioned, and executable by the Rust data plane.

## 2. Source Of Truth And File Layout

Every independently hosted Web application root must contain:

```text
apps/<app-root>/
  sdkwork.app.config.json
  config/
    webserver/
      sdkwork.webserver.config.json
```

- `sdkwork.app.config.json` remains the application identity, release, platform, media, artifact, and publication authority.
- `sdkwork.webserver.config.json` is the authored traffic-serving authority.
- Generated Nginx files, compiled Rust snapshots, runtime secrets, live certificates, node state, logs, and databases must not be committed into the app root.
- The app manifest references the Web Server contract through a standardized `runtime.webServer` object. Adding this object requires coordinated changes to `APP_MANIFEST_SPEC.md`, its JSON Schema, validator, initializer, full example, and projection tooling.

Required manifest reference:

```json
{
  "runtime": {
    "webServer": {
      "enabled": true,
      "configRef": "config/webserver/sdkwork.webserver.config.json",
      "defaultProfile": "static-spa"
    }
  }
}
```

## 3. Top-Level Contract

```json
{
  "schemaVersion": 1,
  "kind": "sdkwork.webserver.app",
  "appKey": "sdkwork-example-pc",
  "compatibility": {},
  "profiles": {},
  "listeners": [],
  "certificates": [],
  "resolvers": [],
  "resources": [],
  "upstreams": [],
  "virtualHosts": [],
  "policies": {},
  "observability": {},
  "deployment": {},
  "metadata": {}
}
```

Rules:

- Unknown fields are rejected unless explicitly allowed inside a bounded metadata extension object.
- IDs are stable lower-kebab-case identifiers unique within the application configuration.
- References must resolve inside the same compiled application configuration or through an approved external resource reference type.
- Secrets and secret values are forbidden. Only secret, KMS, certificate, Drive artifact, or discovery resource references are allowed.
- Configuration must declare explicit limits; an omitted limit resolves to a documented bounded default, never infinity.

This app-owned contract is not the server process configuration. Process binds, service account, worker/runtime sizing, runtime directories, administrative listener, global emergency reserve, and platform secret providers belong to the typed server runtime configuration governed by `CONFIG_SPEC.md` and `RUNTIME_DIRECTORY_SPEC.md`. The compiler combines app contracts, deployment overlays, resolved resources, and host runtime policy into one immutable node snapshot without allowing an application to weaken host-level limits.

## 4. Compatibility

`compatibility` declares the intended Nginx compatibility behavior:

| Field | Requirement |
| --- | --- |
| `nginxProfile` | Supported profile such as `http-core-v1`. |
| `unknownDirectivePolicy` | Must be `error` for Rust activation; `preserve` may be used only by an Nginx round-trip tool. |
| `regexEngine` | `pcre2` when Nginx-compatible regex behavior is required. |
| `variableProfile` | Declared subset of supported Nginx variables. |
| `renderTarget` | Optional Nginx target version/profile used for generated configuration. |

## 5. Default Profiles

The product must provide five complete profiles:

| Profile | Default behavior |
| --- | --- |
| `static-site` | Static files, index documents, conditional/range requests, MIME, safe traversal, bounded caching. |
| `static-spa` | Static site behavior plus client-route fallback to `index.html` and immutable fingerprinted asset caching. |
| `reverse-proxy` | One or more upstreams, forwarding headers, streaming, WebSocket, timeouts, health checks, and bounded retries. |
| `hybrid` | Static assets and SPA shell with selected API paths routed to upstreams. |
| `api-gateway` | Route-based upstream selection, streaming, authentication extension points, limits, audit, and high-volume observability. |

Development defaults bind to `127.0.0.1:8080`. Container defaults bind to `0.0.0.0:8080`. Public 80/443 listeners are created only by an explicit public-edge profile or deployment overlay.

## 6. Listener Contract

Each listener is an application-logical listener. The deployment compiler maps compatible logical listeners from multiple applications onto physical node sockets and proves that their protocol, TLS, Proxy Protocol, and default-host policies can coexist. Each listener supports:

- Stable `id`, bind address, port, socket family, and default-server selection.
- HTTP, HTTPS, HTTP/1.1, HTTP/2, and future-gated HTTP/3 protocol declarations.
- `reusePort`, accept backlog, keep-alive, header timeout, request timeout, idle timeout, graceful drain, and maximum connection limits.
- Optional Proxy Protocol with explicit trusted source networks.
- Optional TLS policy reference; HTTPS listeners require one.
- Platform-aware binding overlays without changing the app-owned logical listener id.

Validation must reject duplicate socket ownership, invalid ports, unsupported protocol combinations, unsafe wildcard exposure, missing TLS dependencies, and multiple defaults on the same effective address.

Host policy may narrow bind addresses, ports, protocols, connection budgets, or public exposure. An app contract cannot force a privileged port, wildcard bind, Proxy Protocol trust, or public administrative endpoint when the host profile forbids it.

## 7. Certificate Contract

Certificates are logical references, never embedded PEM values. Supported sources:

- SDKWork managed certificate resource.
- ACME managed policy.
- Secret-manager/KMS reference.
- Standalone protected file reference.
- Development-only generated self-signed certificate.

Each certificate declaration includes server names, source, lifecycle policy, renewal window, deployment scope, and optional client-auth trust reference. Full HTTPS behavior is defined in [PRD-https-and-certificates.md](PRD-https-and-certificates.md).

## 8. Resource Contract

Resource types:

- `static`: protected filesystem or packaged artifact root.
- `drive-artifact`: immutable SDKWork Drive-backed Web artifact.
- `proxy`: reference to an upstream pool.
- `redirect`: status and safe location template.
- `respond`: bounded fixed response.
- `acme-http-01`: reserved managed challenge responder.

Static resources support index files, `tryFiles`, SPA fallback, directory listing policy, MIME mapping, charset, ETag, Last-Modified, byte ranges, precompressed variants, cache control, hidden-file policy, symlink policy, dot-file policy, and maximum file size policy.

All filesystem paths are resolved relative to an approved application artifact root. Normalization, canonicalization, symlink escape, device files, alternate data streams, encoded traversal, and platform separator variants must be tested and fail closed.

## 9. Virtual Host Contract

Each virtual host includes:

- Stable `id` and listener references.
- Exact, leading-wildcard, trailing-wildcard, and regex server names.
- Default-host designation and canonical-domain redirect policy.
- Optional TLS policy and certificate references.
- Ordered route declarations.
- Host-level security, compression, cache, access, and observability policy references.

Server-name selection follows the declared Nginx compatibility profile. Ambiguous or conflicting ownership must be reported before publication.

## 10. Route Contract

Route matches may include:

- Exact, prefix, Nginx `^~` prefix, or PCRE2 regex path.
- HTTP methods.
- Header, query, cookie, source network, protocol, or host conditions.
- Explicit priority only for match combinations that cannot be represented by Nginx location order.

Route actions are exactly one of static resource, proxy upstream, redirect, fixed response, or managed extension. Rewrites, header transforms, body limits, timeout, cache, compression, rate, access, authentication, and observability are policies around the action rather than hidden side effects.

The compiler must reject ambiguous exact matches, unreachable routes, unsafe rewrites, rewrite loops, invalid regex, conflicting actions, missing references, and route counts above the configured application quota.

## 11. Upstream Contract

Each upstream supports:

- Static targets or an approved discovery reference.
- HTTP/HTTPS origin protocol, SNI, hostname verification, and optional mTLS.
- Target weight, backup status, drain state, and maximum connections.
- Round-robin, weighted round-robin, least-connections, IP hash, random-two-choice, and consistent-hash policy where implemented.
- Connection pooling, keepalive, connect/read/write timeouts, and queue limits.
- Active and passive health checks.
- Bounded retry budget, retryable error/status policy, circuit breaker, outlier detection, and recovery window.

Retries are allowed only before a non-replayable request body has been committed unless an explicit safe replay buffer and body-size bound exists.

## 11.1 Resolver Contract

Each resolver declaration supports approved DNS servers or a platform resolver profile, query timeout, retry bound, positive TTL floor/ceiling, negative TTL, stale-on-error window, IPv4/IPv6 policy, maximum answers, and cache budget.

Resolution is asynchronous and off the request executor's blocking path. Results retain DNS TTL semantics, are partitioned by application security scope, reject malformed or oversized answers, and cannot resolve an unapproved public hostname into a forbidden private/link-local destination when SSRF policy forbids it. An upstream using dynamic names must declare behavior for no healthy address, address-set change, and stale DNS.

## 12. Policy Contract

Policies include:

- Request header, URI, query, cookie, and body limits.
- Response header limits and security headers.
- IP/network access, CORS, method restrictions, and authentication extension references.
- Per-tenant/app/host/route rate and concurrent connection limits.
- Gzip and Brotli negotiation with minimum size and MIME allowlist.
- Static and proxy cache with explicit size, entry, TTL, stale, key, vary, and purge policies.
- Proxy buffering and streaming behavior.
- CSP, HSTS, `nosniff`, frame protection, referrer policy, and permissions policy.

Every queue, cache, buffer, rate bucket, connection pool, and concurrency gate has a finite default and an enforced maximum.

Cache policy additionally declares eligibility, canonical key inputs, `Vary` handling, authorization/cookie behavior, maximum object size, memory and disk budgets, stale behavior, revalidation, collapsed forwarding, purge authorization, and cache-poisoning defenses. Disk spooling and cache writes have per-app and process quotas and must fail without exhausting the runtime volume.

## 13. Observability Contract

Configuration declares:

- Structured access and error log profiles.
- Redaction policy.
- Metrics profile and low-cardinality dimensions.
- Trace sampling and propagation.
- Slow request/upstream thresholds.
- Health, readiness, and metrics exposure policy.

Raw tokens, private keys, authorization headers, cookies, query secrets, request bodies, absolute private paths, and unbounded user values must not be logged or used as metric labels.

## 14. Deployment Contract

Deployment declares supported standalone/cloud profiles, node selectors, revision strategy, canary size, health gates, drain timeout, convergence timeout, automatic rollback policy, and offline-node behavior.

The application config does not contain node credentials or mutable node inventories. Runtime assignments are control-plane state bound to the immutable app revision.

The app contract cannot configure process service accounts, global worker/thread counts, file descriptor limits, runtime directories, crash-dump policy, profiling exposure, or the node administrative listener. Those are host-owned controls so one tenant cannot affect the isolation or availability of other applications.

## 15. Configuration Precedence

Precedence from lowest to highest:

1. Product profile defaults.
2. App-owned `sdkwork.webserver.config.json`.
3. Source-controlled non-secret environment/profile overlay.
4. Published control-plane deployment overlay.
5. Secret/KMS/certificate/discovery resolution.
6. Operator emergency override with expiry, audit, and explicit rollback.

String interpolation is forbidden for security-sensitive values. Typed references are resolved at compile or activation time. A missing required binding is a blocking error.

## 16. Versioning And Lifecycle

- `schemaVersion` changes only when the machine contract changes.
- Additive optional fields may remain within the same schema version when old consumers reject or safely ignore them according to the schema contract.
- Published configurations are canonicalized, checksummed, immutable, and content-addressable.
- Every change supports validate, explain, diff, plan, publish, status, and rollback.
- Breaking schema or behavior changes require migration tooling, compatibility notes, and human review.

## 17. Acceptance Criteria

- Every active Web application root contains a valid app manifest and referenced Web Server configuration.
- All five default profiles validate and execute on standalone and cloud test topologies where applicable.
- Schema, semantic, security, resource-budget, Nginx-compatibility, and deployment validation produce deterministic diagnostics.
- Unknown fields, missing references, secrets, unsafe paths, listener conflicts, route ambiguity, and unbounded settings are rejected.
- The same canonical configuration produces equivalent normalized IR on Windows, Linux, and macOS tooling.
- Generated Nginx output and Rust execution pass the declared compatibility conformance suite.
- Logical listeners from multiple applications compile into conflict-free physical sockets under host policy.
- Resolver, cache, disk spool, connection, body, header, regex, route, and observability budgets remain finite after every precedence layer is applied.
- Application configuration examples contain no live credentials, private keys, environment-specific machine paths, or generated runtime state.
