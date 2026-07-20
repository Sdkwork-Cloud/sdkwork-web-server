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
    SDKWORK_WEB_SERVICE_LAYOUT=unified-process \
    SDKWORK_WEB_APPLICATION_PUBLIC_INGRESS_BIND=0.0.0.0:3800

USER 10001:10001
EXPOSE 3800

ENTRYPOINT ["/app/bin/sdkwork-api-web-server-standalone-gateway"]
CMD ["serve-management"]
