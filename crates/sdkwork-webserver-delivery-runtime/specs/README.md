# SDKWork Web Server Delivery Runtime Specs

`component.spec.json` owns the immutable runtime-set request executor, provider registry,
activation-time provider validator, provider port dependencies, and focused verification for
`sdkwork-webserver-delivery-runtime`. Candidate validation is handler-aware, deduplicated per
logical resource and provider port, deadline-bound, and concurrency-bound before activation.

The crate is transport-neutral. It does not parse HTTP headers, construct SDK clients or
credentials, own runtime snapshot distribution, or depend on the legacy `ResourceConfig` pipeline.
Global standards remain authoritative under `../../../sdkwork-specs/`.
