from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class AgentHeartbeatResponse:
    server_id: Optional[str] = None
    status: Optional[int] = None
    acknowledged_at: Optional[str] = None
