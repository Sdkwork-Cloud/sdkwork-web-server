package com.sdkwork.web.app.sdk

import com.sdkwork.common.core.SdkConfig
import com.sdkwork.web.app.sdk.http.HttpClient
import com.sdkwork.web.app.sdk.api.SiteApi
import com.sdkwork.web.app.sdk.api.DomainApi
import com.sdkwork.web.app.sdk.api.DeploymentApi
import com.sdkwork.web.app.sdk.api.EnvVariableApi
import com.sdkwork.web.app.sdk.api.CertificateApi
import com.sdkwork.web.app.sdk.api.MonitorApi

open class SdkworkAppClient {
    private val httpClient: HttpClient

    lateinit var site: SiteApi
    lateinit var domain: DomainApi
    lateinit var deployment: DeploymentApi
    lateinit var envVariable: EnvVariableApi
    lateinit var certificate: CertificateApi
    lateinit var monitor: MonitorApi

    constructor(baseUrl: String) {
        this.httpClient = HttpClient(baseUrl)
        site = SiteApi(httpClient)
        domain = DomainApi(httpClient)
        deployment = DeploymentApi(httpClient)
        envVariable = EnvVariableApi(httpClient)
        certificate = CertificateApi(httpClient)
        monitor = MonitorApi(httpClient)
    }

    constructor(config: SdkConfig) {
        this.httpClient = HttpClient(config)
        site = SiteApi(httpClient)
        domain = DomainApi(httpClient)
        deployment = DeploymentApi(httpClient)
        envVariable = EnvVariableApi(httpClient)
        certificate = CertificateApi(httpClient)
        monitor = MonitorApi(httpClient)
    }
    fun setAuthToken(token: String): SdkworkAppClient {
        httpClient.setAuthToken(token)
        return this
    }

    fun setAccessToken(token: String): SdkworkAppClient {
        httpClient.setAccessToken(token)
        return this
    }

    fun setHeader(key: String, value: String): SdkworkAppClient {
        httpClient.setHeader(key, value)
        return this
    }
}
