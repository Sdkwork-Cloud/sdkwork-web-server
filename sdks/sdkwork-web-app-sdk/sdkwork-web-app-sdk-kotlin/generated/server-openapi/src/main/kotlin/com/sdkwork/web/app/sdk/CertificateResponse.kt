package com.sdkwork.web.app.sdk

data class CertificateResponse(
    val id: String? = null,
    val certName: String? = null,
    val certType: Int? = null,
    val issuer: String? = null,
    val notBefore: String? = null,
    val notAfter: String? = null,
    val autoRenew: Boolean? = null,
    val status: Int? = null,
    val createdAt: String? = null
)
