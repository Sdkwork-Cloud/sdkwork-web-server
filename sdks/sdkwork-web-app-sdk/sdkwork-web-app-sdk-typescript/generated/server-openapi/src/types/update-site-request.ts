export interface UpdateSiteRequest {
  name?: string;
  description?: string;
  runtimeConfig?: Record<string, unknown>;
}
