import type { RuntimeObservation } from './runtime-observation';

export interface RuntimeAssignmentsObservationsLatestRetrieveResponse {
  code: 0;
  message: string;
  data: unknown & { item: RuntimeObservation; };
  traceId: string;
}
