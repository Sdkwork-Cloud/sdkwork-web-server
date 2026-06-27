module Sdkwork
  module BackendSdk
    class SdkworkBackendClient
      attr_reader :http, :nginx, :server, :agent, :audit
      def initialize(config)
        @http = Http::Client.new(config)
        @nginx = Api::NginxApi.new(@http)
        @server = Api::ServerApi.new(@http)
        @agent = Api::AgentApi.new(@http)
        @audit = Api::AuditApi.new(@http)
      end

      def set_api_key(api_key)
        @http.set_api_key(api_key)
        self
      end

      def set_auth_token(token)
        @http.set_auth_token(token)
        self
      end

      def set_access_token(token)
        @http.set_access_token(token)
        self
      end

      def set_header(key, value)
        @http.set_header(key, value)
        self
      end
    end
  end
end
