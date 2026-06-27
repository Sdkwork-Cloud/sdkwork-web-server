package com.sdkwork.web.backend.sdk

data class ServerResponse(
    val id: String? = null,
    val name: String? = null,
    val host: String? = null,
    val sshPort: Int? = null,
    val status: Int? = null,
    val lastHeartbeatAt: String? = null,
    val createdAt: String? = null
)
