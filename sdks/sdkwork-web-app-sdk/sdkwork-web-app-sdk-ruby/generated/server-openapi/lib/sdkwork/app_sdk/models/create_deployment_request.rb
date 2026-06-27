module Sdkwork
  module AppSdk
    module Models
      class CreateDeploymentRequest
              attr_accessor :deploy_type, :version_tag, :commit_hash, :source_ref, :environment, :idempotency_key

              def initialize(attributes = {})
                attributes = (attributes || {}).transform_keys(&:to_s)
                @deploy_type = attributes['deployType']
                @version_tag = attributes['versionTag']
                @commit_hash = attributes['commitHash']
                @source_ref = attributes['sourceRef']
                @environment = attributes['environment']
                @idempotency_key = attributes['idempotencyKey']
              end

              def self.from_hash(data)
                return nil if data.nil?

                new(data)
              end

              def to_hash
                {
                  'deployType' => @deploy_type,
                  'versionTag' => @version_tag,
                  'commitHash' => @commit_hash,
                  'sourceRef' => @source_ref,
                  'environment' => @environment,
                  'idempotencyKey' => @idempotency_key,
                }
              end
            end
    end
  end
end
