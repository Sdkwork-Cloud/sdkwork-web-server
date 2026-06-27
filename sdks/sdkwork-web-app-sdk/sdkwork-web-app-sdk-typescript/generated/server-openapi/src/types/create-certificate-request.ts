export interface CreateCertificateRequest {
  domainId: string;
  certType: 1 | 2 | 3;
  autoRenew?: boolean;
}
