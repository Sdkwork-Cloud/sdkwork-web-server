export interface CreateDeploymentRequest {
  deployType: 1 | 2 | 3 | 4;
  versionTag?: string;
  commitHash?: string;
  sourceRef?: string;
  environment?: string;
  idempotencyKey?: string;
}
