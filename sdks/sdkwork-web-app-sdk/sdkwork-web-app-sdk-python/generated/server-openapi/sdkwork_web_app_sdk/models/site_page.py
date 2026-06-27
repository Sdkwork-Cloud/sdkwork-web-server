from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .site_response import SiteResponse


@dataclass
class SitePage:
    items: Optional[List[SiteResponse]] = None
    total: Optional[str] = None
    page: Optional[int] = None
    page_size: Optional[int] = None
