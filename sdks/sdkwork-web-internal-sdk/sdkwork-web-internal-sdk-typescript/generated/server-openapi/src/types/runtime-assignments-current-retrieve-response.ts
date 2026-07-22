import type { RuntimeAssignmentDelivery } from './runtime-assignment-delivery';

export interface RuntimeAssignmentsCurrentRetrieveResponse {
  code: 0;
  message: string;
  data: unknown & Record<string, unknown>;
  traceId: string;
}
