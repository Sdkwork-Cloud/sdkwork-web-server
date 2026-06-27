package com.sdkwork.web.backend.sdk;

import com.sdkwork.common.core.Types;
import com.sdkwork.web.backend.sdk.http.HttpClient;
import com.sdkwork.web.backend.sdk.api.NginxApi;
import com.sdkwork.web.backend.sdk.api.ServerApi;
import com.sdkwork.web.backend.sdk.api.AgentApi;
import com.sdkwork.web.backend.sdk.api.AuditApi;

public class SdkworkBackendClient {
    private final HttpClient httpClient;
    private NginxApi nginx;
    private ServerApi server;
    private AgentApi agent;
    private AuditApi audit;

    public SdkworkBackendClient(String baseUrl) {
        this.httpClient = new HttpClient(baseUrl);
        this.nginx = new NginxApi(httpClient);
        this.server = new ServerApi(httpClient);
        this.agent = new AgentApi(httpClient);
        this.audit = new AuditApi(httpClient);
    }

    public SdkworkBackendClient(Types.SdkConfig config) {
        this.httpClient = new HttpClient(config);
        this.nginx = new NginxApi(httpClient);
        this.server = new ServerApi(httpClient);
        this.agent = new AgentApi(httpClient);
        this.audit = new AuditApi(httpClient);
    }

    public NginxApi getNginx() {
        return this.nginx;
    }

    public ServerApi getServer() {
        return this.server;
    }

    public AgentApi getAgent() {
        return this.agent;
    }

    public AuditApi getAudit() {
        return this.audit;
    }
    public SdkworkBackendClient setAuthToken(String token) {
        httpClient.setAuthToken(token);
        return this;
    }

    public SdkworkBackendClient setAccessToken(String token) {
        httpClient.setAccessToken(token);
        return this;
    }

    public SdkworkBackendClient setHeader(String key, String value) {
        httpClient.setHeader(key, value);
        return this;
    }

    public HttpClient getHttpClient() {
        return httpClient;
    }
}
