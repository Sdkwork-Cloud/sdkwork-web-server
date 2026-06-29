import type { DeploymentResponse } from './deployment-response';
import type { PageInfo } from './page-info';

export interface SitesDeploymentsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
