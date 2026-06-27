from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class AgentHeartbeatRequest:
    agent_version: Optional[str] = None
    nginx_enabled: Optional[bool] = None
    active_configs: Optional[str] = None
    last_sync_version: Optional[str] = None
