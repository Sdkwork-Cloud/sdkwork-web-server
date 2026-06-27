package com.sdkwork.web.app.sdk

data class UpdateSiteRequest(
    val name: String? = null,
    val description: String? = null,
    val runtimeConfig: Map<String, Any>? = null
)
