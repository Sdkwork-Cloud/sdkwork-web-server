# SDKWork Web Server Container Image

The image is built only from the verified `sdkwork-web/` directory extracted from an immutable release archive. The Docker build must not use the repository source tree as its context.

The repository currently provides this validated build contract but does not enable a container
workflow target or registry publication. The commands below are an operator procedure, not evidence
that an image has been published.

```powershell
pnpm release:package:cloud
pnpm release:validate:cloud

$archive = Get-ChildItem dist/release/sdkwork-web-linux-*-cloud-server-*.tar.gz | Select-Object -First 1
$context = ".sdkwork/runtime/container-context"
New-Item -ItemType Directory -Force $context | Out-Null
tar -xzf $archive.FullName -C $context
docker build --pull --file deployments/docker/Dockerfile --tag registry.sdkwork.com/apps/sdkwork-web:0.1.0 "$context/sdkwork-web"
```

The extracted bundle contains the runtime binaries, application manifest, configuration schema, examples, and database lifecycle authority. Runtime credentials are injected by the deployment platform; no secret or mutable database state is copied into the image.

Before deployment, publish the image, resolve its registry `sha256` digest, verify provenance/signature/SBOM evidence, and render Kubernetes manifests with that digest. Mutable tags such as `latest` are not accepted as deployment identity.

The cloud image starts `sdkwork-web-server-website-delivery-edge-runtime`, executes as uid/gid
`10001`, and listens for website traffic on port `8080`. The image filesystem is immutable at
runtime; Kubernetes supplies `/tmp`, a node-specific protected recovery volume, and secret-file
credentials. The application standalone gateway remains a packaged standalone-profile binary and
is not the cloud image entrypoint.

The packaged `/app/etc/data-plane/website.cloud.config.json` is a fail-closed base policy and trusts
no forwarding headers. A container placed behind an external TLS terminator must mount a
compiler-validated environment-specific config at `/etc/sdkwork/web/sdkwork.webserver.config.json`,
set `SDKWORK_WEB_SERVER_CONFIG_FILE` to that path, and list only the terminator's direct peer CIDRs
under `listeners[].trustedProxy.trustedCidrs`. The Kubernetes renderer performs this materialization;
setting a universal trusted network is forbidden.
