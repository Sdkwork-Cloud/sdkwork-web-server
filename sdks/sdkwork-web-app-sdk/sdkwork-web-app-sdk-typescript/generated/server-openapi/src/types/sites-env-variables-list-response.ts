import type { EnvVariableResponse } from './env-variable-response';
import type { PageInfo } from './page-info';

export interface SitesEnvVariablesListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
