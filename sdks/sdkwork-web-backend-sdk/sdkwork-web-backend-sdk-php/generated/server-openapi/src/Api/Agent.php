<?php

declare(strict_types=1);

namespace SDKWork\Web\BackendSdk\Api;

use SDKWork\Web\BackendSdk\Models\AgentHeartbeatRequest;
use SDKWork\Web\BackendSdk\Models\AgentHeartbeatResponse;
use SDKWork\Web\BackendSdk\Models\AgentSyncResponse;

final class AgentApi extends BaseApi
{
    /** 边缘节点心跳 */
    public function heartbeat(array|AgentHeartbeatRequest $body): ?AgentHeartbeatResponse
    {
        $path = '/backend/v3/api/agent/heartbeat';
        $payload = $body instanceof AgentHeartbeatRequest ? $body->toArray() : $body;
        $result = $this->client->request('POST', $path, [
            'json' => $payload,
        ]);
        return is_array($result) ? AgentHeartbeatResponse::fromArray($result) : null;
    }

    /** 拉取 nginx 配置与证书 bundle */
    public function sync(?string $ifSyncVersion = null): ?AgentSyncResponse
    {
        $path = '/backend/v3/api/agent/sync';
        $query = $this->buildQueryString([
            new QueryParameterSpec('ifSyncVersion', $ifSyncVersion, 'form', true, false, null),
        ]);
        $path = $this->appendQueryString($path, $query);
        $result = $this->client->request('GET', $path, []);
        return is_array($result) ? AgentSyncResponse::fromArray($result) : null;
    }

}
