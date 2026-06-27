require_relative 'base_api'
require_relative '../models/create_domain_request'
require_relative '../models/domain_page'
require_relative '../models/domain_response'
require_relative '../models/domain_verify_response'

module Sdkwork
  module AppSdk
    module Api
      class DomainApi < BaseApi
          # 获取站点域名列表
          def sites_domains_list(site_id, page: nil, page_size: nil)
            path = interpolate_path('/app/v3/api/sites/{siteId}/domains', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)))
            query = build_query_string([
              QueryParameterSpec.new('page', page, 'form', true, false, nil),
              QueryParameterSpec.new('pageSize', page_size, 'form', true, false, nil),
            ])
            path = append_query_string(path, query)
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::DomainPage.from_hash(result) : nil
          end

          # 绑定域名
          def sites_domains_create(site_id, body: nil)
            path = interpolate_path('/app/v3/api/sites/{siteId}/domains', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)))
            payload = body.respond_to?(:to_hash) ? body.to_hash : body
            options = {}
            options[:json] = payload unless payload.nil?
            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::DomainResponse.from_hash(result) : nil
          end

          # 获取域名详情
          def sites_domains_retrieve(site_id, domain_id)
            path = interpolate_path('/app/v3/api/sites/{siteId}/domains/{domainId}', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)), domainId: serialize_path_parameter(domain_id, PathParameterSpec.new('domainId', 'simple', false)))
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::DomainResponse.from_hash(result) : nil
          end

          # 解绑域名
          def sites_domains_delete(site_id, domain_id)
            path = interpolate_path('/app/v3/api/sites/{siteId}/domains/{domainId}', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)), domainId: serialize_path_parameter(domain_id, PathParameterSpec.new('domainId', 'simple', false)))
            options = {}

            result = @client.request('DELETE', path, **options)
            result
          end

          # 验证域名所有权
          def sites_domains_verify(site_id, domain_id)
            path = interpolate_path('/app/v3/api/sites/{siteId}/domains/{domainId}/verify', siteId: serialize_path_parameter(site_id, PathParameterSpec.new('siteId', 'simple', false)), domainId: serialize_path_parameter(domain_id, PathParameterSpec.new('domainId', 'simple', false)))
            options = {}

            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::DomainVerifyResponse.from_hash(result) : nil
          end

      end
    end
  end
end
