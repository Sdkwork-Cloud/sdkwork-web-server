module Sdkwork
  module AppSdk
    module Models
      class CreateCertificateRequest
              attr_accessor :domain_id, :cert_type, :auto_renew

              def initialize(attributes = {})
                attributes = (attributes || {}).transform_keys(&:to_s)
                @domain_id = attributes['domainId']
                @cert_type = attributes['certType']
                @auto_renew = attributes['autoRenew']
              end

              def self.from_hash(data)
                return nil if data.nil?

                new(data)
              end

              def to_hash
                {
                  'domainId' => @domain_id,
                  'certType' => @cert_type,
                  'autoRenew' => @auto_renew,
                }
              end
            end
    end
  end
end
