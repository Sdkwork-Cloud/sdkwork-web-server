import Foundation
import SDKworkCommon

public class SdkworkBackendClient {
    private let httpClient: HttpClient
    public let nginx: NginxApi
    public let server: ServerApi
    public let agent: AgentApi
    public let audit: AuditApi

    public init(baseURL: String) {
        self.httpClient = HttpClient(baseURL: baseURL)
        self.nginx = NginxApi(client: httpClient)
        self.server = ServerApi(client: httpClient)
        self.agent = AgentApi(client: httpClient)
        self.audit = AuditApi(client: httpClient)
    }

    public init(config: SdkConfig) {
        self.httpClient = HttpClient(config: config)
        self.nginx = NginxApi(client: httpClient)
        self.server = ServerApi(client: httpClient)
        self.agent = AgentApi(client: httpClient)
        self.audit = AuditApi(client: httpClient)
    }
    public func setAuthToken(_ token: String) -> SdkworkBackendClient {
        httpClient.setAuthToken(token)
        return self
    }

    public func setAccessToken(_ token: String) -> SdkworkBackendClient {
        httpClient.setAccessToken(token)
        return self
    }

    public func setHeader(_ key: String, value: String) -> SdkworkBackendClient {
        httpClient.setHeader(key, value: value)
        return self
    }
}
