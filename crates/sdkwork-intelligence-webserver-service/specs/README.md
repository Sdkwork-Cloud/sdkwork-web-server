# SDKWork Web Server Business Service Specs

`component.spec.json` is the machine contract for the Web Server business service and its repository/provider ports.

The crate orchestrates application and backend operations through `WebRepositoryPort`, `CertificateIssuer`, and `EdgeRuntime`. It owns no HTTP route manifest or generated SDK surface.

Verify from the repository root with `cargo test -p sdkwork-intelligence-webserver-service` and the strict component-port validator.
