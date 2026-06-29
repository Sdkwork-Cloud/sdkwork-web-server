import type { SiteResponse } from './site-response';

export interface SitesActivateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
