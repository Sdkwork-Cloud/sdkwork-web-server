import type { DomainVerifyResponse } from './domain-verify-response';

export interface SitesDomainsVerifyResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
