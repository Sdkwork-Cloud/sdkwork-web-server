# SDKWork Web Server

Standards-aligned HTTP backend for site, domain, deployment, certificate, nginx, and edge-agent management. Exposes **app-api** and **backend-api** surfaces integrated with SDKWork platform frameworks.

## Framework Integration

| Framework | Status |
| --- | --- |
| `sdkwork-web-framework` | Integrated on app-api and backend-api routers |
| `sdkwork-database` | Integrated through `database/` assets and `sdkwork-webserver-database-host` |
| `sdkwork-utils-rust` | API envelope, crypto, env parsing, slugify, serde helpers |
| `sdkwork-id-core` (via `sdkwork-database-id`) | Snowflake PKs and UUID resource identifiers |
| `sdkwork-discovery` | Not required until RPC split-services are introduced |
| `sdkwork-drive` | Not required until file-upload API operations are added |

## Root Layout

| Directory | Purpose |
| --- | --- |
| `apis/` | Authoritative OpenAPI contracts |
| `crates/` | Rust service, repository, route, and gateway crates |
| `database/` | Database contract, baseline DDL, migrations, seeds |
| `specs/` | Component and topology contracts |
| `configs/` | Topology profile env templates |
| `deployments/` | Docker and Kubernetes descriptors |
| `docs/` | PRD, architecture, ADRs, standards alignment |
| `tests/` | Cross-package contract tests |

## Development

```powershell
pnpm dev
pnpm check
pnpm verify
```

## Documentation

- Standards entry: `../sdkwork-specs/README.md`
- Alignment status: [docs/standards-alignment.md](docs/standards-alignment.md)
- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)

## Application Roots

- [apps directory index](apps/README.md)
