package com.sdkwork.web.app.sdk.model;

import java.util.Map;

public class UpdateSiteRequest {
    private String name;
    private String description;
    private Map<String, Object> runtimeConfig;

    public String getName() {
        return this.name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getDescription() {
        return this.description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public Map<String, Object> getRuntimeConfig() {
        return this.runtimeConfig;
    }

    public void setRuntimeConfig(Map<String, Object> runtimeConfig) {
        this.runtimeConfig = runtimeConfig;
    }
}
