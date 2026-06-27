export interface NginxValidateResponse {
  valid?: boolean;
  errors?: Record<string, unknown>[];
}
