export interface CertificateResponse {
  id?: string;
  certName?: string;
  certType?: number;
  issuer?: string;
  notBefore?: string;
  notAfter?: string;
  autoRenew?: boolean;
  status?: number;
  createdAt?: string;
}
