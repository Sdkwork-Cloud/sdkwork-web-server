require_relative 'base_api'
require_relative '../models/certificate_page'
require_relative '../models/certificate_response'
require_relative '../models/create_certificate_request'

module Sdkwork
  module AppSdk
    module Api
      class CertificateApi < BaseApi
          # 获取证书列表
          def certificates_list(page: nil, page_size: nil)
            path = '/app/v3/api/certificates'
            query = build_query_string([
              QueryParameterSpec.new('page', page, 'form', true, false, nil),
              QueryParameterSpec.new('pageSize', page_size, 'form', true, false, nil),
            ])
            path = append_query_string(path, query)
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::CertificatePage.from_hash(result) : nil
          end

          # 申请证书
          def certificates_create(body: nil)
            path = '/app/v3/api/certificates'
            payload = body.respond_to?(:to_hash) ? body.to_hash : body
            options = {}
            options[:json] = payload unless payload.nil?
            result = @client.request('POST', path, **options)
            result.is_a?(Hash) ? Models::CertificateResponse.from_hash(result) : nil
          end

      end
    end
  end
end
