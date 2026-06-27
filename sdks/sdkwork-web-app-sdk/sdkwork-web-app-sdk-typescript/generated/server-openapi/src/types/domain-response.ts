export interface DomainResponse {
  id?: string;
  hostname?: string;
  isPrimary?: boolean;
  isVerified?: boolean;
  sslEnabled?: boolean;
  sslProvider?: string;
  status?: number;
  createdAt?: string;
}
