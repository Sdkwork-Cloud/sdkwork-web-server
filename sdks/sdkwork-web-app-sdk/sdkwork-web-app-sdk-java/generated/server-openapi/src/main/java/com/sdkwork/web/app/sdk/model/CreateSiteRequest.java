package com.sdkwork.web.app.sdk.model;

import java.util.Map;

public class CreateSiteRequest {
    private String name;
    private String slug;
    private String description;
    private Integer siteType;
    private Map<String, Object> runtimeConfig;

    public String getName() {
        return this.name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getSlug() {
        return this.slug;
    }

    public void setSlug(String slug) {
        this.slug = slug;
    }

    public String getDescription() {
        return this.description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public Integer getSiteType() {
        return this.siteType;
    }

    public void setSiteType(Integer siteType) {
        this.siteType = siteType;
    }

    public Map<String, Object> getRuntimeConfig() {
        return this.runtimeConfig;
    }

    public void setRuntimeConfig(Map<String, Object> runtimeConfig) {
        this.runtimeConfig = runtimeConfig;
    }
}
