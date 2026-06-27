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

public struct CreateSiteRequest: Codable {
    public let name: String?
    public let slug: String?
    public let description: String?
    public let siteType: Int?
    public let runtimeConfig: [String: Any]?


    public init(name: String? = nil, slug: String? = nil, description: String? = nil, siteType: Int? = nil, runtimeConfig: [String: Any]? = nil) {
        self.name = name
        self.slug = slug
        self.description = description
        self.siteType = siteType
        self.runtimeConfig = runtimeConfig
    }
}

public struct UpdateSiteRequest: Codable {
    public let name: String?
    public let description: String?
    public let runtimeConfig: [String: Any]?


    public init(name: String? = nil, description: String? = nil, runtimeConfig: [String: Any]? = nil) {
        self.name = name
        self.description = description
        self.runtimeConfig = runtimeConfig
    }
}

public struct SiteResponse: Codable {
    public let id: String?
    public let name: String?
    public let slug: String?
    public let description: String?
    public let siteType: Int?
    public let status: Int?
    public let runtimeConfig: [String: Any]?
    public let createdAt: String?
    public let updatedAt: String?


    public init(id: String? = nil, name: String? = nil, slug: String? = nil, description: String? = nil, siteType: Int? = nil, status: Int? = nil, runtimeConfig: [String: Any]? = nil, createdAt: String? = nil, updatedAt: String? = nil) {
        self.id = id
        self.name = name
        self.slug = slug
        self.description = description
        self.siteType = siteType
        self.status = status
        self.runtimeConfig = runtimeConfig
        self.createdAt = createdAt
        self.updatedAt = updatedAt
    }
}

public struct SitePage: Codable {
    public let items: [SiteResponse]?
    public let total: String?
    public let page: Int?
    public let pageSize: Int?


    public init(items: [SiteResponse]? = nil, total: String? = nil, page: Int? = nil, pageSize: Int? = nil) {
        self.items = items
        self.total = total
        self.page = page
        self.pageSize = pageSize
    }
}

public struct CreateDomainRequest: Codable {
    public let hostname: String?
    public let isPrimary: Bool?
    public let sslEnabled: Bool?
    public let sslProvider: String?


    public init(hostname: String? = nil, isPrimary: Bool? = nil, sslEnabled: Bool? = nil, sslProvider: String? = nil) {
        self.hostname = hostname
        self.isPrimary = isPrimary
        self.sslEnabled = sslEnabled
        self.sslProvider = sslProvider
    }
}

public struct DomainResponse: Codable {
    public let id: String?
    public let hostname: String?
    public let isPrimary: Bool?
    public let isVerified: Bool?
    public let sslEnabled: Bool?
    public let sslProvider: String?
    public let status: Int?
    public let createdAt: String?


    public init(id: String? = nil, hostname: String? = nil, isPrimary: Bool? = nil, isVerified: Bool? = nil, sslEnabled: Bool? = nil, sslProvider: String? = nil, status: Int? = nil, createdAt: String? = nil) {
        self.id = id
        self.hostname = hostname
        self.isPrimary = isPrimary
        self.isVerified = isVerified
        self.sslEnabled = sslEnabled
        self.sslProvider = sslProvider
        self.status = status
        self.createdAt = createdAt
    }
}

public struct DomainPage: Codable {
    public let items: [DomainResponse]?
    public let total: String?


    public init(items: [DomainResponse]? = nil, total: String? = nil) {
        self.items = items
        self.total = total
    }
}

public struct DomainVerifyResponse: Codable {
    public let verified: Bool?
    public let method: String?
    public let token: String?


    public init(verified: Bool? = nil, method: String? = nil, token: String? = nil) {
        self.verified = verified
        self.method = method
        self.token = token
    }
}

public struct CreateDeploymentRequest: Codable {
    public let deployType: Int?
    public let versionTag: String?
    public let commitHash: String?
    public let sourceRef: String?
    public let environment: String?
    public let idempotencyKey: String?


    public init(deployType: Int? = nil, versionTag: String? = nil, commitHash: String? = nil, sourceRef: String? = nil, environment: String? = nil, idempotencyKey: String? = nil) {
        self.deployType = deployType
        self.versionTag = versionTag
        self.commitHash = commitHash
        self.sourceRef = sourceRef
        self.environment = environment
        self.idempotencyKey = idempotencyKey
    }
}

public struct DeploymentResponse: Codable {
    public let id: String?
    public let siteId: String?
    public let deployType: Int?
    public let versionTag: String?
    public let status: Int?
    public let startedAt: String?
    public let completedAt: String?
    public let durationMs: String?
    public let createdAt: String?


