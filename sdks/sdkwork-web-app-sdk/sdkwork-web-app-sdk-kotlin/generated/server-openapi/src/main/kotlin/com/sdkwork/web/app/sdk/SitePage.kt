package com.sdkwork.web.app.sdk

data class SitePage(
    val items: List<SiteResponse>? = null,
    val total: String? = null,
    val page: Int? = null,
    val pageSize: Int? = null
)
