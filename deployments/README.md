# SDKWork Web Server Deployments

Production packaging assets for `sdkwork-web`:

| Path | Purpose |
| --- | --- |
| `docker/` | Multi-stage container image for the standalone/cloud Web gateway runtime |
| `kubernetes/` | Cloud deployment, service, and migration job manifests |

Local development uses `pnpm dev` with topology profile `configs/topology/standalone.development.env`.

Production topology profile: `configs/topology/cloud.production.env`.
