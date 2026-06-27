package com.sdkwork.web.app.sdk

data class HealthCheckResponse(
    val id: String? = null,
    val checkType: Int? = null,
    val checkUrl: String? = null,
    val checkInterval: Int? = null,
    val status: Int? = null,
    val createdAt: String? = null
)
