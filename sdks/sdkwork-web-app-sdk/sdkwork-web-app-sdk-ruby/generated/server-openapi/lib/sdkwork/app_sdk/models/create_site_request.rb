module Sdkwork
  module AppSdk
    module Models
      class CreateSiteRequest
              attr_accessor :name, :slug, :description, :site_type, :runtime_config

              def initialize(attributes = {})
                attributes = (attributes || {}).transform_keys(&:to_s)
                @name = attributes['name']
                @slug = attributes['slug']
                @description = attributes['description']
                @site_type = attributes['siteType']
                @runtime_config = attributes['runtimeConfig'].is_a?(Hash) ? attributes['runtimeConfig'] : {}
              end

              def self.from_hash(data)
                return nil if data.nil?

                new(data)
              end

              def to_hash
                {
                  'name' => @name,
                  'slug' => @slug,
                  'description' => @description,
                  'siteType' => @site_type,
                  'runtimeConfig' => @runtime_config,
                }
              end
            end
    end
  end
end
