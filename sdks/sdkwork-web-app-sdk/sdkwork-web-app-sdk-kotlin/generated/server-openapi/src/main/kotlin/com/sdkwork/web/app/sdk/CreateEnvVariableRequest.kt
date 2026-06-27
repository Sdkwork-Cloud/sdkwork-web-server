package com.sdkwork.web.app.sdk

data class CreateEnvVariableRequest(
    val key: String? = null,
    val value_: String? = null,
    val environment: String? = null,
    val isSecret: Boolean? = null
)
