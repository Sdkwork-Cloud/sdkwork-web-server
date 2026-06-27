from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class AgentCertificateBundle:
    certificate_id: Optional[str] = None
    cert_name: Optional[str] = None
    fingerprint: Optional[str] = None
    fullchain_pem: Optional[str] = None
    privkey_pem: Optional[str] = None
