<?php

declare(strict_types=1);

namespace SDKWork\Web\AppSdk\Api;

use SDKWork\Web\AppSdk\Models\CreateSiteRequest;
use SDKWork\Web\AppSdk\Models\SitePage;
use SDKWork\Web\AppSdk\Models\SiteResponse;
use SDKWork\Web\AppSdk\Models\UpdateSiteRequest;

final class SiteApi extends BaseApi
{
    /** 获取站点列表 */
    public function sitesList(?int $page = null, ?int $pageSize = null, ?int $status = null, ?int $siteType = null, ?string $keyword = null): ?SitePage
    {
        $path = '/app/v3/api/sites';
        $query = $this->buildQueryString([
            new QueryParameterSpec('page', $page, 'form', true, false, null),
            new QueryParameterSpec('pageSize', $pageSize, 'form', true, false, null),
            new QueryParameterSpec('status', $status, 'form', true, false, null),
            new QueryParameterSpec('siteType', $siteType, 'form', true, false, null),
            new QueryParameterSpec('keyword', $keyword, 'form', true, false, null),
        ]);
        $path = $this->appendQueryString($path, $query);
        $result = $this->client->request('GET', $path, []);
        return is_array($result) ? SitePage::fromArray($result) : null;
    }

    /** 创建站点 */
    public function sitesCreate(array|CreateSiteRequest $body): ?SiteResponse
    {
        $path = '/app/v3/api/sites';
        $payload = $body instanceof CreateSiteRequest ? $body->toArray() : $body;
        $result = $this->client->request('POST', $path, [
            'json' => $payload,
        ]);
        return is_array($result) ? SiteResponse::fromArray($result) : null;
    }

    /** 获取站点详情 */
    public function sitesRetrieve(string $siteId): ?SiteResponse
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false))]);
        $result = $this->client->request('GET', $path, []);
        return is_array($result) ? SiteResponse::fromArray($result) : null;
    }

    /** 更新站点 */
    public function sitesUpdate(string $siteId, array|UpdateSiteRequest $body): ?SiteResponse
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false))]);
        $payload = $body instanceof UpdateSiteRequest ? $body->toArray() : $body;
        $result = $this->client->request('PATCH', $path, [
            'json' => $payload,
        ]);
        return is_array($result) ? SiteResponse::fromArray($result) : null;
    }

    /** 删除站点 */
    public function sitesDelete(string $siteId): mixed
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false))]);
        $result = $this->client->request('DELETE', $path, []);
        return $result;
    }

    /** 激活站点 */
    public function sitesActivate(string $siteId): ?SiteResponse
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}/activate', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false))]);
        $result = $this->client->request('POST', $path, []);
        return is_array($result) ? SiteResponse::fromArray($result) : null;
    }

    /** 暂停站点 */
    public function sitesPause(string $siteId): ?SiteResponse
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}/pause', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false))]);
        $result = $this->client->request('POST', $path, []);
        return is_array($result) ? SiteResponse::fromArray($result) : null;
    }

}
