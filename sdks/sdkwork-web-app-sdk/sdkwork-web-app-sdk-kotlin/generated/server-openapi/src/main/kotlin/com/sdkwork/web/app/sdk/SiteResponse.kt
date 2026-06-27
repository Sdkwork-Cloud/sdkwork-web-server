package com.sdkwork.web.app.sdk

data class SiteResponse(
    val id: String? = null,
    val name: String? = null,
    val slug: String? = null,
    val description: String? = null,
    val siteType: Int? = null,
    val status: Int? = null,
    val runtimeConfig: Map<String, Any>? = null,
    val createdAt: String? = null,
    val updatedAt: String? = null
)
