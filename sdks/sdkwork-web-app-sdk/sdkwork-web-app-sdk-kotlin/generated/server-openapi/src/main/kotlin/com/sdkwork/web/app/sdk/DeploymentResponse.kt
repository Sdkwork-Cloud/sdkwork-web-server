package com.sdkwork.web.app.sdk

data class DeploymentResponse(
    val id: String? = null,
    val siteId: String? = null,
    val deployType: Int? = null,
    val versionTag: String? = null,
    val status: Int? = null,
    val startedAt: String? = null,
    val completedAt: String? = null,
    val durationMs: String? = null,
    val createdAt: String? = null
)
