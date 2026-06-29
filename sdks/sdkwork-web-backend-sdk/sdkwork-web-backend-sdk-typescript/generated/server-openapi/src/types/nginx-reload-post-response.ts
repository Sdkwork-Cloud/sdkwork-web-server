import type { NginxReloadResponse } from './nginx-reload-response';

export interface NginxReloadPostResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
