from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .agent_certificate_bundle import AgentCertificateBundle
    from .agent_nginx_config_bundle import AgentNginxConfigBundle


@dataclass
class AgentSyncResponse:
    server_id: Optional[str] = None
    sync_version: Optional[str] = None
    unchanged: Optional[bool] = None
    nginx_configs: Optional[List[AgentNginxConfigBundle]] = None
    certificates: Optional[List[AgentCertificateBundle]] = None
