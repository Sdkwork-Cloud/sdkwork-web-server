package com.sdkwork.web.backend.sdk

data class CreateNginxConfigRequest(
    val configType: Int? = null,
    val configName: String? = null,
    val configContent: String? = null,
    val siteId: String? = null,
    val domainId: String? = null
)
