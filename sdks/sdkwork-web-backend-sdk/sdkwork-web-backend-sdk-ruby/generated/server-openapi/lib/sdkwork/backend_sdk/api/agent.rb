require_relative 'base_api'
require_relative '../models/agent_heartbeat_request'
require_relative '../models/agent_heartbeat_response'
require_relative '../models/agent_sync_response'

module Sdkwork
  module BackendSdk
    module Api
      class AgentApi < BaseApi
          # 边缘节点心跳
          def heartbeat(body: nil)
            path = '/backend/v3/api/agent/heartbeat'
            payload = body.respond_to?(:to_hash) ? body.to_hash : body
            options = {}
            options[:json] = payload unless payload.nil?
            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::AgentHeartbeatResponse.from_hash(result) : nil
          end

          # 拉取 nginx 配置与证书 bundle
          def sync(if_sync_version: nil)
            path = '/backend/v3/api/agent/sync'
            query = build_query_string([
              QueryParameterSpec.new('ifSyncVersion', if_sync_version, 'form', true, false, nil),
            ])
            path = append_query_string(path, query)
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::AgentSyncResponse.from_hash(result) : nil
          end

      end
    end
  end
end
