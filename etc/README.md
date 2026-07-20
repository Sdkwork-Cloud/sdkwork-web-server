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

Production installs materialize reviewed configuration under `/etc/sdkwork/web/`. The Kubernetes
profile obtains database URLs, independent encryption roots, and the ACME contact through the
`sdkwork-web-server-runtime` secret reference documented in `../deployments/kubernetes/README.md`.

`examples/sdkwork.webserver.config.json` is the safe standalone data-plane example. It is validated
against `../specs/sdkwork.webserver.config.schema.json`; certificate and private-key values are file
references rather than embedded secrets.

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
