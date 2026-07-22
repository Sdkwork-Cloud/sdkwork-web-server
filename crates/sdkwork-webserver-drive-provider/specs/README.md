# SDKWork Web Server Drive Provider Specs

Machine-readable integration authority: [component.spec.json](component.spec.json).

The crate adapts the generated Drive Internal SDK to the Web Server website resource and
static-content provider ports. Its injected client resolver is bound to one exact tenant scope; it
never infers credential authority from runtime-set membership. It never reads Drive tables or
object-store coordinates directly.
