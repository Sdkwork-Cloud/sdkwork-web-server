using System;
using SDKwork.Common.Core;
using SdkHttpClient = SDKWork.Web.BackendSdk.Http.HttpClient;
using SDKWork.Web.BackendSdk.Api;

namespace SDKWork.Web.BackendSdk
{
    public class SdkworkBackendClient
    {
        private readonly SdkHttpClient _httpClient;

        public NginxApi Nginx { get; }
        public ServerApi Server { get; }
        public AgentApi Agent { get; }
        public AuditApi Audit { get; }

        public SdkworkBackendClient(string baseUrl)
        {
            _httpClient = new SdkHttpClient(baseUrl);
            Nginx = new NginxApi(_httpClient);
            Server = new ServerApi(_httpClient);
            Agent = new AgentApi(_httpClient);
            Audit = new AuditApi(_httpClient);
        }

        public SdkworkBackendClient(SdkConfig config)
        {
            _httpClient = new SdkHttpClient(config);
            Nginx = new NginxApi(_httpClient);
            Server = new ServerApi(_httpClient);
            Agent = new AgentApi(_httpClient);
            Audit = new AuditApi(_httpClient);
        }
        public SdkworkBackendClient SetAuthToken(string token)
        {
            _httpClient.SetAuthToken(token);
            return this;
        }

        public SdkworkBackendClient SetAccessToken(string token)
        {
            _httpClient.SetAccessToken(token);
            return this;
        }

        public SdkworkBackendClient SetHeader(string key, string value)
        {
            _httpClient.SetHeader(key, value);
            return this;
        }
    }
}
