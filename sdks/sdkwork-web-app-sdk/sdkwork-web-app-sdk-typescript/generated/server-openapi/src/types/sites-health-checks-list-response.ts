import type { HealthCheckResponse } from './health-check-response';
import type { PageInfo } from './page-info';

export interface SitesHealthChecksListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
