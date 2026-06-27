package com.sdkwork.web.backend.sdk.api

import com.sdkwork.web.backend.sdk.http.HttpClient

/**
 * API modules for sdkwork-web-backend-sdk
 */
class Api(private val client: HttpClient) {
    val nginx: NginxApi = NginxApi(client)
    val server: ServerApi = ServerApi(client)
    val agent: AgentApi = AgentApi(client)
    val audit: AuditApi = AuditApi(client)
}
