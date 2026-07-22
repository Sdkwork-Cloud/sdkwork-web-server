use super::{WebsiteRuntimeDescriptor, WebsiteRuntimeDescriptorError};
use crate::canonical_json::canonical_sha256_excluding_field;

pub fn website_runtime_descriptor_sha256(
    descriptor: &WebsiteRuntimeDescriptor,
) -> Result<String, WebsiteRuntimeDescriptorError> {
    canonical_sha256_excluding_field(descriptor, "descriptorSha256")
        .map_err(WebsiteRuntimeDescriptorError::Json)
}
