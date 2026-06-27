from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateCertificateRequest:
    domain_id: str
    cert_type: int
    auto_renew: Optional[bool] = None
