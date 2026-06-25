# WEB Database Module

Canonical lifecycle assets for `sdkwork-web-server` per `DATABASE_FRAMEWORK_SPEC.md`.

- moduleId: `web`
- serviceCode: `WEB`
- tablePrefix: `web_`

## Commands

```bash
pnpm run db:materialize:contract
pnpm run db:validate
pnpm run db:bootstrap
```

Runtime bootstrap uses `sdkwork-database-cli` and `sdkwork-webserver-database-host`.
