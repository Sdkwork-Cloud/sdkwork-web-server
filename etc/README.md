# SDKWork source workspace

`sdkwork.workspace.json` is the source-workspace application build map used by
the root `build:app` and `release:package:*` commands.

The application deployment entrypoint is `sdkwork.deployment.config.json`. It
selects a typed `<deployment-profile>.<environment>` profile and resolves the
referenced Web Server or gateway configuration from this directory. The
repository-level configuration schema authority is `../specs/`; concrete bind,
port, domain, upstream, certificate-file reference, and runtime values belong
here and must not be copied into `sdkwork.app.config.json`.

`pnpm dev` delegates to `pnpm dev:standalone` and resolves the tracked
`standalone.development` profile. `pnpm dev:cloud` resolves
`node-daemon/cloud.development.json`, requires a remote HTTPS control-plane origin, and starts only
the local Web Node Daemon; it does not start a local database, API process, or gateway. Supply
`SDKWORK_WEB_NODE_TOKEN` through the local process environment for an actual cloud run. The tracked
profile contains no token, and `--dry-run` never requires or prints one.

Paired `pnpm release:package:standalone` and `pnpm release:package:cloud` commands produce profile-
specific Linux x64 or arm64 server archive candidates under `dist/release/`. The reusable workflow
supplies `SDKWORK_DEPLOYMENT_PROFILE`, `SDKWORK_PACKAGE_ARCHITECTURE`, and
`SDKWORK_PACKAGE_VERSION`. Actual archive creation requires a Linux process whose architecture
matches the selected package architecture; Windows dry-runs prove selection and naming only, not an
installable release.
Every package operation validates the completed archive before returning, and the workflow repeats
`release:validate:standalone` or `release:validate:cloud` before upload. Validation is streaming and
bounded, requires the exact manifest/inventory, and rejects unsafe paths, links, special entries,
metadata drift, mode drift, size drift, or checksum drift. REQ-2026-0058 and REQ-2026-0059 add real
x64/arm64 extraction and HTTP/HTTPS runtime smoke. They do not provide native hardware capacity,
SBOM, signature, provenance, installer, upgrade, or rollback evidence.

The map owns repository locations, stable application aliases, build actions,
and package actions only. Native package managers remain authoritative for
dependencies and application-local scripts.

`sdkwork.webserver.im-dev.json` is the tracked, non-secret development ingress
configuration used by `pnpm dev`. It selects the canonical SDKWork IM app
manifest and lifecycle environment, deployment profile, standalone server script,
gateway target and service route prefixes, listener bind, mobile user-agent tokens,
application roots, and internal Vite ports. The selected manifest environment
owns the public protocol, hostname, port, and root path. Relative paths resolve
from the `etc/` directory.

Edit the SDKWork IM root `etc/sdkwork.deployment.config.json#environments` to change the
public origin. The IM app manifest contains identity and release declarations only. Edit this
file to select an environment, change internal Vite or
gateway ports, select the managed IM server script, or adjust service route prefixes.
In standalone mode, browser SDK URLs are injected as the public application origin
and the listed API/WebSocket routes proxy to `deployment.gateway` before PC/H5
selection. Restart `pnpm dev` after changes. For an HTTPS manifest origin,
`listener.certificate.mode` accepts `auto`, which materializes ignored local
certificate files, or `files`, which reads explicit `certificateFile` and
`privateKeyFile` paths. Never place private-key content in this tracked config.
The `$schema` entry points to `../specs/sdkwork.webserver.im-dev.schema.json` for
editor completion and validation.

`examples/sdkwork.webserver.config.json` is the safe, non-secret standalone data
plane example validated by the gateway command. Listener `proxyProtocol` is an
opt-in edge trust boundary, not a default. When enabled, declare only immediate
load-balancer CIDRs and accepted wire versions, for example:

```json
"proxyProtocol": {
  "trustedSourceCidrs": ["10.0.0.0/8"],
  "versions": ["v1", "v2"],
  "timeoutMs": 3000,
  "maxHeaderBytes": 536,
  "crc32cPolicy": "validate-if-present"
}
```

The field requires every connection on that listener to begin with a valid
PROXY header before TLS ClientHello or HTTP bytes. It is mutually exclusive with
HTTP `trustedProxy`; do not enable it on an Internet-facing listener whose
immediate peers are not exclusively trusted load balancers. CRC32C checking is
not peer authentication: use `validate-if-present` during a producer rollout and
move to `required` only after every trusted load balancer emits one valid v2 CRC.

Local overrides use ignored `etc/**/*.local.*` files. Tracked files may contain
secret-file references and placeholders only; TLS private keys, tokens,
passwords, runtime databases, logs, and caches are forbidden. Operators
materialize reviewed production configuration under `/etc/sdkwork/web/` and
inject secrets through mounted protected files or the platform secret manager.

`node-daemon/development.env.example` is the canonical source example for the SDKWork Web Node
Daemon profile. New deployments use the `SDKWORK_WEB_NODE_*` keys documented there. The
`agent/development.env.example` location and `SDKWORK_WEB_AGENT_*` aliases remain only for the v3
compatibility window; conflicting preferred and legacy values fail daemon startup. Tracked examples
contain placeholders only.

Validate source configuration ownership and the runnable example with:

```powershell
node ..\sdkwork-specs\tools\check-source-config-standard.mjs --root .
cargo run -p sdkwork-api-web-server-standalone-gateway -- validate etc/examples/sdkwork.webserver.config.json
pnpm release:smoke:standalone   # Linux x64 only: extracted HTTP/HTTPS/stop smoke
pnpm release:smoke:cloud        # Linux x64 only: extracted HTTP/HTTPS/stop smoke
```

`im-dev.hosts.example` contains the cross-platform development DNS mapping. Copy
the loopback entry into the Windows, macOS, or Linux hosts file for a browser on
the development machine. Physical phones and other LAN clients require the same
hostname to resolve to the development machine's private LAN address through the
LAN DNS server/router; a hosts entry on the development machine is not visible to
other devices.

Examples:

```text
pnpm build:app sdkwork-im-pc
pnpm build:app sdkwork-im-h5
pnpm build:app sdkwork-im-flutter
pnpm release:package:ios sdkwork-im-flutter
pnpm release:package:android sdkwork-im-flutter
pnpm release:package:ios sdkwork-im-h5
pnpm release:package:android sdkwork-im-h5
```
