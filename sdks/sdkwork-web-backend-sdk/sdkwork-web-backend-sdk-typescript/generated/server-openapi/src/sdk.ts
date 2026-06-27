import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkBackendConfig } from './types/common';
import type { AuthTokenManager } from '@sdkwork/sdk-common';

import { NginxApi, createNginxApi } from './api/nginx';
import { ServerApi, createServerApi } from './api/server';
import { AgentApi, createAgentApi } from './api/agent';
import { AuditApi, createAuditApi } from './api/audit';

export class SdkworkBackendClient {
  private httpClient: HttpClient;

  public readonly nginx: NginxApi;
  public readonly server: ServerApi;
  public readonly agent: AgentApi;
  public readonly audit: AuditApi;

  constructor(config: SdkworkBackendConfig) {
    this.httpClient = createHttpClient(config);
    this.nginx = createNginxApi(this.httpClient);

    this.server = createServerApi(this.httpClient);

    this.agent = createAgentApi(this.httpClient);

    this.audit = createAuditApi(this.httpClient);
  }
  setAuthToken(token: string): this {
    this.httpClient.setAuthToken(token);
    return this;
  }

  setAccessToken(token: string): this {
    this.httpClient.setAccessToken(token);
    return this;
  }

  setTokenManager(manager: AuthTokenManager): this {
    this.httpClient.setTokenManager(manager);
    return this;
  }

  get http(): HttpClient {
    return this.httpClient;
  }
}

export function createClient(config: SdkworkBackendConfig): SdkworkBackendClient {
  return new SdkworkBackendClient(config);
}

export default SdkworkBackendClient;
