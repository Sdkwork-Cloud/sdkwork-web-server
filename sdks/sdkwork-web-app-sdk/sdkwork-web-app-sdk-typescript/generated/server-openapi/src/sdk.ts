import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkAppConfig } from './types/common';
import type { AuthTokenManager } from '@sdkwork/sdk-common';

import { SiteApi, createSiteApi } from './api/site';
import { DomainApi, createDomainApi } from './api/domain';
import { DeploymentApi, createDeploymentApi } from './api/deployment';
import { EnvVariableApi, createEnvVariableApi } from './api/env-variable';
import { CertificateApi, createCertificateApi } from './api/certificate';
import { MonitorApi, createMonitorApi } from './api/monitor';

export class SdkworkAppClient {
  private httpClient: HttpClient;

  public readonly site: SiteApi;
  public readonly domain: DomainApi;
  public readonly deployment: DeploymentApi;
  public readonly envVariable: EnvVariableApi;
  public readonly certificate: CertificateApi;
  public readonly monitor: MonitorApi;

  constructor(config: SdkworkAppConfig) {
    this.httpClient = createHttpClient(config);
    this.site = createSiteApi(this.httpClient);

    this.domain = createDomainApi(this.httpClient);

    this.deployment = createDeploymentApi(this.httpClient);

    this.envVariable = createEnvVariableApi(this.httpClient);

    this.certificate = createCertificateApi(this.httpClient);

    this.monitor = createMonitorApi(this.httpClient);
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

export function createClient(config: SdkworkAppConfig): SdkworkAppClient {
  return new SdkworkAppClient(config);
}

export default SdkworkAppClient;
