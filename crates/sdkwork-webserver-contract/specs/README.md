# SDKWork Web Server Contract Specs

`component.spec.json` owns the Rust app/backend service ports, request/response DTOs, and service problem types shared across route, service, repository, and agent components.

The crate is an implementation contract module, not an OpenAPI authority or generated SDK family. Verify it with its Cargo tests and strict component-port validation.
