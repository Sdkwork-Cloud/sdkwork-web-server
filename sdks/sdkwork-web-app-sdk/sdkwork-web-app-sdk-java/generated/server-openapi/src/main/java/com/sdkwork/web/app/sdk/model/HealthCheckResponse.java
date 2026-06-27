package com.sdkwork.web.app.sdk.model;


public class HealthCheckResponse {
    private String id;
    private Integer checkType;
    private String checkUrl;
    private Integer checkInterval;
    private Integer status;
    private String createdAt;

    public String getId() {
        return this.id;
    }

    public void setId(String id) {
        this.id = id;
    }

    public Integer getCheckType() {
        return this.checkType;
    }

    public void setCheckType(Integer checkType) {
        this.checkType = checkType;
    }

    public String getCheckUrl() {
        return this.checkUrl;
    }

    public void setCheckUrl(String checkUrl) {
        this.checkUrl = checkUrl;
    }

    public Integer getCheckInterval() {
        return this.checkInterval;
    }

    public void setCheckInterval(Integer checkInterval) {
        this.checkInterval = checkInterval;
    }

    public Integer getStatus() {
        return this.status;
    }

    public void setStatus(Integer status) {
        this.status = status;
    }

    public String getCreatedAt() {
        return this.createdAt;
    }

    public void setCreatedAt(String createdAt) {
        this.createdAt = createdAt;
    }
}
