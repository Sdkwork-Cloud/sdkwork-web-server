# SDKWork source workspace

`sdkwork.workspace.json` is the source-workspace application build map used by
the root `build:app` and `release:package:*` commands.

The map owns repository locations, stable application aliases, build actions,
and package actions only. Native package managers remain authoritative for
dependencies and application-local scripts.

`sdkwork.webserver.im-dev.json` is the tracked, non-secret development ingress
configuration used by `pnpm dev`. It owns the shared `/sdkwork-im/` path, HTTPS
listener, development certificate source, mobile user-agent tokens, application
roots, and internal Vite ports. Relative paths resolve from the `etc/` directory.

Edit that file to change the development ports or path. Restart `pnpm dev` after
changes. `certificate.mode` accepts `auto`, which materializes ignored local
certificate files, or `files`, which reads explicit `certificateFile` and
`privateKeyFile` paths. Never place private-key content in this tracked config.
The `$schema` entry points to `../specs/sdkwork.webserver.im-dev.schema.json` for
editor completion and validation.

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
