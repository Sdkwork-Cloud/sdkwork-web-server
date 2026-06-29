import type { CertificateResponse } from './certificate-response';

export interface CertificatesCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
