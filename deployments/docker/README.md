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

The runtime executes as uid/gid `10001`, listens on port `3800`, and uses `/tmp` as its only writable filesystem location.
