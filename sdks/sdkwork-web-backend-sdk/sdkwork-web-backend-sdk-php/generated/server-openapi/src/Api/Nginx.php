<?php

declare(strict_types=1);

namespace SDKWork\Web\BackendSdk\Api;

use SDKWork\Web\BackendSdk\Models\CreateNginxConfigRequest;
use SDKWork\Web\BackendSdk\Models\NginxConfigPage;
use SDKWork\Web\BackendSdk\Models\NginxConfigResponse;
use SDKWork\Web\BackendSdk\Models\NginxDeployResponse;
use SDKWork\Web\BackendSdk\Models\NginxReloadResponse;
use SDKWork\Web\BackendSdk\Models\NginxStatusResponse;
use SDKWork\Web\BackendSdk\Models\NginxValidateResponse;
use SDKWork\Web\BackendSdk\Models\UpdateNginxConfigRequest;

final class NginxApi extends BaseApi
{
    /** 获取 Nginx 配置列表 */
    public function configsList(?int $page = null, ?int $pageSize = null, ?string $siteId = null, ?int $configType = null, ?bool $isActive = null): ?NginxConfigPage
    {
        $path = '/backend/v3/api/nginx/configs';
        $query = $this->buildQueryString([
            new QueryParameterSpec('page', $page, 'form', true, false, null),
            new QueryParameterSpec('pageSize', $pageSize, 'form', true, false, null),
            new QueryParameterSpec('siteId', $siteId, 'form', true, false, null),
            new QueryParameterSpec('configType', $configType, 'form', true, false, null),
            new QueryParameterSpec('isActive', $isActive, 'form', true, false, null),
        ]);
        $path = $this->appendQueryString($path, $query);
        $result = $this->client->request('GET', $path, []);
        return is_array($result) ? NginxConfigPage::fromArray($result) : null;
    }

    /** 创建 Nginx 配置 */
    public function configsCreate(array|CreateNginxConfigRequest $body): ?NginxConfigResponse
    {
        $path = '/backend/v3/api/nginx/configs';
        $payload = $body instanceof CreateNginxConfigRequest ? $body->toArray() : $body;
        $result = $this->client->request('POST', $path, [
            'json' => $payload,
        ]);
        return is_array($result) ? NginxConfigResponse::fromArray($result) : null;
    }

    /** 获取 Nginx 配置详情 */
    public function configsRetrieve(string $configId): ?NginxConfigResponse
    {
        $path = $this->interpolatePath('/backend/v3/api/nginx/etc/{configId}', ['configId' => $this->serializePathParameter($configId, new PathParameterSpec('configId', 'simple', false))]);
        $result = $this->client->request('GET', $path, []);
        return is_array($result) ? NginxConfigResponse::fromArray($result) : null;
    }

    /** 更新 Nginx 配置 */
    public function configsUpdate(string $configId, array|UpdateNginxConfigRequest $body): ?NginxConfigResponse
    {
        $path = $this->interpolatePath('/backend/v3/api/nginx/etc/{configId}', ['configId' => $this->serializePathParameter($configId, new PathParameterSpec('configId', 'simple', false))]);
        $payload = $body instanceof UpdateNginxConfigRequest ? $body->toArray() : $body;
        $result = $this->client->request('PUT', $path, [
            'json' => $payload,
        ]);
        return is_array($result) ? NginxConfigResponse::fromArray($result) : null;
    }

    /** 校验 Nginx 配置 */
    public function configsValidate(string $configId): ?NginxValidateResponse
    {
        $path = $this->interpolatePath('/backend/v3/api/nginx/etc/{configId}/validate', ['configId' => $this->serializePathParameter($configId, new PathParameterSpec('configId', 'simple', false))]);
        $result = $this->client->request('POST', $path, []);
        return is_array($result) ? NginxValidateResponse::fromArray($result) : null;
    }

    /** 部署 Nginx 配置 */
    public function configsDeploy(string $configId): ?NginxDeployResponse
    {
        $path = $this->interpolatePath('/backend/v3/api/nginx/etc/{configId}/deploy', ['configId' => $this->serializePathParameter($configId, new PathParameterSpec('configId', 'simple', false))]);
        $result = $this->client->request('POST', $path, []);
        return is_array($result) ? NginxDeployResponse::fromArray($result) : null;
    }

    /** 热加载 Nginx */
    public function reload(): ?NginxReloadResponse
    {
        $path = '/backend/v3/api/nginx/reload';
        $result = $this->client->request('POST', $path, []);
        return is_array($result) ? NginxReloadResponse::fromArray($result) : null;
    }

    /** 获取 Nginx 状态 */
    public function statusRetrieve(): ?NginxStatusResponse
    {
        $path = '/backend/v3/api/nginx/status';
        $result = $this->client->request('GET', $path, []);
        return is_array($result) ? NginxStatusResponse::fromArray($result) : null;
    }

}
