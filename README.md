# SDKWork Web Server

SDKWork Web Server is a standards-aligned HTTP backend service with app-api and
backend-api surfaces, integrated with `sdkwork-web-framework`, `sdkwork-database`, and
`sdkwork-utils-rust`.

## Standards Alignment

| Framework | Status |
| --- | --- |
| `sdkwork-web-framework` | Integrated on app-api and backend-api routers |
| `sdkwork-database` | Integrated through `database/` assets and `sdkwork-webserver-database-host` |
| `sdkwork-utils-rust` | Used for env parsing and shared validation helpers |
| `sdkwork-discovery` | Deferred until RPC services are introduced |

## Root Layout

| Directory | Purpose |
| --- | --- |
| `apis/` | Authoritative OpenAPI contracts |
| `crates/` | Rust service, repository, route, and API server crates |
| `database/` | Database contract, baseline DDL, migrations, seeds |
| `specs/` | Component and topology contracts |
| `configs/` | Topology profile env templates |
| `deployments/` | Docker and release handoff descriptors |
| `docs/` | PRD, architecture, ADRs |
| `tests/` | Cross-package contract tests |

## Development

```powershell
pnpm dev
pnpm check
pnpm verify
```

## Documentation

- Standards entry: `../sdkwork-specs/README.md`
- Alignment notes: `docs/standards-alignment.md`

## Documentation Canon

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)

