<?php

declare(strict_types=1);

namespace SDKWork\Web\AppSdk\Api;

use SDKWork\Web\AppSdk\Models\CreateDeploymentRequest;
use SDKWork\Web\AppSdk\Models\DeploymentPage;
use SDKWork\Web\AppSdk\Models\DeploymentResponse;

final class DeploymentApi extends BaseApi
{
    /** 获取部署历史 */
    public function sitesDeploymentsList(string $siteId, ?int $page = null, ?int $pageSize = null, ?int $status = null): ?DeploymentPage
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}/deployments', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false))]);
        $query = $this->buildQueryString([
            new QueryParameterSpec('page', $page, 'form', true, false, null),
            new QueryParameterSpec('pageSize', $pageSize, 'form', true, false, null),
            new QueryParameterSpec('status', $status, 'form', true, false, null),
        ]);
        $path = $this->appendQueryString($path, $query);
        $result = $this->client->request('GET', $path, []);
        return is_array($result) ? DeploymentPage::fromArray($result) : null;
    }

    /** 发起部署 */
    public function sitesDeploymentsCreate(string $siteId, array|CreateDeploymentRequest $body): ?DeploymentResponse
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}/deployments', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false))]);
        $payload = $body instanceof CreateDeploymentRequest ? $body->toArray() : $body;
        $result = $this->client->request('POST', $path, [
            'json' => $payload,
        ]);
        return is_array($result) ? DeploymentResponse::fromArray($result) : null;
    }

    /** 获取部署详情 */
    public function sitesDeploymentsRetrieve(string $siteId, string $deploymentId): ?DeploymentResponse
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}/deployments/{deploymentId}', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false)), 'deploymentId' => $this->serializePathParameter($deploymentId, new PathParameterSpec('deploymentId', 'simple', false))]);
        $result = $this->client->request('GET', $path, []);
        return is_array($result) ? DeploymentResponse::fromArray($result) : null;
    }

    /** 回滚部署 */
    public function sitesDeploymentsRollback(string $siteId, string $deploymentId): ?DeploymentResponse
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}/deployments/{deploymentId}/rollback', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false)), 'deploymentId' => $this->serializePathParameter($deploymentId, new PathParameterSpec('deploymentId', 'simple', false))]);
        $result = $this->client->request('POST', $path, []);
        return is_array($result) ? DeploymentResponse::fromArray($result) : null;
    }

}
