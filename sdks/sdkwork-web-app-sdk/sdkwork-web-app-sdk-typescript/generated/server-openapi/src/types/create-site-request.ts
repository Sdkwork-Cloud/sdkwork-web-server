export interface CreateSiteRequest {
  name: string;
  slug?: string;
  description?: string;
  siteType: 1 | 2 | 3 | 4 | 5 | 6;
  runtimeConfig?: Record<string, unknown>;
}
