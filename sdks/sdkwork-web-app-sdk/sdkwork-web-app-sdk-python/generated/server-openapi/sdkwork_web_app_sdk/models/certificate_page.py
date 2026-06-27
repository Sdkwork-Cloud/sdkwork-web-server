from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .certificate_response import CertificateResponse


@dataclass
class CertificatePage:
    items: Optional[List[CertificateResponse]] = None
    total: Optional[str] = None
