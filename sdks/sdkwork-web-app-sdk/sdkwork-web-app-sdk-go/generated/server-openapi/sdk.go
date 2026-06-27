package app

import (
    "github.com/sdkwork/sdkwork-web-app-sdk/api"
    sdkhttp "github.com/sdkwork/sdkwork-web-app-sdk/http"
)

type SdkworkAppClient struct {
    http *sdkhttp.Client
    Site *api.SiteApi
    Domain *api.DomainApi
    Deployment *api.DeploymentApi
    EnvVariable *api.EnvVariableApi
    Certificate *api.CertificateApi
    Monitor *api.MonitorApi
}

func NewSdkworkAppClient(baseURL string) *SdkworkAppClient {
    cfg := sdkhttp.NewDefaultConfig(baseURL)
    return NewSdkworkAppClientWithConfig(cfg)
}

func NewSdkworkAppClientWithConfig(config sdkhttp.Config) *SdkworkAppClient {
    client := sdkhttp.NewClient(config)
    return &SdkworkAppClient{
        http: client,
        Site: api.NewSiteApi(client),
        Domain: api.NewDomainApi(client),
        Deployment: api.NewDeploymentApi(client),
        EnvVariable: api.NewEnvVariableApi(client),
        Certificate: api.NewCertificateApi(client),
        Monitor: api.NewMonitorApi(client),
    }
}

func (c *SdkworkAppClient) SetAuthToken(token string) *SdkworkAppClient {
    c.http.SetAuthToken(token)
    return c
}

func (c *SdkworkAppClient) SetAccessToken(token string) *SdkworkAppClient {
    c.http.SetAccessToken(token)
    return c
}

func (c *SdkworkAppClient) SetHeader(key string, value string) *SdkworkAppClient {
    c.http.SetHeader(key, value)
    return c
}

func (c *SdkworkAppClient) Http() *sdkhttp.Client {
    return c.http
}
