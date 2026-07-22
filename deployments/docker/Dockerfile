FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 sdkwork \
    && useradd --uid 10001 --gid 10001 --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin sdkwork

WORKDIR /app

# The build context is the verified sdkwork-web/ root extracted from the release archive.
COPY --chown=10001:10001 . /app/

ENV SDKWORK_WEB_APP_ROOT=/app \
    SDKWORK_WEB_ENVIRONMENT=production \
    SDKWORK_WEB_CONFIG_PROFILE=production \
    SDKWORK_WEB_DEPLOYMENT_PROFILE=cloud \
    SDKWORK_WEB_RUNTIME_TARGET=container \
    SDKWORK_WEB_SERVICE_LAYOUT=website-data-plane \
    SDKWORK_WEB_SERVER_CONFIG_FILE=/app/etc/data-plane/website.cloud.config.json

USER 10001:10001
EXPOSE 8080

ENTRYPOINT ["/app/bin/sdkwork-web-server-website-delivery-edge-runtime"]
