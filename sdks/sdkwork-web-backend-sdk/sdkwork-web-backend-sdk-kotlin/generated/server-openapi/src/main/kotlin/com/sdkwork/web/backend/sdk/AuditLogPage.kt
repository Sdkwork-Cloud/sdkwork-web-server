package com.sdkwork.web.backend.sdk

data class AuditLogPage(
    val items: List<AuditLogResponse>? = null,
    val total: String? = null
)
