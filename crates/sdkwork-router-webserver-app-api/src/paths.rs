pub const PREFIX: &str = "/app/v3/api";

pub const SITES: &str = "/app/v3/api/sites";
pub const SITE: &str = "/app/v3/api/sites/{siteId}";
pub const SITE_ACTIVATE: &str = "/app/v3/api/sites/{siteId}/activate";
pub const SITE_PAUSE: &str = "/app/v3/api/sites/{siteId}/pause";
pub const SITE_DOMAINS: &str = "/app/v3/api/sites/{siteId}/domains";
pub const SITE_DOMAIN: &str = "/app/v3/api/sites/{siteId}/domains/{domainId}";
pub const SITE_DOMAIN_VERIFY: &str = "/app/v3/api/sites/{siteId}/domains/{domainId}/verify";
pub const SITE_DEPLOYMENTS: &str = "/app/v3/api/sites/{siteId}/deployments";
pub const SITE_DEPLOYMENT: &str = "/app/v3/api/sites/{siteId}/deployments/{deploymentId}";
pub const SITE_DEPLOYMENT_ROLLBACK: &str =
    "/app/v3/api/sites/{siteId}/deployments/{deploymentId}/rollback";
pub const SITE_ENV_VARIABLES: &str = "/app/v3/api/sites/{siteId}/env_variables";
pub const CERTIFICATES: &str = "/app/v3/api/certificates";
pub const SITE_HEALTH_CHECKS: &str = "/app/v3/api/sites/{siteId}/health_checks";
