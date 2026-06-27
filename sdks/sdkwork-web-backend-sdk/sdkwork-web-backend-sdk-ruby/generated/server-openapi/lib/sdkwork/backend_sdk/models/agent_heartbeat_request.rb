module Sdkwork
  module BackendSdk
    module Models
      class AgentHeartbeatRequest
              attr_accessor :agent_version, :nginx_enabled, :active_configs, :last_sync_version

              def initialize(attributes = {})
                attributes = (attributes || {}).transform_keys(&:to_s)
                @agent_version = attributes['agentVersion']
                @nginx_enabled = attributes['nginxEnabled']
                @active_configs = attributes['activeConfigs']
                @last_sync_version = attributes['lastSyncVersion']
              end

              def self.from_hash(data)
                return nil if data.nil?

                new(data)
              end

              def to_hash
                {
                  'agentVersion' => @agent_version,
                  'nginxEnabled' => @nginx_enabled,
                  'activeConfigs' => @active_configs,
                  'lastSyncVersion' => @last_sync_version,
                }
              end
            end
    end
  end
end
