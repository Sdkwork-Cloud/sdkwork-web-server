pub const PREFIX: &str = "/backend/v3/api";

pub const NGINX_CONFIGS: &str = "/backend/v3/api/nginx/configs";
pub const NGINX_CONFIG: &str = "/backend/v3/api/nginx/configs/{configId}";
pub const NGINX_CONFIG_VALIDATE: &str = "/backend/v3/api/nginx/configs/{configId}/validate";
pub const NGINX_CONFIG_DEPLOY: &str = "/backend/v3/api/nginx/configs/{configId}/deploy";
pub const NGINX_RELOAD: &str = "/backend/v3/api/nginx/reload";
pub const NGINX_STATUS: &str = "/backend/v3/api/nginx/status";
pub const SERVERS: &str = "/backend/v3/api/servers";
pub const AUDIT_LOGS: &str = "/backend/v3/api/audit_logs";
pub const AGENT_HEARTBEAT: &str = "/backend/v3/api/agent/heartbeat";
pub const AGENT_SYNC: &str = "/backend/v3/api/agent/sync";
