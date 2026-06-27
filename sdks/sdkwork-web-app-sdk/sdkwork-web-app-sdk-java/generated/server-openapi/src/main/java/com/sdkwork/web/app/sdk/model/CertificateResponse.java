package com.sdkwork.web.app.sdk.model;


public class CertificateResponse {
    private String id;
    private String certName;
    private Integer certType;
    private String issuer;
    private String notBefore;
    private String notAfter;
    private Boolean autoRenew;
    private Integer status;
    private String createdAt;

    public String getId() {
        return this.id;
    }

    public void setId(String id) {
        this.id = id;
    }

    public String getCertName() {
        return this.certName;
    }

    public void setCertName(String certName) {
        this.certName = certName;
    }

    public Integer getCertType() {
        return this.certType;
    }

    public void setCertType(Integer certType) {
        this.certType = certType;
    }

    public String getIssuer() {
        return this.issuer;
    }

    public void setIssuer(String issuer) {
        this.issuer = issuer;
    }

    public String getNotBefore() {
        return this.notBefore;
    }

    public void setNotBefore(String notBefore) {
        this.notBefore = notBefore;
    }

    public String getNotAfter() {
        return this.notAfter;
    }

    public void setNotAfter(String notAfter) {
        this.notAfter = notAfter;
    }

    public Boolean getAutoRenew() {
        return this.autoRenew;
    }

    public void setAutoRenew(Boolean autoRenew) {
        this.autoRenew = autoRenew;
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
