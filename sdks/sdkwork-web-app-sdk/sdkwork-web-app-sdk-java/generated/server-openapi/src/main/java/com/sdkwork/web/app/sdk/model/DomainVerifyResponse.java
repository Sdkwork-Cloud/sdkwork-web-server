package com.sdkwork.web.app.sdk.model;


public class DomainVerifyResponse {
    private Boolean verified;
    private String method;
    private String token;

    public Boolean getVerified() {
        return this.verified;
    }

    public void setVerified(Boolean verified) {
        this.verified = verified;
    }

    public String getMethod() {
        return this.method;
    }

    public void setMethod(String method) {
        this.method = method;
    }

    public String getToken() {
        return this.token;
    }

    public void setToken(String token) {
        this.token = token;
    }
}
