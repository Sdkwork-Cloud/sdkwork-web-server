import type { AuditLogResponse } from './audit-log-response';
import type { PageInfo } from './page-info';

export interface AuditLogsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
