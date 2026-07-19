# SDKWork Web Server Edge Runtime Specs

`component.spec.json` declares the host adapter that materializes certificate bundles, validates/deploys Nginx site configuration, and reloads the local edge process.

The runtime owns filesystem and process integration only; certificate issuance and business orchestration remain separate provider/service components.

Verify from the repository root with `cargo test -p sdkwork-webserver-edge-runtime` and strict component-port validation.
