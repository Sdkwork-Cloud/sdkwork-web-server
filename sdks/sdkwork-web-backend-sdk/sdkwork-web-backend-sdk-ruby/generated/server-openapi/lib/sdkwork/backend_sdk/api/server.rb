require_relative 'base_api'
require_relative '../models/create_server_request'
require_relative '../models/create_server_response'
require_relative '../models/server_page'

module Sdkwork
  module BackendSdk
    module Api
      class ServerApi < BaseApi
          # 获取服务器列表
          def servers_list(page: nil, page_size: nil)
            path = '/backend/v3/api/servers'
            query = build_query_string([
              QueryParameterSpec.new('page', page, 'form', true, false, nil),
              QueryParameterSpec.new('pageSize', page_size, 'form', true, false, nil),
            ])
            path = append_query_string(path, query)
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::ServerPage.from_hash(result) : nil
          end

          # 注册服务器
          def servers_create(body: nil)
            path = '/backend/v3/api/servers'
            payload = body.respond_to?(:to_hash) ? body.to_hash : body
            options = {}
            options[:json] = payload unless payload.nil?
            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::CreateServerResponse.from_hash(result) : nil
          end

      end
    end
  end
end
