# SDKWork Web Server Certificate Worker Specs

`component.spec.json` declares the renewal scheduler binary and its service/repository runtime dependencies.

The worker owns scheduling only. Certificate selection, issuance, persistence, and edge materialization remain in their declared provider/service components.

Verify from the repository root with the worker Cargo tests and strict component-port validation.
