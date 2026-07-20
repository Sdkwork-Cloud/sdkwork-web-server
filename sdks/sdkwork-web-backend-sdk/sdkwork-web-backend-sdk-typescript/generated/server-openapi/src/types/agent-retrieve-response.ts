import type { AgentSyncResponse } from './agent-sync-response';

export interface AgentRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
