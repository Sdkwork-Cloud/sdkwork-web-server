import type { NginxDeployResponse } from './nginx-deploy-response';

export interface NginxConfigsDeployResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
