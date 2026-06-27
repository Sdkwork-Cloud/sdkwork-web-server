import type { CertificateResponse } from './certificate-response';

export interface CertificatePage {
  items?: CertificateResponse[];
  /** Total item count as a string to avoid JavaScript precision loss. */
  total?: string;
}
