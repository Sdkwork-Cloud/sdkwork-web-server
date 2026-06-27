<?php

declare(strict_types=1);

namespace SDKWork\Web\AppSdk\Api;

use SDKWork\Web\AppSdk\Models\CertificatePage;
use SDKWork\Web\AppSdk\Models\CertificateResponse;
use SDKWork\Web\AppSdk\Models\CreateCertificateRequest;

final class CertificateApi extends BaseApi
{
    /** 获取证书列表 */
    public function certificatesList(?int $page = null, ?int $pageSize = null): ?CertificatePage
    {
        $path = '/app/v3/api/certificates';
        $query = $this->buildQueryString([
            new QueryParameterSpec('page', $page, 'form', true, false, null),
            new QueryParameterSpec('pageSize', $pageSize, 'form', true, false, null),
        ]);
        $path = $this->appendQueryString($path, $query);
        $result = $this->client->request('GET', $path, []);
        return is_array($result) ? CertificatePage::fromArray($result) : null;
    }

    /** 申请证书 */
    public function certificatesCreate(array|CreateCertificateRequest $body): ?CertificateResponse
    {
        $path = '/app/v3/api/certificates';
        $payload = $body instanceof CreateCertificateRequest ? $body->toArray() : $body;
        $result = $this->client->request('POST', $path, [
            'json' => $payload,
        ]);
        return is_array($result) ? CertificateResponse::fromArray($result) : null;
    }

}
