require_relative 'base_api'
require_relative '../models/create_env_variable_request'
require_relative '../models/env_variable_page'
require_relative '../models/env_variable_response'

module Sdkwork
  module AppSdk
    module Api
      class EnvVariableApi < BaseApi
          # 获取环境变量列表
          def sites_env_variables_list(site_id, environment: nil)
            path = interpolate_path('/app/v3/api/sites/{siteId}/env_variables', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)))
            query = build_query_string([
              QueryParameterSpec.new('environment', environment, 'form', true, false, nil),
            ])
            path = append_query_string(path, query)
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::EnvVariablePage.from_hash(result) : nil
          end

          # 创建环境变量
          def sites_env_variables_create(site_id, body: nil)
            path = interpolate_path('/app/v3/api/sites/{siteId}/env_variables', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)))
            payload = body.respond_to?(:to_hash) ? body.to_hash : body
            options = {}
            options[:json] = payload unless payload.nil?
            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::EnvVariableResponse.from_hash(result) : nil
          end

      end
    end
  end
end
