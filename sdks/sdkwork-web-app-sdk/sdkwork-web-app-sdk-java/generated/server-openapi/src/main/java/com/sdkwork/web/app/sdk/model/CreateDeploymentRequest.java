package com.sdkwork.web.app.sdk.model;


public class CreateDeploymentRequest {
    private Integer deployType;
    private String versionTag;
    private String commitHash;
    private String sourceRef;
    private String environment;
    private String idempotencyKey;

    public Integer getDeployType() {
        return this.deployType;
    }

    public void setDeployType(Integer deployType) {
        this.deployType = deployType;
    }

    public String getVersionTag() {
        return this.versionTag;
    }

    public void setVersionTag(String versionTag) {
        this.versionTag = versionTag;
    }

    public String getCommitHash() {
        return this.commitHash;
    }

    public void setCommitHash(String commitHash) {
        this.commitHash = commitHash;
    }

    public String getSourceRef() {
        return this.sourceRef;
    }

    public void setSourceRef(String sourceRef) {
        this.sourceRef = sourceRef;
    }

    public String getEnvironment() {
        return this.environment;
    }

    public void setEnvironment(String environment) {
        this.environment = environment;
    }

    public String getIdempotencyKey() {
        return this.idempotencyKey;
    }

    public void setIdempotencyKey(String idempotencyKey) {
        this.idempotencyKey = idempotencyKey;
    }
}
