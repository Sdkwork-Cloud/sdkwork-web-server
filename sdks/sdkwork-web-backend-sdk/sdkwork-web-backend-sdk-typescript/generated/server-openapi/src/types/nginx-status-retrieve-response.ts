import type { NginxStatusResponse } from './nginx-status-response';

export interface NginxStatusRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
