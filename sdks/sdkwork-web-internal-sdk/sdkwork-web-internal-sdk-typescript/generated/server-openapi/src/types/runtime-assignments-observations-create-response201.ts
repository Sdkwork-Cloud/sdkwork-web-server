import type { RuntimeObservation } from './runtime-observation';

export interface RuntimeAssignmentsObservationsCreateResponse201 {
  code: 0;
  message: string;
  data: unknown & { item: RuntimeObservation; };
  traceId: string;
}
