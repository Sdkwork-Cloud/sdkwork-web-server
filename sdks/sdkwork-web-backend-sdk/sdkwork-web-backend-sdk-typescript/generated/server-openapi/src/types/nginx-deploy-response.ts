export interface NginxDeployResponse {
  success?: boolean;
  configId?: string;
  deployedAt?: string;
  reloadResult?: Record<string, unknown>;
}
