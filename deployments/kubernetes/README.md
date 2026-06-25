# SDKWork Web Server Kubernetes Manifests

Apply order:

1. Create secrets `sdkwork-web-database` and `sdkwork-web-iam-database`
2. `kubectl apply -f migration-job.yaml` (wait for completion)
3. `kubectl apply -f deployment.yaml`
4. `kubectl apply -f service.yaml`

Health endpoints:

- `GET /healthz` — process liveness
- `GET /readyz` — database readiness

API surfaces:

- App: `/app/v3/api/*`
- Backend: `/backend/v3/api/*`
