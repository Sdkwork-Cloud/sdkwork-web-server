require_relative 'base_api'
require_relative '../models/create_site_request'
require_relative '../models/site_page'
require_relative '../models/site_response'
require_relative '../models/update_site_request'

module Sdkwork
  module AppSdk
    module Api
      class SiteApi < BaseApi
          # 获取站点列表
          def sites_list(page: nil, page_size: nil, status: nil, site_type: nil, keyword: nil)
            path = '/app/v3/api/sites'
            query = build_query_string([
              QueryParameterSpec.new('page', page, 'form', true, false, nil),
              QueryParameterSpec.new('pageSize', page_size, 'form', true, false, nil),
              QueryParameterSpec.new('status', status, 'form', true, false, nil),
              QueryParameterSpec.new('siteType', site_type, 'form', true, false, nil),
              QueryParameterSpec.new('keyword', keyword, 'form', true, false, nil),
            ])
            path = append_query_string(path, query)
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::SitePage.from_hash(result) : nil
          end

          # 创建站点
          def sites_create(body: nil)
            path = '/app/v3/api/sites'
            payload = body.respond_to?(:to_hash) ? body.to_hash : body
            options = {}
            options[:json] = payload unless payload.nil?
            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::SiteResponse.from_hash(result) : nil
          end

          # 获取站点详情
          def sites_retrieve(site_id)
            path = interpolate_path('/app/v3/api/sites/{siteId}', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)))
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::SiteResponse.from_hash(result) : nil
          end

          # 更新站点
          def sites_update(site_id, body: nil)
            path = interpolate_path('/app/v3/api/sites/{siteId}', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)))
            payload = body.respond_to?(:to_hash) ? body.to_hash : body
            options = {}
            options[:json] = payload unless payload.nil?
            result = @client.request('PATCH', path, **options)
            result.is_a?(Hash) ? Models::SiteResponse.from_hash(result) : nil
          end

          # 删除站点
          def sites_delete(site_id)
            path = interpolate_path('/app/v3/api/sites/{siteId}', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)))
            options = {}

            result = @client.request('DELETE', path, **options)
            result
          end

          # 激活站点
          def sites_activate(site_id)
            path = interpolate_path('/app/v3/api/sites/{siteId}/activate', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)))
            options = {}

            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::SiteResponse.from_hash(result) : nil
          end

          # 暂停站点
          def sites_pause(site_id)
            path = interpolate_path('/app/v3/api/sites/{siteId}/pause', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)))
            options = {}

            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::SiteResponse.from_hash(result) : nil
          end

      end
    end
  end
end
