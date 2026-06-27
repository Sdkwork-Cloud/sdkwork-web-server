import type { SiteResponse } from './site-response';

export interface SitePage {
  items?: SiteResponse[];
  /** Total item count as a string to avoid JavaScript precision loss. */
  total?: string;
  page?: number;
  pageSize?: number;
}
