package com.sdkwork.web.backend.sdk.model;


public class CreateNginxConfigRequest {
    private Integer configType;
    private String configName;
    private String configContent;
    private String siteId;
    private String domainId;

    public Integer getConfigType() {
        return this.configType;
    }

    public void setConfigType(Integer configType) {
        this.configType = configType;
    }

    public String getConfigName() {
        return this.configName;
    }

    public void setConfigName(String configName) {
        this.configName = configName;
    }

    public String getConfigContent() {
        return this.configContent;
    }

    public void setConfigContent(String configContent) {
        this.configContent = configContent;
    }

    public String getSiteId() {
        return this.siteId;
    }

    public void setSiteId(String siteId) {
        this.siteId = siteId;
    }

    public String getDomainId() {
        return this.domainId;
    }

    public void setDomainId(String domainId) {
        this.domainId = domainId;
    }
}
