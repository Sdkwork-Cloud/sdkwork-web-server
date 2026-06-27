<?php

declare(strict_types=1);

namespace SDKWork\Web\BackendSdk\Models;

final class AgentCertificateBundle
{
    public ?string $certificateId = null;

    public ?string $certName = null;

    public ?string $fingerprint = null;

    public ?string $fullchainPem = null;

    public ?string $privkeyPem = null;

    public function __construct(array $data = [])
    {
        $this->certificateId = array_key_exists('certificateId', $data)
            ? $data['certificateId']
            : null;
        $this->certName = array_key_exists('certName', $data)
            ? $data['certName']
            : null;
        $this->fingerprint = array_key_exists('fingerprint', $data)
            ? $data['fingerprint']
            : null;
        $this->fullchainPem = array_key_exists('fullchainPem', $data)
            ? $data['fullchainPem']
            : null;
        $this->privkeyPem = array_key_exists('privkeyPem', $data)
            ? $data['privkeyPem']
            : null;
    }

    public static function fromArray(?array $data): ?self
    {
        return $data === null ? null : new self($data);
    }

    public function toArray(): array
    {
        return [
            'certificateId' => $this->certificateId,
            'certName' => $this->certName,
            'fingerprint' => $this->fingerprint,
            'fullchainPem' => $this->fullchainPem,
            'privkeyPem' => $this->privkeyPem,
        ];
    }
}
