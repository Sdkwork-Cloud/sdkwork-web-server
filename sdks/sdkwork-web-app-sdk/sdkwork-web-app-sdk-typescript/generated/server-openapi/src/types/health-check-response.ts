export interface HealthCheckResponse {
  id?: string;
  checkType?: number;
  checkUrl?: string;
  checkInterval?: number;
  status?: number;
  createdAt?: string;
}
