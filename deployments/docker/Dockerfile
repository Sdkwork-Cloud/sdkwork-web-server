FROM rust:1-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p sdkwork-web-standalone-gateway

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /src/target/release/sdkwork-web-standalone-gateway /usr/local/bin/sdkwork-web-standalone-gateway
COPY --from=builder /src/database /app/database
ENV SDKWORK_WEB_APP_ROOT=/app
ENV SDKWORK_WEB_ENVIRONMENT=production
ENV SDKWORK_WEB_CONFIG_PROFILE=production
ENV SDKWORK_WEB_DEPLOYMENT_PROFILE=cloud
ENV SDKWORK_WEB_SERVICE_LAYOUT=unified-process
ENV SDKWORK_WEB_APPLICATION_PUBLIC_INGRESS_BIND=0.0.0.0:3800
EXPOSE 3800
CMD ["sdkwork-web-standalone-gateway"]
