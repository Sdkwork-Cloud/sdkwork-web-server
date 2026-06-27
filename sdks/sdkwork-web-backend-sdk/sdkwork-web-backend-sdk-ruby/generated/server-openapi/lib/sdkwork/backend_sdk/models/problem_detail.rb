module Sdkwork
  module BackendSdk
    module Models
      class ProblemDetail
              attr_accessor :type, :title, :status, :detail, :instance, :request_id

              def initialize(attributes = {})
                attributes = (attributes || {}).transform_keys(&:to_s)
                @type = attributes['type']
                @title = attributes['title']
                @status = attributes['status']
                @detail = attributes['detail']
                @instance = attributes['instance']
                @request_id = attributes['requestId']
              end

              def self.from_hash(data)
                return nil if data.nil?

                new(data)
              end

              def to_hash
                {
                  'type' => @type,
                  'title' => @title,
                  'status' => @status,
                  'detail' => @detail,
                  'instance' => @instance,
                  'requestId' => @request_id,
                }
              end
            end
    end
  end
end
