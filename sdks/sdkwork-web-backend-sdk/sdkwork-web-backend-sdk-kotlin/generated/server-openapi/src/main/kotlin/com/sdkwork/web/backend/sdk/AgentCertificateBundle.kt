package com.sdkwork.web.backend.sdk

data class AgentCertificateBundle(
    val certificateId: String? = null,
    val certName: String? = null,
    val fingerprint: String? = null,
    val fullchainPem: String? = null,
    val privkeyPem: String? = null
)
