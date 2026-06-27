require_relative 'sdkwork/backend_sdk/version'
require_relative 'sdkwork/backend_sdk/sdk_config'
require_relative 'sdkwork/backend_sdk/models/problem_detail'
require_relative 'sdkwork/backend_sdk/models/create_nginx_config_request'
require_relative 'sdkwork/backend_sdk/models/update_nginx_config_request'
require_relative 'sdkwork/backend_sdk/models/nginx_config_response'
require_relative 'sdkwork/backend_sdk/models/nginx_config_page'
require_relative 'sdkwork/backend_sdk/models/nginx_validate_response'
require_relative 'sdkwork/backend_sdk/models/nginx_deploy_response'
require_relative 'sdkwork/backend_sdk/models/nginx_reload_response'
require_relative 'sdkwork/backend_sdk/models/nginx_status_response'
require_relative 'sdkwork/backend_sdk/models/create_server_request'
require_relative 'sdkwork/backend_sdk/models/server_response'
require_relative 'sdkwork/backend_sdk/models/create_server_response'
require_relative 'sdkwork/backend_sdk/models/agent_heartbeat_request'
require_relative 'sdkwork/backend_sdk/models/agent_heartbeat_response'
require_relative 'sdkwork/backend_sdk/models/agent_sync_response'
require_relative 'sdkwork/backend_sdk/models/agent_nginx_config_bundle'
require_relative 'sdkwork/backend_sdk/models/agent_certificate_bundle'
require_relative 'sdkwork/backend_sdk/models/server_page'
require_relative 'sdkwork/backend_sdk/models/audit_log_response'
require_relative 'sdkwork/backend_sdk/models/audit_log_page'
require_relative 'sdkwork/backend_sdk/http/client'
require_relative 'sdkwork/backend_sdk/api/base_api'
require_relative 'sdkwork/backend_sdk/api/nginx'
require_relative 'sdkwork/backend_sdk/api/server'
require_relative 'sdkwork/backend_sdk/api/agent'
require_relative 'sdkwork/backend_sdk/api/audit'
require_relative 'sdkwork/backend_sdk/client'

module Sdkwork
  module BackendSdk
    def self.create_client(config = SdkConfig.new)
      SdkworkBackendClient.new(config)
    end
  end
end
