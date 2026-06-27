package com.sdkwork.web.app.sdk

data class CreateDeploymentRequest(
    val deployType: Int? = null,
    val versionTag: String? = null,
    val commitHash: String? = null,
    val sourceRef: String? = null,
    val environment: String? = null,
    val idempotencyKey: String? = null
)
