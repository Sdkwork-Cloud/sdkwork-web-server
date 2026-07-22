# SDKWork Web Server Knowledgebase Provider Specs

`component.spec.json` owns the generated Knowledgebase Internal SDK dependency, the Web Server
resource and Wiki provider ports, the external-service runtime requirement, and focused verification
for `sdkwork-webserver-knowledgebase-provider`.

The crate consumes a bootstrap-injected, tenant-bound SDK client resolver. One provider client is
valid only for the exact tenant scope configured by bootstrap; a service credential is never
reused across tenant scopes merely because a scope appears in a runtime-set. It does not construct
credentials, assume same-origin Knowledgebase APIs, mount public HTTP routes, or edit generated SDK
transport. Global standards remain authoritative under `../../../sdkwork-specs/`.
