from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DomainResponse:
    id: Optional[str] = None
    hostname: Optional[str] = None
    is_primary: Optional[bool] = None
    is_verified: Optional[bool] = None
    ssl_enabled: Optional[bool] = None
    ssl_provider: Optional[str] = None
    status: Optional[int] = None
    created_at: Optional[str] = None
