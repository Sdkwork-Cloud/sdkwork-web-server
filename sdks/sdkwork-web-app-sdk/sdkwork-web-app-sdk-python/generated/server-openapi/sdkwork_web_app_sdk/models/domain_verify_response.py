from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DomainVerifyResponse:
    verified: Optional[bool] = None
    method: Optional[str] = None
    token: Optional[str] = None
