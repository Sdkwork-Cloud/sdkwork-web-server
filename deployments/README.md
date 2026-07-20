# SDKWork Web Server Deployments

Validated production deployment templates for the `sdkwork-web-server` repository. Catalog identity
remains `sdkwork-web`; topology and deployment identity follow the repository name.

| Path | Purpose |
| --- | --- |
| `deploy.yaml` | Deploy v2 authority for cloud and standalone production profiles |
| `docker/` | Minimal image assembled from a verified release bundle |
| `kubernetes/` | Digest-bound cloud workload, service, migration, and disruption contracts |

`cloud.production` uses an immutable container image on Kubernetes with multi-tenant shared infrastructure and high availability. `standalone.production` uses the signed Linux host package with a dedicated single-tenant host service. Both profiles expose API surfaces only; the planned PC component contracts are not deployed.

These files define deployable target state but do not prove publication. Application release
packages remain disabled with `releaseBuildDeferred: true`; no registry image or production rollout
is claimed until the corresponding artifact, digest, signature, SBOM, provenance, and approval
evidence exists.

Runtime values come from `etc/topology/*.env` and the deployment platform's secret manager. `sdkwork.app.config.json` declares identity and release capability, not concrete runtime configuration.
