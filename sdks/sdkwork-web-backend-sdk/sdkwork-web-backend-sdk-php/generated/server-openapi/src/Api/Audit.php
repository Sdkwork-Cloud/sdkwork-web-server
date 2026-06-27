<?php

declare(strict_types=1);

namespace SDKWork\Web\BackendSdk\Api;

use SDKWork\Web\BackendSdk\Models\AuditLogPage;

final class AuditApi extends BaseApi
{
    /** 获取审计日志列表 */
    public function logsList(?int $page = null, ?int $pageSize = null, ?string $targetType = null, ?string $action = null, ?string $operatorId = null, ?string $startDate = null, ?string $endDate = null): ?AuditLogPage
    {
        $path = '/backend/v3/api/audit_logs';
        $query = $this->buildQueryString([
            new QueryParameterSpec('page', $page, 'form', true, false, null),
            new QueryParameterSpec('pageSize', $pageSize, 'form', true, false, null),
            new QueryParameterSpec('targetType', $targetType, 'form', true, false, null),
            new QueryParameterSpec('action', $action, 'form', true, false, null),
            new QueryParameterSpec('operatorId', $operatorId, 'form', true, false, null),
            new QueryParameterSpec('startDate', $startDate, 'form', true, false, null),
            new QueryParameterSpec('endDate', $endDate, 'form', true, false, null),
        ]);
        $path = $this->appendQueryString($path, $query);
        $result = $this->client->request('GET', $path, []);
        return is_array($result) ? AuditLogPage::fromArray($result) : null;
    }

}
