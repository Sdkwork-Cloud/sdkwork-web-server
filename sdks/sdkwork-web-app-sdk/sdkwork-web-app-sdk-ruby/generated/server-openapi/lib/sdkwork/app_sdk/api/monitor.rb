require_relative 'base_api'
require_relative '../models/create_health_check_request'
require_relative '../models/health_check_page'
require_relative '../models/health_check_response'

module Sdkwork
  module AppSdk
    module Api
      class MonitorApi < BaseApi
          # 获取健康检查配置
          def sites_health_checks_list(site_id)
            path = interpolate_path('/app/v3/api/sites/{siteId}/health_checks', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)))
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::HealthCheckPage.from_hash(result) : nil
          end

          # 创建健康检查
          def sites_health_checks_create(site_id, body: nil)
            path = interpolate_path('/app/v3/api/sites/{siteId}/health_checks', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)))
            payload = body.respond_to?(:to_hash) ? body.to_hash : body
            options = {}
            options[:json] = payload unless payload.nil?
            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::HealthCheckResponse.from_hash(result) : nil
          end

      end
    end
  end
end
