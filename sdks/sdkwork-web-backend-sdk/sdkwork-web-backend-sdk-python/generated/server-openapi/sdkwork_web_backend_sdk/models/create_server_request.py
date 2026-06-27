from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateServerRequest:
    name: str
    host: str
    ssh_port: int
    ssh_user: Optional[str] = None
    ssh_key_path: Optional[str] = None
    description: Optional[str] = None
