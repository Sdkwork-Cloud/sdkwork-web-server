package com.sdkwork.web.backend.sdk

data class AgentHeartbeatRequest(
    val agentVersion: String? = null,
    val nginxEnabled: Boolean? = null,
    val activeConfigs: String? = null,
    val lastSyncVersion: String? = null
)
