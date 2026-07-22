mod canonical;
mod compiled;
mod error;
mod loader;
mod model;
mod runtime_set;
mod validate;

pub use canonical::website_runtime_descriptor_sha256;
pub use compiled::{
    CompiledWebsiteRuntimeDescriptor, SelectedWebsiteRedirect, SelectedWebsiteRoute,
    WebsiteClientClassificationSource, WebsiteRequestRoutingContext, WebsiteRouteSelection,
    WebsiteVariantSelectionReason,
};
pub use error::{WebsiteRouteSelectionError, WebsiteRuntimeDescriptorError};
pub use loader::{compile_website_runtime_descriptor, MAX_WEBSITE_RUNTIME_DESCRIPTOR_BYTES};
pub use model::*;
pub use runtime_set::{
    compile_website_runtime_set_snapshot, website_runtime_set_snapshot_sha256,
    CompiledWebsiteRuntimeSet, WebsiteRuntimeActivationReport, WebsiteRuntimeRegistry,
    WebsiteRuntimeRollbackReport, WebsiteRuntimeSetError, WebsiteRuntimeSetSnapshot,
    MAX_WEBSITE_RUNTIME_SET_BYTES, WEBSITE_RUNTIME_SET_KIND, WEBSITE_RUNTIME_SET_SCHEMA_VERSION,
};
pub use validate::normalize_website_hostname;

pub(crate) use validate::validate_website_runtime_descriptor;
