# SDKWork Web Server Docker Image

Build from the application root:

```powershell
docker build -f deployments/docker/Dockerfile -t sdkwork-webserver-api-server:latest .
```

Run with PostgreSQL:

```powershell
docker run --rm -p 3800:3800 `
  -e SDKWORK_WEB_DATABASE_URL="postgres://user:pass@host:5432/web" `
  -e SDKWORK_IAM_DATABASE_URL="postgres://user:pass@host:5432/iam" `
  sdkwork-webserver-api-server:latest
```

Database migration only:

```powershell
docker run --rm `
  -e SDKWORK_WEB_DATABASE_AUTO_MIGRATE=true `
  -e SDKWORK_WEB_DATABASE_URL="postgres://user:pass@host:5432/web" `
  sdkwork-webserver-api-server:latest db-migrate
```

Health endpoints:

- `GET /healthz` — process liveness
- `GET /readyz` — database readiness
