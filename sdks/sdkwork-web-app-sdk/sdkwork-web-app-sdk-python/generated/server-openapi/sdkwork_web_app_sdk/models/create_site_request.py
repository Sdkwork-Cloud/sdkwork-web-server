from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateSiteRequest:
    name: str
    site_type: int
    slug: Optional[str] = None
    description: Optional[str] = None
    runtime_config: Optional[Dict[str, Any]] = None
