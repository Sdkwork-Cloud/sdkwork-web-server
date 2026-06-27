package com.sdkwork.web.app.sdk;

import com.sdkwork.common.core.Types;
import com.sdkwork.web.app.sdk.http.HttpClient;
import com.sdkwork.web.app.sdk.api.SiteApi;
import com.sdkwork.web.app.sdk.api.DomainApi;
import com.sdkwork.web.app.sdk.api.DeploymentApi;
import com.sdkwork.web.app.sdk.api.EnvVariableApi;
import com.sdkwork.web.app.sdk.api.CertificateApi;
import com.sdkwork.web.app.sdk.api.MonitorApi;

public class SdkworkAppClient {
    private final HttpClient httpClient;
    private SiteApi site;
    private DomainApi domain;
    private DeploymentApi deployment;
    private EnvVariableApi envVariable;
    private CertificateApi certificate;
    private MonitorApi monitor;

    public SdkworkAppClient(String baseUrl) {
        this.httpClient = new HttpClient(baseUrl);
        this.site = new SiteApi(httpClient);
        this.domain = new DomainApi(httpClient);
        this.deployment = new DeploymentApi(httpClient);
        this.envVariable = new EnvVariableApi(httpClient);
        this.certificate = new CertificateApi(httpClient);
        this.monitor = new MonitorApi(httpClient);
    }

    public SdkworkAppClient(Types.SdkConfig config) {
        this.httpClient = new HttpClient(config);
        this.site = new SiteApi(httpClient);
        this.domain = new DomainApi(httpClient);
        this.deployment = new DeploymentApi(httpClient);
        this.envVariable = new EnvVariableApi(httpClient);
        this.certificate = new CertificateApi(httpClient);
        this.monitor = new MonitorApi(httpClient);
    }

    public SiteApi getSite() {
        return this.site;
    }

    public DomainApi getDomain() {
        return this.domain;
    }

    public DeploymentApi getDeployment() {
        return this.deployment;
    }

    public EnvVariableApi getEnvVariable() {
        return this.envVariable;
    }

    public CertificateApi getCertificate() {
        return this.certificate;
    }

    public MonitorApi getMonitor() {
        return this.monitor;
    }
    public SdkworkAppClient setAuthToken(String token) {
        httpClient.setAuthToken(token);
        return this;
    }

    public SdkworkAppClient setAccessToken(String token) {
        httpClient.setAccessToken(token);
        return this;
    }

    public SdkworkAppClient setHeader(String key, String value) {
        httpClient.setHeader(key, value);
        return this;
    }

    public HttpClient getHttpClient() {
        return httpClient;
    }
}
