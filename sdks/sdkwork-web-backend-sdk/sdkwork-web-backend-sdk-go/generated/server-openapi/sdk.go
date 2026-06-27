package backend

import (
    "github.com/sdkwork/sdkwork-web-backend-sdk/api"
    sdkhttp "github.com/sdkwork/sdkwork-web-backend-sdk/http"
)

type SdkworkBackendClient struct {
    http *sdkhttp.Client
    Nginx *api.NginxApi
    Server *api.ServerApi
    Agent *api.AgentApi
    Audit *api.AuditApi
}

func NewSdkworkBackendClient(baseURL string) *SdkworkBackendClient {
    cfg := sdkhttp.NewDefaultConfig(baseURL)
    return NewSdkworkBackendClientWithConfig(cfg)
}

func NewSdkworkBackendClientWithConfig(config sdkhttp.Config) *SdkworkBackendClient {
    client := sdkhttp.NewClient(config)
    return &SdkworkBackendClient{
        http: client,
        Nginx: api.NewNginxApi(client),
        Server: api.NewServerApi(client),
        Agent: api.NewAgentApi(client),
        Audit: api.NewAuditApi(client),
    }
}

func (c *SdkworkBackendClient) SetAuthToken(token string) *SdkworkBackendClient {
    c.http.SetAuthToken(token)
    return c
}

func (c *SdkworkBackendClient) SetAccessToken(token string) *SdkworkBackendClient {
    c.http.SetAccessToken(token)
    return c
}

func (c *SdkworkBackendClient) SetHeader(key string, value string) *SdkworkBackendClient {
    c.http.SetHeader(key, value)
    return c
}

func (c *SdkworkBackendClient) Http() *sdkhttp.Client {
    return c.http
}
