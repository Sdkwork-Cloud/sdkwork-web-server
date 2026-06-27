import 'package:sdkwork_common_flutter/sdkwork_common_flutter.dart';
import 'src/http/client.dart';
import 'src/api/nginx.dart';
import 'src/api/server.dart';
import 'src/api/agent.dart';
import 'src/api/audit.dart';

class SdkworkBackendClient {
  final HttpClient _httpClient;

  late final NginxApi nginx;
  late final ServerApi server;
  late final AgentApi agent;
  late final AuditApi audit;

  SdkworkBackendClient({
    required SdkConfig config,
  }) : _httpClient = HttpClient(config: config) {
    nginx = NginxApi(_httpClient);
    server = ServerApi(_httpClient);
    agent = AgentApi(_httpClient);
    audit = AuditApi(_httpClient);
  }

  factory SdkworkBackendClient.withBaseUrl({
    required String baseUrl,
    String? authToken,
    String? accessToken,
    Map<String, String>? headers,
    int timeout = 30000,
  }) {
    return SdkworkBackendClient(
      config: SdkConfig(
        baseUrl: baseUrl,
        timeout: timeout,
        headers: headers ?? const {},
        authToken: authToken,
        accessToken: accessToken,
      ),
    );
  }

  void setAuthToken(String token) {
    _httpClient.setAuthToken(token);
  }

  void setAccessToken(String token) {
    _httpClient.setAccessToken(token);
  }

  void setHeader(String key, String value) {
    _httpClient.setHeader(key, value);
  }
}
