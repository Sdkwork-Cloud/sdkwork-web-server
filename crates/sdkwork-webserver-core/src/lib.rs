//! Web Server core runtime helpers.

pub mod runtime_env;
pub mod util;

pub use runtime_env::{
    web_dev_auth_bypass_enabled, web_environment_name, web_is_production_like_environment,
    web_use_dev_inline_auth_resolver,
};
pub use util::{normalize_pagination, pagination_offset};
