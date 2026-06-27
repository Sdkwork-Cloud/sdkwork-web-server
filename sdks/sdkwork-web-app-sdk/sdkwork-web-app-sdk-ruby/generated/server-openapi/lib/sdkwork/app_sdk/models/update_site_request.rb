module Sdkwork
  module AppSdk
    module Models
      class UpdateSiteRequest
              attr_accessor :name, :description, :runtime_config

              def initialize(attributes = {})
                attributes = (attributes || {}).transform_keys(&:to_s)
                @name = attributes['name']
                @description = attributes['description']
                @runtime_config = attributes['runtimeConfig'].is_a?(Hash) ? attributes['runtimeConfig'] : {}
              end

              def self.from_hash(data)
                return nil if data.nil?

                new(data)
              end

              def to_hash
                {
                  'name' => @name,
                  'description' => @description,
                  'runtimeConfig' => @runtime_config,
                }
              end
            end
    end
  end
end
