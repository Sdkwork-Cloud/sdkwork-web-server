package com.sdkwork.web.backend.sdk.model;


public class AgentHeartbeatRequest {
    private String agentVersion;
    private Boolean nginxEnabled;
    private String activeConfigs;
    private String lastSyncVersion;

    public String getAgentVersion() {
        return this.agentVersion;
    }

    public void setAgentVersion(String agentVersion) {
        this.agentVersion = agentVersion;
    }

    public Boolean getNginxEnabled() {
        return this.nginxEnabled;
    }

    public void setNginxEnabled(Boolean nginxEnabled) {
        this.nginxEnabled = nginxEnabled;
    }

    public String getActiveConfigs() {
        return this.activeConfigs;
    }

    public void setActiveConfigs(String activeConfigs) {
        this.activeConfigs = activeConfigs;
    }

    public String getLastSyncVersion() {
        return this.lastSyncVersion;
    }

    public void setLastSyncVersion(String lastSyncVersion) {
        this.lastSyncVersion = lastSyncVersion;
    }
}
