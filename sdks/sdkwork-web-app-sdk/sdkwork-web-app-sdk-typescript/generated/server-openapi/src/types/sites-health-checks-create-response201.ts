import type { HealthCheckResponse } from './health-check-response';

export interface SitesHealthChecksCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
