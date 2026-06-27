module Sdkwork
  module AppSdk
    module Models
      class DeploymentResponse
              attr_accessor :id, :site_id, :deploy_type, :version_tag, :status, :started_at, :completed_at, :duration_ms, :created_at

              def initialize(attributes = {})
                attributes = (attributes || {}).transform_keys(&:to_s)
                @id = attributes['id']
                @site_id = attributes['siteId']
                @deploy_type = attributes['deployType']
                @version_tag = attributes['versionTag']
                @status = attributes['status']
                @started_at = attributes['startedAt']
                @completed_at = attributes['completedAt']
                @duration_ms = attributes['durationMs']
                @created_at = attributes['createdAt']
              end

              def self.from_hash(data)
                return nil if data.nil?

                new(data)
              end

              def to_hash
                {
                  'id' => @id,
                  'siteId' => @site_id,
                  'deployType' => @deploy_type,
                  'versionTag' => @version_tag,
                  'status' => @status,
                  'startedAt' => @started_at,
                  'completedAt' => @completed_at,
                  'durationMs' => @duration_ms,
                  'createdAt' => @created_at,
                }
              end
            end
    end
  end
end
