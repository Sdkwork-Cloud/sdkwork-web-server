import type { RuntimeAssignment } from './runtime-assignment';

export interface RuntimeAssignmentsUpdateResponse {
  code: 0;
  message: string;
  data: unknown & Record<string, unknown>;
  traceId: string;
}
