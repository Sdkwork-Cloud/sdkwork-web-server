import type { CreateServerResponse } from './create-server-response';

export interface ServersCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
