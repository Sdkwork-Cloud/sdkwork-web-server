module Sdkwork
  module AppSdk
    class SdkworkAppClient
      attr_reader :http, :site, :domain, :deployment, :env_variable, :certificate, :monitor
      def initialize(config)
        @http = Http::Client.new(config)
        @site = Api::SiteApi.new(@http)
        @domain = Api::DomainApi.new(@http)
        @deployment = Api::DeploymentApi.new(@http)
        @env_variable = Api::EnvVariableApi.new(@http)
        @certificate = Api::CertificateApi.new(@http)
        @monitor = Api::MonitorApi.new(@http)
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
