require_relative 'base_api'
require_relative '../models/audit_log_page'

module Sdkwork
  module BackendSdk
    module Api
      class AuditApi < BaseApi
          # 获取审计日志列表
          def logs_list(page: nil, page_size: nil, target_type: nil, action: nil, operator_id: nil, start_date: nil, end_date: nil)
            path = '/backend/v3/api/audit_logs'
            query = build_query_string([
              QueryParameterSpec.new('page', page, 'form', true, false, nil),
              QueryParameterSpec.new('pageSize', page_size, 'form', true, false, nil),
              QueryParameterSpec.new('targetType', target_type, 'form', true, false, nil),
              QueryParameterSpec.new('action', action, 'form', true, false, nil),
              QueryParameterSpec.new('operatorId', operator_id, 'form', true, false, nil),
              QueryParameterSpec.new('startDate', start_date, 'form', true, false, nil),
              QueryParameterSpec.new('endDate', end_date, 'form', true, false, nil),
            ])
            path = append_query_string(path, query)
            options = {}

            result = @client.request('GET', path, **options)
            result.is_a?(Hash) ? Models::AuditLogPage.from_hash(result) : nil
          end

      end
    end
  end
end
