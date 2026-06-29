import type { NginxValidateResponse } from './nginx-validate-response';

export interface NginxConfigsValidateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
