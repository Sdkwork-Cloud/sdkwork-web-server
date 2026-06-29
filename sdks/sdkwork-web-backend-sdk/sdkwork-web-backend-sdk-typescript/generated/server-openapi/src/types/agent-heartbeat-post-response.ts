import type { AgentHeartbeatResponse } from './agent-heartbeat-response';

export interface AgentHeartbeatPostResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
