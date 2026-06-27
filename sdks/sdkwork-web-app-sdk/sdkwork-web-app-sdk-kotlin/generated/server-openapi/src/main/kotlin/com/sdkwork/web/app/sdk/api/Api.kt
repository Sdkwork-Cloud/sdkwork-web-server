package com.sdkwork.web.app.sdk.api

import com.sdkwork.web.app.sdk.http.HttpClient

/**
 * API modules for sdkwork-web-app-sdk
 */
class Api(private val client: HttpClient) {
    val site: SiteApi = SiteApi(client)
    val domain: DomainApi = DomainApi(client)
    val deployment: DeploymentApi = DeploymentApi(client)
    val envVariable: EnvVariableApi = EnvVariableApi(client)
    val certificate: CertificateApi = CertificateApi(client)
    val monitor: MonitorApi = MonitorApi(client)
}
