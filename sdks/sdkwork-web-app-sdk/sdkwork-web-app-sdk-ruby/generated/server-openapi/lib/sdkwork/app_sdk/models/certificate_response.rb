module Sdkwork
  module AppSdk
    module Models
      class CertificateResponse
              attr_accessor :id, :cert_name, :cert_type, :issuer, :not_before, :not_after, :auto_renew, :status, :created_at

              def initialize(attributes = {})
                attributes = (attributes || {}).transform_keys(&:to_s)
                @id = attributes['id']
                @cert_name = attributes['certName']
                @cert_type = attributes['certType']
                @issuer = attributes['issuer']
                @not_before = attributes['notBefore']
                @not_after = attributes['notAfter']
                @auto_renew = attributes['autoRenew']
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
                  'certName' => @cert_name,
                  'certType' => @cert_type,
                  'issuer' => @issuer,
                  'notBefore' => @not_before,
                  'notAfter' => @not_after,
                  'autoRenew' => @auto_renew,
                  'status' => @status,
                  'createdAt' => @created_at,
                }
              end
            end
    end
  end
end
