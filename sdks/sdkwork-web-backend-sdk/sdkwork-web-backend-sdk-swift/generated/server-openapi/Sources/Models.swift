import Foundation

public struct ProblemDetail: Codable {
    public let type: String?
    public let title: String?
    public let status: Int?
    public let detail: String?
    public let instance: String?
    public let requestId: String?


    public init(type: String? = nil, title: String? = nil, status: Int? = nil, detail: String? = nil, instance: String? = nil, requestId: String? = nil) {
        self.type = type
        self.title = title
        self.status = status
        self.detail = detail
        self.instance = instance
        self.requestId = requestId
    }
}

public struct CreateNginxConfigRequest: Codable {
    public let configType: Int?
    public let configName: String?
    public let configContent: String?
    public let siteId: String?
    public let domainId: String?


    public init(configType: Int? = nil, configName: String? = nil, configContent: String? = nil, siteId: String? = nil, domainId: String? = nil) {
        self.configType = configType
        self.configName = configName
        self.configContent = configContent
        self.siteId = siteId
        self.domainId = domainId
    }
}

public struct UpdateNginxConfigRequest: Codable {
    public let configContent: String?
    public let configName: String?


    public init(configContent: String? = nil, configName: String? = nil) {
        self.configContent = configContent
        self.configName = configName
    }
}

public struct NginxConfigResponse: Codable {
    public let id: String?
    public let configType: Int?
    public let configName: String?
    public let configContent: String?
    public let configHash: String?
    public let isActive: Bool?
    public let versionNo: Int?
    public let deployedAt: String?
    public let status: Int?
    public let createdAt: String?
    public let updatedAt: String?


    public init(id: String? = nil, configType: Int? = nil, configName: String? = nil, configContent: String? = nil, configHash: String? = nil, isActive: Bool? = nil, versionNo: Int? = nil, deployedAt: String? = nil, status: Int? = nil, createdAt: String? = nil, updatedAt: String? = nil) {
        self.id = id
        self.configType = configType
        self.configName = configName
        self.configContent = configContent
        self.configHash = configHash
        self.isActive = isActive
        self.versionNo = versionNo
        self.deployedAt = deployedAt
        self.status = status
        self.createdAt = createdAt
        self.updatedAt = updatedAt
    }
}

public struct NginxConfigPage: Codable {
    public let items: [NginxConfigResponse]?
    public let total: String?


    public init(items: [NginxConfigResponse]? = nil, total: String? = nil) {
        self.items = items
        self.total = total
    }
}

public struct NginxValidateResponse: Codable {
    public let valid: Bool?
    public let errors: [[String: Any]]?


    public init(valid: Bool? = nil, errors: [[String: Any]]? = nil) {
        self.valid = valid
        self.errors = errors
    }
}

public struct NginxDeployResponse: Codable {
    public let success: Bool?
    public let configId: String?
    public let deployedAt: String?
    public let reloadResult: [String: Any]?


    public init(success: Bool? = nil, configId: String? = nil, deployedAt: String? = nil, reloadResult: [String: Any]? = nil) {
        self.success = success
        self.configId = configId
        self.deployedAt = deployedAt
        self.reloadResult = reloadResult
    }
}

public struct NginxReloadResponse: Codable {
    public let success: Bool?
    public let message: String?
    public let timestamp: String?


    public init(success: Bool? = nil, message: String? = nil, timestamp: String? = nil) {
        self.success = success
        self.message = message
        self.timestamp = timestamp
    }
}

public struct NginxStatusResponse: Codable {
    public let running: Bool?
    public let version: String?
    public let pid: Int?
    public let activeConnections: Int?
    public let configPath: String?
    public let uptime: String?


    public init(running: Bool? = nil, version: String? = nil, pid: Int? = nil, activeConnections: Int? = nil, configPath: String? = nil, uptime: String? = nil) {
        self.running = running
        self.version = version
        self.pid = pid
        self.activeConnections = activeConnections
        self.configPath = configPath
        self.uptime = uptime
    }
}

public struct CreateServerRequest: Codable {
    public let name: String?
    public let host: String?
    public let sshPort: Int?
    public let sshUser: String?
    public let sshKeyPath: String?
    public let description: String?


    public init(name: String? = nil, host: String? = nil, sshPort: Int? = nil, sshUser: String? = nil, sshKeyPath: String? = nil, description: String? = nil) {
        self.name = name
        self.host = host
        self.sshPort = sshPort
        self.sshUser = sshUser
        self.sshKeyPath = sshKeyPath
        self.description = description
    }
}

public struct ServerResponse: Codable {
    public let id: String?
    public let name: String?
    public let host: String?
    public let sshPort: Int?
    public let status: Int?
    public let lastHeartbeatAt: String?
    public let createdAt: String?


