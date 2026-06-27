require_relative 'sdkwork/app_sdk/version'
require_relative 'sdkwork/app_sdk/sdk_config'
require_relative 'sdkwork/app_sdk/models/problem_detail'
require_relative 'sdkwork/app_sdk/models/create_site_request'
require_relative 'sdkwork/app_sdk/models/update_site_request'
require_relative 'sdkwork/app_sdk/models/site_response'
require_relative 'sdkwork/app_sdk/models/site_page'
require_relative 'sdkwork/app_sdk/models/create_domain_request'
require_relative 'sdkwork/app_sdk/models/domain_response'
require_relative 'sdkwork/app_sdk/models/domain_page'
require_relative 'sdkwork/app_sdk/models/domain_verify_response'
require_relative 'sdkwork/app_sdk/models/create_deployment_request'
require_relative 'sdkwork/app_sdk/models/deployment_response'
require_relative 'sdkwork/app_sdk/models/deployment_page'
require_relative 'sdkwork/app_sdk/models/create_env_variable_request'
require_relative 'sdkwork/app_sdk/models/env_variable_response'
require_relative 'sdkwork/app_sdk/models/env_variable_page'
require_relative 'sdkwork/app_sdk/models/create_certificate_request'
require_relative 'sdkwork/app_sdk/models/certificate_response'
require_relative 'sdkwork/app_sdk/models/certificate_page'
require_relative 'sdkwork/app_sdk/models/create_health_check_request'
require_relative 'sdkwork/app_sdk/models/health_check_response'
require_relative 'sdkwork/app_sdk/models/health_check_page'
require_relative 'sdkwork/app_sdk/http/client'
require_relative 'sdkwork/app_sdk/api/base_api'
require_relative 'sdkwork/app_sdk/api/site'
require_relative 'sdkwork/app_sdk/api/domain'
require_relative 'sdkwork/app_sdk/api/deployment'
require_relative 'sdkwork/app_sdk/api/env_variable'
require_relative 'sdkwork/app_sdk/api/certificate'
require_relative 'sdkwork/app_sdk/api/monitor'
require_relative 'sdkwork/app_sdk/client'

module Sdkwork
  module AppSdk
    def self.create_client(config = SdkConfig.new)
      SdkworkAppClient.new(config)
    end
  end
end
