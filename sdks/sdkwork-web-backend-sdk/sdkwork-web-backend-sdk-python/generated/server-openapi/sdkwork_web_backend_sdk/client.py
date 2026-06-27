from .http_client import HttpClient, SdkConfig
from .api.nginx import NginxApi
from .api.server import ServerApi
from .api.agent import AgentApi
from .api.audit import AuditApi


class SdkworkBackendClient:
    """sdkwork-web-backend-sdk SDK Client."""

    def __init__(self, config: SdkConfig):
        self._client = HttpClient(config)
        self.nginx: NginxApi
        self.server: ServerApi
        self.agent: AgentApi
        self.audit: AuditApi

        # Initialize API modules
        self.nginx = NginxApi(self._client)
        self.server = ServerApi(self._client)
        self.agent = AgentApi(self._client)
        self.audit = AuditApi(self._client)
    def set_auth_token(self, token: str) -> 'SdkworkBackendClient':
        """Set auth token for authentication."""
        self._client.set_auth_token(token)
        return self

    def set_access_token(self, token: str) -> 'SdkworkBackendClient':
        """Set access token for authentication."""
        self._client.set_access_token(token)
        return self

    def set_header(self, key: str, value: str) -> 'SdkworkBackendClient':
        """Set custom header."""
        self._client.set_header(key, value)
        return self

    @property
    def http(self) -> HttpClient:
        """Get the underlying HTTP client."""
        return self._client


def create_client(config: SdkConfig) -> SdkworkBackendClient:
    """Create a new SDK client instance."""
    return SdkworkBackendClient(config)
