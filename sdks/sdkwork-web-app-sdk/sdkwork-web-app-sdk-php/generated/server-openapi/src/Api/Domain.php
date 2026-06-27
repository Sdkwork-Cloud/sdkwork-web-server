<?php

declare(strict_types=1);

namespace SDKWork\Web\AppSdk\Api;

use SDKWork\Web\AppSdk\Models\CreateDomainRequest;
use SDKWork\Web\AppSdk\Models\DomainPage;
use SDKWork\Web\AppSdk\Models\DomainResponse;
use SDKWork\Web\AppSdk\Models\DomainVerifyResponse;

final class DomainApi extends BaseApi
{
    /** 获取站点域名列表 */
    public function sitesDomainsList(string $siteId, ?int $page = null, ?int $pageSize = null): ?DomainPage
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}/domains', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false))]);
        $query = $this->buildQueryString([
            new QueryParameterSpec('page', $page, 'form', true, false, null),
            new QueryParameterSpec('pageSize', $pageSize, 'form', true, false, null),
        ]);
        $path = $this->appendQueryString($path, $query);
        $result = $this->client->request('GET', $path, []);
        return is_array($result) ? DomainPage::fromArray($result) : null;
    }

    /** 绑定域名 */
    public function sitesDomainsCreate(string $siteId, array|CreateDomainRequest $body): ?DomainResponse
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}/domains', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false))]);
        $payload = $body instanceof CreateDomainRequest ? $body->toArray() : $body;
        $result = $this->client->request('POST', $path, [
            'json' => $payload,
        ]);
        return is_array($result) ? DomainResponse::fromArray($result) : null;
    }

    /** 获取域名详情 */
    public function sitesDomainsRetrieve(string $siteId, string $domainId): ?DomainResponse
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}/domains/{domainId}', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false)), 'domainId' => $this->serializePathParameter($domainId, new PathParameterSpec('domainId', 'simple', false))]);
        $result = $this->client->request('GET', $path, []);
        return is_array($result) ? DomainResponse::fromArray($result) : null;
    }

    /** 解绑域名 */
    public function sitesDomainsDelete(string $siteId, string $domainId): mixed
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}/domains/{domainId}', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false)), 'domainId' => $this->serializePathParameter($domainId, new PathParameterSpec('domainId', 'simple', false))]);
        $result = $this->client->request('DELETE', $path, []);
        return $result;
    }

    /** 验证域名所有权 */
    public function sitesDomainsVerify(string $siteId, string $domainId): ?DomainVerifyResponse
    {
        $path = $this->interpolatePath('/app/v3/api/sites/{siteId}/domains/{domainId}/verify', ['siteId' => $this->serializePathParameter($siteId, new PathParameterSpec('siteId', 'simple', false)), 'domainId' => $this->serializePathParameter($domainId, new PathParameterSpec('domainId', 'simple', false))]);
        $result = $this->client->request('POST', $path, []);
        return is_array($result) ? DomainVerifyResponse::fromArray($result) : null;
    }

}
