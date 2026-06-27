package com.sdkwork.web.backend.sdk.model;


public class CreateServerRequest {
    private String name;
    private String host;
    private Integer sshPort;
    private String sshUser;
    private String sshKeyPath;
    private String description;

    public String getName() {
        return this.name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getHost() {
        return this.host;
    }

    public void setHost(String host) {
        this.host = host;
    }

    public Integer getSshPort() {
        return this.sshPort;
    }

    public void setSshPort(Integer sshPort) {
        this.sshPort = sshPort;
    }

    public String getSshUser() {
        return this.sshUser;
    }

    public void setSshUser(String sshUser) {
        this.sshUser = sshUser;
    }

    public String getSshKeyPath() {
        return this.sshKeyPath;
    }

    public void setSshKeyPath(String sshKeyPath) {
        this.sshKeyPath = sshKeyPath;
    }

    public String getDescription() {
        return this.description;
    }

    public void setDescription(String description) {
        this.description = description;
    }
}
