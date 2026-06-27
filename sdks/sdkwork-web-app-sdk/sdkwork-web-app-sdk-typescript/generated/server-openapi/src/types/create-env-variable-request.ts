export interface CreateEnvVariableRequest {
  key: string;
  value: string;
  environment?: string;
  isSecret?: boolean;
}
