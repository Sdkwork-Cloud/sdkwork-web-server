package com.sdkwork.web.backend.sdk

data class CreateServerRequest(
    val name: String? = null,
    val host: String? = null,
    val sshPort: Int? = null,
    val sshUser: String? = null,
    val sshKeyPath: String? = null,
    val description: String? = null
)
