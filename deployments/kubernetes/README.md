# SDKWork Web Server Kubernetes Deployment

The authored manifests are production templates. They deliberately contain
`__SDKWORK_IMAGE_DIGEST__`; apply only rendered output bound to an attested registry digest. The
application manifest currently defers release builds, so these templates are not an assertion that
a registry image or production environment exists.

## Required Runtime Secret

Create `sdkwork-web-server-runtime` through the approved secret manager integration with these keys:

| Key | Purpose |
| --- | --- |
| `database-url` | PostgreSQL authority for Web Server data |
| `iam-database-url` | IAM request-context database authority |
| `secret-encryption-key` | Production environment-value encryption root |
| `certificate-encryption-key` | Certificate private-key encryption root |
| `acme-contact-email` | ACME account contact |

Do not commit a Secret manifest or plaintext values. Encryption roots must be independent, randomly generated, rotation-governed values.

## Release And Apply

1. Build and validate the cloud release archive and container image.
2. Verify the image checksum, signature, provenance/attestation, and SBOM.
3. Render manifests with the verified registry digest:

   ```powershell
   node scripts/render-kubernetes-manifests.mjs --image-digest <64-hex-sha256>
   ```

4. Apply the rendered `migration-job.yaml`, wait for successful completion, then apply `service.yaml` and `deployment.yaml`.
5. Wait for the StatefulSet rollout and verify `/healthz`, `/readyz`, and metrics from the operations surface before routing production traffic.

The StatefulSet ordinal becomes the bounded Snowflake node id, giving each replica a stable numeric identity. The workload uses a read-only root filesystem, disabled service-account token mounting, dropped Linux capabilities, RuntimeDefault seccomp, bounded ephemeral storage, startup/readiness/liveness probes, rolling updates, and a PodDisruptionBudget.

Scaling is limited to 1024 replicas by the node-id contract. A larger deployment requires a reviewed ID-allocation design rather than ordinal reuse.
