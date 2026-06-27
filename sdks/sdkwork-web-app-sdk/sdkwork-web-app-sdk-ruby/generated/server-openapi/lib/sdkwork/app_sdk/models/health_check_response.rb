module Sdkwork
  module AppSdk
    module Models
      class HealthCheckResponse
              attr_accessor :id, :check_type, :check_url, :check_interval, :status, :created_at

              def initialize(attributes = {})
                attributes = (attributes || {}).transform_keys(&:to_s)
                @id = attributes['id']
                @check_type = attributes['checkType']
                @check_url = attributes['checkUrl']
                @check_interval = attributes['checkInterval']
                @status = attributes['status']
                @created_at = attributes['createdAt']
              end

              def self.from_hash(data)
                return nil if data.nil?

                new(data)
              end

              def to_hash
                {
                  'id' => @id,
                  'checkType' => @check_type,
                  'checkUrl' => @check_url,
                  'checkInterval' => @check_interval,
                  'status' => @status,
                  'createdAt' => @created_at,
                }
              end
            end
    end
  end
end
