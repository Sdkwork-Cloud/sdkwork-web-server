from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateServerResponse:
    agent_token: str
    id: Optional[str] = None
    name: Optional[str] = None
    host: Optional[str] = None
    ssh_port: Optional[int] = None
    status: Optional[int] = None
    last_heartbeat_at: Optional[str] = None
    created_at: Optional[str] = None
