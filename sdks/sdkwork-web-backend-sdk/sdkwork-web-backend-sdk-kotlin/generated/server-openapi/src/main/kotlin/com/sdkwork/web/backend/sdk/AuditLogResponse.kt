package com.sdkwork.web.backend.sdk

data class AuditLogResponse(
    val id: String? = null,
    val operatorId: String? = null,
    val operatorType: String? = null,
    val action: String? = null,
    val targetType: String? = null,
    val targetId: String? = null,
    val targetUuid: String? = null,
    val requestId: String? = null,
    val ipAddress: String? = null,
    val changes: Map<String, Any>? = null,
    val createdAt: String? = null
)
