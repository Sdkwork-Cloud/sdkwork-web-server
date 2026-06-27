module Sdkwork
  module AppSdk
    module Models
      class DomainVerifyResponse
              attr_accessor :verified, :method, :token

              def initialize(attributes = {})
                attributes = (attributes || {}).transform_keys(&:to_s)
                @verified = attributes['verified']
                @method = attributes['method']
                @token = attributes['token']
              end

              def self.from_hash(data)
                return nil if data.nil?

                new(data)
              end

              def to_hash
                {
                  'verified' => @verified,
                  'method' => @method,
                  'token' => @token,
                }
              end
            end
    end
  end
end