    public init(id: String? = nil, name: String? = nil, host: String? = nil, sshPort: Int? = nil, status: Int? = nil, lastHeartbeatAt: String? = nil, createdAt: String? = nil) {
        self.id = id
        self.name = name
        self.host = host
        self.sshPort = sshPort
        self.status = status
        self.lastHeartbeatAt = lastHeartbeatAt
        self.createdAt = createdAt
    }
}

public struct CreateServerResponse: Codable {
    public let id: String?
    public let name: String?
    public let host: String?
    public let sshPort: Int?
    public let status: Int?
    public let lastHeartbeatAt: String?
    public let createdAt: String?
    public let agentToken: String?


    public init(id: String? = nil, name: String? = nil, host: String? = nil, sshPort: Int? = nil, status: Int? = nil, lastHeartbeatAt: String? = nil, createdAt: String? = nil, agentToken: String? = nil) {
        self.id = id
        self.name = name
        self.host = host
        self.sshPort = sshPort
        self.status = status
        self.lastHeartbeatAt = lastHeartbeatAt
        self.createdAt = createdAt
        self.agentToken = agentToken
    }
}

public struct AgentHeartbeatRequest: Codable {
    public let agentVersion: String?
    public let nginxEnabled: Bool?
    public let activeConfigs: String?
    public let lastSyncVersion: String?


    public init(agentVersion: String? = nil, nginxEnabled: Bool? = nil, activeConfigs: String? = nil, lastSyncVersion: String? = nil) {
        self.agentVersion = agentVersion
        self.nginxEnabled = nginxEnabled
        self.activeConfigs = activeConfigs
        self.lastSyncVersion = lastSyncVersion
    }
}

public struct AgentHeartbeatResponse: Codable {
    public let serverId: String?
    public let status: Int?
    public let acknowledgedAt: String?


    public init(serverId: String? = nil, status: Int? = nil, acknowledgedAt: String? = nil) {
        self.serverId = serverId
        self.status = status
        self.acknowledgedAt = acknowledgedAt
    }
}

public struct AgentSyncResponse: Codable {
    public let serverId: String?
    public let syncVersion: String?
    public let unchanged: Bool?
    public let nginxConfigs: [AgentNginxConfigBundle]?
    public let certificates: [AgentCertificateBundle]?


    public init(serverId: String? = nil, syncVersion: String? = nil, unchanged: Bool? = nil, nginxConfigs: [AgentNginxConfigBundle]? = nil, certificates: [AgentCertificateBundle]? = nil) {
        self.serverId = serverId
        self.syncVersion = syncVersion
        self.unchanged = unchanged
        self.nginxConfigs = nginxConfigs
        self.certificates = certificates
    }
}

public struct AgentNginxConfigBundle: Codable {
    public let configId: String?
    public let domain: String?
    public let configContent: String?
    public let fingerprint: String?
    public let version: String?


    public init(configId: String? = nil, domain: String? = nil, configContent: String? = nil, fingerprint: String? = nil, version: String? = nil) {
        self.configId = configId
        self.domain = domain
        self.configContent = configContent
        self.fingerprint = fingerprint
        self.version = version
    }
}

public struct AgentCertificateBundle: Codable {
    public let certificateId: String?
    public let certName: String?
    public let fingerprint: String?
    public let fullchainPem: String?
    public let privkeyPem: String?


    public init(certificateId: String? = nil, certName: String? = nil, fingerprint: String? = nil, fullchainPem: String? = nil, privkeyPem: String? = nil) {
        self.certificateId = certificateId
        self.certName = certName
        self.fingerprint = fingerprint
        self.fullchainPem = fullchainPem
        self.privkeyPem = privkeyPem
    }
}

public struct ServerPage: Codable {
    public let items: [ServerResponse]?
    public let total: String?


    public init(items: [ServerResponse]? = nil, total: String? = nil) {
        self.items = items
        self.total = total
    }
}

public struct AuditLogResponse: Codable {
    public let id: String?
    public let operatorId: String?
    public let operatorType: String?
    public let action: String?
    public let targetType: String?
    public let targetId: String?
    public let targetUuid: String?
    public let requestId: String?
    public let ipAddress: String?
    public let changes: [String: Any]?
    public let createdAt: String?


    public init(id: String? = nil, operatorId: String? = nil, operatorType: String? = nil, action: String? = nil, targetType: String? = nil, targetId: String? = nil, targetUuid: String? = nil, requestId: String? = nil, ipAddress: String? = nil, changes: [String: Any]? = nil, createdAt: String? = nil) {
        self.id = id
        self.operatorId = operatorId
        self.operatorType = operatorType
        self.action = action
        self.targetType = targetType
        self.targetId = targetId
        self.targetUuid = targetUuid
        self.requestId = requestId
        self.ipAddress = ipAddress
        self.changes = changes
        self.createdAt = createdAt
    }
}

public struct AuditLogPage: Codable {
    public let items: [AuditLogResponse]?
    public let total: String?


    public init(items: [AuditLogResponse]? = nil, total: String? = nil) {
        self.items = items
        self.total = total
    }
}
