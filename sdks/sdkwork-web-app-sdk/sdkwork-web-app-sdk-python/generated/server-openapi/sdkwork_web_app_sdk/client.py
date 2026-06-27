from .http_client import HttpClient, SdkConfig
from .api.site import SiteApi
from .api.domain import DomainApi
from .api.deployment import DeploymentApi
from .api.env_variable import EnvVariableApi
from .api.certificate import CertificateApi
from .api.monitor import MonitorApi


class SdkworkAppClient:
    """sdkwork-web-app-sdk SDK Client."""

    def __init__(self, config: SdkConfig):
        self._client = HttpClient(config)
        self.site: SiteApi
        self.domain: DomainApi
        self.deployment: DeploymentApi
        self.env_variable: EnvVariableApi
        self.certificate: CertificateApi
        self.monitor: MonitorApi

        # Initialize API modules
        self.site = SiteApi(self._client)
        self.domain = DomainApi(self._client)
        self.deployment = DeploymentApi(self._client)
        self.env_variable = EnvVariableApi(self._client)
        self.certificate = CertificateApi(self._client)
        self.monitor = MonitorApi(self._client)
    def set_auth_token(self, token: str) -> 'SdkworkAppClient':
        """Set auth token for authentication."""
        self._client.set_auth_token(token)
        return self

    def set_access_token(self, token: str) -> 'SdkworkAppClient':
        """Set access token for authentication."""
        self._client.set_access_token(token)
        return self

    def set_header(self, key: str, value: str) -> 'SdkworkAppClient':
        """Set custom header."""
        self._client.set_header(key, value)
        return self

    @property
    def http(self) -> HttpClient:
        """Get the underlying HTTP client."""
        return self._client


def create_client(config: SdkConfig) -> SdkworkAppClient:
    """Create a new SDK client instance."""
    return SdkworkAppClient(config)
