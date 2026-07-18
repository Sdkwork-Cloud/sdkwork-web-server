# SDKWork source workspace

`sdkwork.workspace.json` is the source-workspace application build map used by
the root `build:app` and `release:package:*` commands.

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
