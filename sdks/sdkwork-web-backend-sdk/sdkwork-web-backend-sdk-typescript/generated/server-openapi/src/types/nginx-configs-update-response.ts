import type { NginxConfigResponse } from './nginx-config-response';

export interface NginxConfigsUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
