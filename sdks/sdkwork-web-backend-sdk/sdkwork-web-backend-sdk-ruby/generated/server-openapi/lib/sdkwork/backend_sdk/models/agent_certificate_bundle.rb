module Sdkwork
  module BackendSdk
    module Models
      class AgentCertificateBundle
              attr_accessor :certificate_id, :cert_name, :fingerprint, :fullchain_pem, :privkey_pem

              def initialize(attributes = {})
                attributes = (attributes || {}).transform_keys(&:to_s)
                @certificate_id = attributes['certificateId']
                @cert_name = attributes['certName']
                @fingerprint = attributes['fingerprint']
                @fullchain_pem = attributes['fullchainPem']
                @privkey_pem = attributes['privkeyPem']
              end

              def self.from_hash(data)
                return nil if data.nil?

                new(data)
              end

              def to_hash
                {
                  'certificateId' => @certificate_id,
                  'certName' => @cert_name,
                  'fingerprint' => @fingerprint,
                  'fullchainPem' => @fullchain_pem,
                  'privkeyPem' => @privkey_pem,
                }
              end
            end
    end
  end
end
