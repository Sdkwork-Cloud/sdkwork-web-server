# SDKWork Web Server Source Configuration

`etc/sdkwork.deployment.config.json` is the source configuration entrypoint. It identifies
`sdkwork-web-server`, links `../specs/topology.spec.json`, and maps the four supported profiles to
tracked environment files:

| Profile | Source |
| --- | --- |
| `standalone.development` | `topology/standalone.development.env` |
| `cloud.development` | `topology/cloud.development.env` |
| `standalone.production` | `topology/standalone.production.env` |
| `cloud.production` | `topology/cloud.production.env` |

`sdkwork.app.config.json` owns application identity and release declarations only. Concrete binds,
origins, API surface URLs, database selection, upstream targets, and deployment profile values are
owned by `etc/` and `specs/topology.spec.json`.

## Development Profiles

`pnpm dev` and `pnpm dev:standalone` select `standalone.development` with runtime target `server`.
The plan starts the application-owned standalone gateway on `127.0.0.1:3800`.

`pnpm dev:cloud` selects `cloud.development` with runtime target `server`. The plan starts only the
local `sdkwork-web-node-daemon` client and resolves the deployed development surfaces from explicit
`https://*-dev.sdkwork.com` URLs. It does not start a gateway, API listener, database, migration,
seed process, or deployed-service worker.

`node-daemon/development.env.example` is the canonical non-secret Node Daemon environment example.
`agent/development.env.example` and `SDKWORK_WEB_AGENT_*` remain wire/runtime compatibility aliases
for the v3 Agent contract; conflicting canonical and compatibility values fail startup.

## Runtime And Secrets

Tracked files contain no access tokens, Node Tokens, passwords, private keys, or database
credentials. Use process environment overrides, protected secret files, or the deployment
platform's secret manager. Local overrides and materialized runtime state belong under ignored
`.sdkwork/runtime/` or approved operator paths; they must not be committed.

Production images carry the fail-closed website listener base policy at
`/app/etc/data-plane/website.cloud.config.json`; it trusts no forwarding metadata. Kubernetes
renders the reviewed direct-ingress CIDRs into an immutable per-Node ConfigMap mounted at
`/etc/sdkwork/web/sdkwork.webserver.config.json`. Mutable Node identity, provider-event subscriptions,
and credentials are mounted read-only under `/run/secrets/sdkwork-web-node/`. The Kubernetes
migration Job obtains database URLs, independent encryption roots, and the ACME contact through the
`sdkwork-web-server-runtime` secret reference documented in `../deployments/kubernetes/README.md`.

`examples/sdkwork.webserver.config.json` is the safe standalone data-plane example. It is validated
against `../specs/sdkwork.webserver.config.schema.json`; certificate and private-key values are file
references rather than embedded secrets.

`data-plane/website.development.env.example` is the non-secret standalone/development
website/Wiki data-plane example and explicitly selects the `file` assignment source.
`data-plane/website.cloud.env.example` is the production cloud fragment and selects the
authenticated Web Internal API assignment source. Both examples point credentials at protected
secret files; no credential value belongs in source configuration. Each data-plane process is
explicitly bound to one Web Node identity and one 64-character
`tenantScopeHash`; its provider credentials must authorize that same tenant, and a candidate
runtime-set containing another or multiple tenant scopes is rejected before activation. The token
files contain only deployment-provided ingress tokens and must never be committed. Production and
staging provider origins must use HTTPS. Provider resources are validated before initial activation
and every watched update with bounded concurrency; a failure retains the last-known-good set.
`SDKWORK_WEB_WEBSITE_RUNTIME_SET_RECOVERY_DIRECTORY` owns a dedicated node-local A/B slot
directory containing only complete, hash-verified `sdkwork.website-runtime-set.v1` snapshots.
Staging and production require this directory. Bootstrap selects the highest valid generation from
the source and recovery state, rejects same-generation hash conflicts and node/environment scope
mismatches, and can restart from the recovered snapshot while the source is unavailable. A source
older than the recovered generation cannot lower the replay barrier. Successful initial and watched
activations persist the inactive slot with bounded asynchronous I/O before the update is considered
durable. The directory is node data-plane state, not Web business persistence or a substitute for
authenticated Deploy runtime-set distribution; it must be writable only by the service identity,
must not share files with another subsystem, and belongs on durable host storage.

`data-plane/website-provider-events.development.json.example` is the provider-event ingress
instance selected by `SDKWORK_WEB_WEBSITE_PROVIDER_EVENT_CONFIG_FILE` and validated by
`../specs/sdkwork.website-provider-event-ingress.schema.json`. It binds only to loopback, maps each
unguessable subscription path to an expected provider/channel/tenant/organization, references a
protected signing secret file, and writes dual-slot per-stream checkpoints under ignored runtime
state. Drive uses the original channel verification token; the receiver derives Drive's signing
key exactly as the owner contract requires. Knowledgebase uses its outbox webhook secret directly.
Production and staging place an authenticated internal HTTPS ingress or sidecar in front of this
loopback listener; the public website listener never mounts provider-event routes. Both owner
webhooks sign `delivery-time + "." + exact-body`, and the receiver enforces the configured clock
window before strict AsyncAPI parsing. A production/staging website runtime-set that uses either
provider fails bootstrap when this event-ingress configuration is absent.

The website data plane starts with:

```powershell
cargo run -p sdkwork-web-server-website-delivery-edge-runtime
```

The dedicated edge runtime loads `SDKWORK_WEB_SERVER_CONFIG_FILE` for listener/TLS limits and the assignment
source selected by `SDKWORK_WEB_RUNTIME_ASSIGNMENT_SOURCE` for immutable
Site/Binding/Variant/Mount routing. `cloud` is the production source: the generated Web Internal
SDK authenticates with the secret-file `SDKWORK_WEB_NODE_TOKEN_FILE`, conditionally pulls the
current assignment for `SDKWORK_WEB_NODE_UUID` and
`SDKWORK_WEB_WEBSITE_RUNTIME_ENVIRONMENT`, verifies assignment identity/hash and the complete
runtime-set, and reports `RECEIVED`, `VALIDATED`, `STAGED`, `ACTIVE`, or bounded `REJECTED`
observations. `file` is limited to standalone/development and reads
`SDKWORK_WEB_WEBSITE_RUNTIME_SET_FILE`. Both modes retain the durable last-known-good runtime-set
when an update is invalid, stale, terminally rejected, or requires an unavailable provider, and
recover it after restart when the source is temporarily unavailable. A cloud node with a valid
last-known-good snapshot can start during a temporary control-plane outage; a first-start node
without one fails closed.

For an HTTP listener behind a TLS terminator, the runtime uses `X-Forwarded-Proto` only when the
immediate TCP peer is covered by `trustedProxy.trustedCidrs`. It accepts exactly one `http` or
`https` value; duplicates, lists, whitespace variants, non-text values, and oversized trusted
headers fail with `400`. Untrusted peers cannot override the listener transport, and native TLS
cannot be downgraded by forwarding metadata.

## Validation

```powershell
node ..\sdkwork-specs\tools\check-source-config-standard.mjs --root .
pnpm topology:validate
pnpm exec sdkwork-app doctor
cargo run -p sdkwork-api-web-server-standalone-gateway -- validate etc/examples/sdkwork.webserver.config.json
```

Use `pnpm release:package:standalone` or `pnpm release:package:cloud` only on a Linux runner whose
architecture matches `SDKWORK_PACKAGE_ARCHITECTURE`. Release declarations remain disabled in the
application manifest until release evidence and publication authority are approved.
