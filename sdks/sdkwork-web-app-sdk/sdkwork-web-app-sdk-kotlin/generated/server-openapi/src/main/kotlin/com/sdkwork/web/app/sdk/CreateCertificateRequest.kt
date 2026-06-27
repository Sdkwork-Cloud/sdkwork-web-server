package com.sdkwork.web.app.sdk

data class CreateCertificateRequest(
    val domainId: String? = null,
    val certType: Int? = null,
    val autoRenew: Boolean? = null
)
