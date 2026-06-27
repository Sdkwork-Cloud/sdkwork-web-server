from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DeploymentResponse:
    id: Optional[str] = None
    site_id: Optional[str] = None
    deploy_type: Optional[int] = None
    version_tag: Optional[str] = None
    status: Optional[int] = None
    started_at: Optional[str] = None
    completed_at: Optional[str] = None
    duration_ms: Optional[str] = None
    created_at: Optional[str] = None
