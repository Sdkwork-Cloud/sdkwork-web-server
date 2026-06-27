import Foundation

/// API modules for sdkwork-web-backend-sdk
public struct API {
    public static let nginx = NginxApi.self
    public static let server = ServerApi.self
    public static let agent = AgentApi.self
    public static let audit = AuditApi.self
}
