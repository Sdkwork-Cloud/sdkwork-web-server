# SDKWork Web Server ACME Service Specs

`component.spec.json` declares certificate issuance, challenge storage, and private-key encryption provider ports.

The crate owns no HTTP route or SDK. Provider I/O, challenge files, private-key material, timeouts, and retries must remain bounded and are governed by the linked config, security, performance, and test standards.

Verify from the repository root with `cargo test -p sdkwork-webserver-acme-service` and strict component-port validation.
