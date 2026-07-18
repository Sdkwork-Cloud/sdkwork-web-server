require_relative 'base_api'
require_relative '../models/create_nginx_config_request'
require_relative '../models/nginx_config_page'
require_relative '../models/nginx_config_response'
require_relative '../models/nginx_deploy_response'
require_relative '../models/nginx_reload_response'
require_relative '../models/nginx_status_response'
require_relative '../models/nginx_validate_response'
require_relative '../models/update_nginx_config_request'

module Sdkwork
  module BackendSdk
    module Api
      class NginxApi < BaseApi
          # 获取 Nginx 配置列表
          def configs_list(page: nil, page_size: nil, site_id: nil, config_type: nil, is_active: nil)
            path = '/backend/v3/api/nginx/configs'
            query = build_query_string([
              QueryParameterSpec.new('page', page, 'form', true, false, nil),
              QueryParameterSpec.new('pageSize', page_size, 'form', true, false, nil),
              QueryParameterSpec.new('siteId', site_id, 'form', true, false, nil),
              QueryParameterSpec.new('configType', config_type, 'form', true, false, nil),
              QueryParameterSpec.new('isActive', is_active, 'form', true, false, nil),
            ])
            path = append_query_string(path, query)
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::NginxConfigPage.from_hash(result) : nil
          end

          # 创建 Nginx 配置
          def configs_create(body: nil)
            path = '/backend/v3/api/nginx/configs'
            payload = body.respond_to?(:to_hash) ? body.to_hash : body
            options = {}
            options[:json] = payload unless payload.nil?
            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::NginxConfigResponse.from_hash(result) : nil
          end

          # 获取 Nginx 配置详情
          def configs_retrieve(config_id)
            path = interpolate_path('/backend/v3/api/nginx/etc/{configId}', configId: serialize_path_parameter(config_id, PathParameterSpec.new('configId', 'simple', false)))
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::NginxConfigResponse.from_hash(result) : nil
          end

          # 更新 Nginx 配置
          def configs_update(config_id, body: nil)
            path = interpolate_path('/backend/v3/api/nginx/etc/{configId}', configId: serialize_path_parameter(config_id, PathParameterSpec.new('configId', 'simple', false)))
            payload = body.respond_to?(:to_hash) ? body.to_hash : body
            options = {}
            options[:json] = payload unless payload.nil?
            result = @client.request('PUT', path, **options)
            result.is_a?(Hash) ? Models::NginxConfigResponse.from_hash(result) : nil
          end

          # 校验 Nginx 配置
          def configs_validate(config_id)
            path = interpolate_path('/backend/v3/api/nginx/etc/{configId}/validate', configId: serialize_path_parameter(config_id, PathParameterSpec.new('configId', 'simple', false)))
            options = {}

            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::NginxValidateResponse.from_hash(result) : nil
          end

          # 部署 Nginx 配置
          def configs_deploy(config_id)
            path = interpolate_path('/backend/v3/api/nginx/etc/{configId}/deploy', configId: serialize_path_parameter(config_id, PathParameterSpec.new('configId', 'simple', false)))
            options = {}

            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::NginxDeployResponse.from_hash(result) : nil
          end

          # 热加载 Nginx
          def reload()
            path = '/backend/v3/api/nginx/reload'
            options = {}

            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::NginxReloadResponse.from_hash(result) : nil
          end

          # 获取 Nginx 状态
          def status_retrieve()
            path = '/backend/v3/api/nginx/status'
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::NginxStatusResponse.from_hash(result) : nil
          end

      end
    end
  end
end
