# SDKWork Web Server Deployments

Production packaging assets for `sdkwork-web`:

| Path | Purpose |
| --- | --- |
| `docker/` | Multi-stage container image for unified-process API server |
| `kubernetes/` | Cloud deployment, service, and migration job manifests |

Local development uses `pnpm dev` with topology profile `configs/topology/cloud.unified-process.development.env`.

Production topology profile: `configs/topology/cloud.unified-process.production.env`.
