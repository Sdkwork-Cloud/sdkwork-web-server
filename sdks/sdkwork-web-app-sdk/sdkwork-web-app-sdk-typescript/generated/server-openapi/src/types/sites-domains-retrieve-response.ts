import type { DomainResponse } from './domain-response';

export interface SitesDomainsRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
