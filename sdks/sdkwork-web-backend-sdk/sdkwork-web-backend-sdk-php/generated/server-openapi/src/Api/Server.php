<?php

declare(strict_types=1);

namespace SDKWork\Web\BackendSdk\Api;

use SDKWork\Web\BackendSdk\Models\CreateServerRequest;
use SDKWork\Web\BackendSdk\Models\CreateServerResponse;
use SDKWork\Web\BackendSdk\Models\ServerPage;

final class ServerApi extends BaseApi
{
    /** 获取服务器列表 */
    public function serversList(?int $page = null, ?int $pageSize = null): ?ServerPage
    {
        $path = '/backend/v3/api/servers';
        $query = $this->buildQueryString([
            new QueryParameterSpec('page', $page, 'form', true, false, null),
            new QueryParameterSpec('pageSize', $pageSize, 'form', true, false, null),
        ]);
        $path = $this->appendQueryString($path, $query);
        $result = $this->client->request('GET', $path, []);
        return is_array($result) ? ServerPage::fromArray($result) : null;
    }

    /** 注册服务器 */
    public function serversCreate(array|CreateServerRequest $body): ?CreateServerResponse
    {
        $path = '/backend/v3/api/servers';
        $payload = $body instanceof CreateServerRequest ? $body->toArray() : $body;
        $result = $this->client->request('POST', $path, [
            'json' => $payload,
        ]);
        return is_array($result) ? CreateServerResponse::fromArray($result) : null;
    }

}