    public init(id: String? = nil, siteId: String? = nil, deployType: Int? = nil, versionTag: String? = nil, status: Int? = nil, startedAt: String? = nil, completedAt: String? = nil, durationMs: String? = nil, createdAt: String? = nil) {
        self.id = id
        self.siteId = siteId
        self.deployType = deployType
        self.versionTag = versionTag
        self.status = status
        self.startedAt = startedAt
        self.completedAt = completedAt
        self.durationMs = durationMs
        self.createdAt = createdAt
    }
}

public struct DeploymentPage: Codable {
    public let items: [DeploymentResponse]?
    public let total: String?


    public init(items: [DeploymentResponse]? = nil, total: String? = nil) {
        self.items = items
        self.total = total
    }
}

public struct CreateEnvVariableRequest: Codable {
    public let key: String?
    public let value: String?
    public let environment: String?
    public let isSecret: Bool?


    public init(key: String? = nil, value: String? = nil, environment: String? = nil, isSecret: Bool? = nil) {
        self.key = key
        self.value = value
        self.environment = environment
        self.isSecret = isSecret
    }
}

public struct EnvVariableResponse: Codable {
    public let id: String?
    public let key: String?
    public let environment: String?
    public let isSecret: Bool?
    public let createdAt: String?


    public init(id: String? = nil, key: String? = nil, environment: String? = nil, isSecret: Bool? = nil, createdAt: String? = nil) {
        self.id = id
        self.key = key
        self.environment = environment
        self.isSecret = isSecret
        self.createdAt = createdAt
    }
}

public struct EnvVariablePage: Codable {
    public let items: [EnvVariableResponse]?
    public let total: String?


    public init(items: [EnvVariableResponse]? = nil, total: String? = nil) {
        self.items = items
        self.total = total
    }
}

public struct CreateCertificateRequest: Codable {
    public let domainId: String?
    public let certType: Int?
    public let autoRenew: Bool?


    public init(domainId: String? = nil, certType: Int? = nil, autoRenew: Bool? = nil) {
        self.domainId = domainId
        self.certType = certType
        self.autoRenew = autoRenew
    }
}

public struct CertificateResponse: Codable {
    public let id: String?
    public let certName: String?
    public let certType: Int?
    public let issuer: String?
    public let notBefore: String?
    public let notAfter: String?
    public let autoRenew: Bool?
    public let status: Int?
    public let createdAt: String?


    public init(id: String? = nil, certName: String? = nil, certType: Int? = nil, issuer: String? = nil, notBefore: String? = nil, notAfter: String? = nil, autoRenew: Bool? = nil, status: Int? = nil, createdAt: String? = nil) {
        self.id = id
        self.certName = certName
        self.certType = certType
        self.issuer = issuer
        self.notBefore = notBefore
        self.notAfter = notAfter
        self.autoRenew = autoRenew
        self.status = status
        self.createdAt = createdAt
    }
}

public struct CertificatePage: Codable {
    public let items: [CertificateResponse]?
    public let total: String?


    public init(items: [CertificateResponse]? = nil, total: String? = nil) {
        self.items = items
        self.total = total
    }
}

public struct CreateHealthCheckRequest: Codable {
    public let checkType: Int?
    public let checkUrl: String?
    public let checkInterval: Int?
    public let timeoutMs: Int?
    public let retryCount: Int?


    public init(checkType: Int? = nil, checkUrl: String? = nil, checkInterval: Int? = nil, timeoutMs: Int? = nil, retryCount: Int? = nil) {
        self.checkType = checkType
        self.checkUrl = checkUrl
        self.checkInterval = checkInterval
        self.timeoutMs = timeoutMs
        self.retryCount = retryCount
    }
}

public struct HealthCheckResponse: Codable {
    public let id: String?
    public let checkType: Int?
    public let checkUrl: String?
    public let checkInterval: Int?
    public let status: Int?
    public let createdAt: String?


    public init(id: String? = nil, checkType: Int? = nil, checkUrl: String? = nil, checkInterval: Int? = nil, status: Int? = nil, createdAt: String? = nil) {
        self.id = id
        self.checkType = checkType
        self.checkUrl = checkUrl
        self.checkInterval = checkInterval
        self.status = status
        self.createdAt = createdAt
    }
}

public struct HealthCheckPage: Codable {
    public let items: [HealthCheckResponse]?
    public let total: String?


    public init(items: [HealthCheckResponse]? = nil, total: String? = nil) {
        self.items = items
        self.total = total
    }
}
