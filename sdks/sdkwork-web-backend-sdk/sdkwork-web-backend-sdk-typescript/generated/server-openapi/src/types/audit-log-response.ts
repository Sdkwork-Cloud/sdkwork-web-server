export interface AuditLogResponse {
  id?: string;
  /** Operator user id as a string to avoid JavaScript precision loss. */
  operatorId?: string;
  operatorType?: string;
  action?: string;
  targetType?: string;
  /** Target snowflake id as a string to avoid JavaScript precision loss. */
  targetId?: string;
  targetUuid?: string;
  requestId?: string;
  ipAddress?: string;
  changes?: Record<string, unknown>;
  createdAt?: string;
}
