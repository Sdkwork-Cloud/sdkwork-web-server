<?php

declare(strict_types=1);

namespace SDKWork\Web\AppSdk\Models;

final class CertificateResponse
{
    public ?string $id = null;

    public ?string $certName = null;

    public ?int $certType = null;

    public ?string $issuer = null;

    public ?string $notBefore = null;

    public ?string $notAfter = null;

    public ?bool $autoRenew = null;

    public ?int $status = null;

    public ?string $createdAt = null;

    public function __construct(array $data = [])
    {
        $this->id = array_key_exists('id', $data)
            ? $data['id']
            : null;
        $this->certName = array_key_exists('certName', $data)
            ? $data['certName']
            : null;
        $this->certType = array_key_exists('certType', $data)
            ? $data['certType']
            : null;
        $this->issuer = array_key_exists('issuer', $data)
            ? $data['issuer']
            : null;
        $this->notBefore = array_key_exists('notBefore', $data)
            ? $data['notBefore']
            : null;
        $this->notAfter = array_key_exists('notAfter', $data)
            ? $data['notAfter']
            : null;
        $this->autoRenew = array_key_exists('autoRenew', $data)
            ? $data['autoRenew']
            : null;
        $this->status = array_key_exists('status', $data)
            ? $data['status']
            : null;
        $this->createdAt = array_key_exists('createdAt', $data)
            ? $data['createdAt']
            : null;
    }

    public static function fromArray(?array $data): ?self
    {
        return $data === null ? null : new self($data);
    }

    public function toArray(): array
    {
        return [
            'id' => $this->id,
            'certName' => $this->certName,
            'certType' => $this->certType,
            'issuer' => $this->issuer,
            'notBefore' => $this->notBefore,
            'notAfter' => $this->notAfter,
            'autoRenew' => $this->autoRenew,
            'status' => $this->status,
            'createdAt' => $this->createdAt,
        ];
    }
}
