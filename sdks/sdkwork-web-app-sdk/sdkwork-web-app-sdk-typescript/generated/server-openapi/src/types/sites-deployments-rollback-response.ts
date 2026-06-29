import type { DeploymentResponse } from './deployment-response';

export interface SitesDeploymentsRollbackResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
