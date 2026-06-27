require_relative 'base_api'
require_relative '../models/create_deployment_request'
require_relative '../models/deployment_page'
require_relative '../models/deployment_response'

module Sdkwork
  module AppSdk
    module Api
      class DeploymentApi < BaseApi
          # 获取部署历史
          def sites_deployments_list(site_id, page: nil, page_size: nil, status: nil)
            path = interpolate_path('/app/v3/api/sites/{siteId}/deployments', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)))
            query = build_query_string([
              QueryParameterSpec.new('page', page, 'form', true, false, nil),
              QueryParameterSpec.new('pageSize', page_size, 'form', true, false, nil),
              QueryParameterSpec.new('status', status, 'form', true, false, nil),
            ])
            path = append_query_string(path, query)
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::DeploymentPage.from_hash(result) : nil
          end

          # 发起部署
          def sites_deployments_create(site_id, body: nil)
            path = interpolate_path('/app/v3/api/sites/{siteId}/deployments', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)))
            payload = body.respond_to?(:to_hash) ? body.to_hash : body
            options = {}
            options[:json] = payload unless payload.nil?
            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::DeploymentResponse.from_hash(result) : nil
          end

          # 获取部署详情
          def sites_deployments_retrieve(site_id, deployment_id)
            path = interpolate_path('/app/v3/api/sites/{siteId}/deployments/{deploymentId}', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)), deploymentId: serialize_path_parameter(deployment_id, PathParameterSpec.new('deploymentId', 'simple', false)))
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::DeploymentResponse.from_hash(result) : nil
          end

          # 回滚部署
          def sites_deployments_rollback(site_id, deployment_id)
            path = interpolate_path('/app/v3/api/sites/{siteId}/deployments/{deploymentId}/rollback', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)), deploymentId: serialize_path_parameter(deployment_id, PathParameterSpec.new('deploymentId', 'simple', false)))
            options = {}

            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::DeploymentResponse.from_hash(result) : nil
          end

      end
    end
  end
end
