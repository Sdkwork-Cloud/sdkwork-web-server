from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CertificateResponse:
    id: Optional[str] = None
    cert_name: Optional[str] = None
    cert_type: Optional[int] = None
    issuer: Optional[str] = None
    not_before: Optional[str] = None
    not_after: Optional[str] = None
    auto_renew: Optional[bool] = None
    status: Optional[int] = None
    created_at: Optional[str] = None
