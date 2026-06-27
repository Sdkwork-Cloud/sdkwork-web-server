export interface SiteResponse {
  id?: string;
  name?: string;
  slug?: string;
  description?: string;
  siteType?: number;
  status?: number;
  runtimeConfig?: Record<string, unknown>;
  createdAt?: string;
  updatedAt?: string;
}
