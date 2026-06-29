import type { DeploymentResponse } from './deployment-response';

export interface SitesDeploymentsRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
