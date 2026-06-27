from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class AgentNginxConfigBundle:
    config_id: Optional[str] = None
    domain: Optional[str] = None
    config_content: Optional[str] = None
    fingerprint: Optional[str] = None
    version: Optional[str] = None
