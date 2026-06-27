from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class HealthCheckResponse:
    id: Optional[str] = None
    check_type: Optional[int] = None
    check_url: Optional[str] = None
    check_interval: Optional[int] = None
    status: Optional[int] = None
    created_at: Optional[str] = None
