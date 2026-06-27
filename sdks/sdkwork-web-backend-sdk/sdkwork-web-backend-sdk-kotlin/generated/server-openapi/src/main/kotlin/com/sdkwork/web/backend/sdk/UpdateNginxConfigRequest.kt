package com.sdkwork.web.backend.sdk

data class UpdateNginxConfigRequest(
    val configContent: String? = null,
    val configName: String? = null
)
