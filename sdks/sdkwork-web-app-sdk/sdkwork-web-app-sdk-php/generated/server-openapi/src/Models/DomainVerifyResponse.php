<?php

declare(strict_types=1);

namespace SDKWork\Web\AppSdk\Models;

final class DomainVerifyResponse
{
    public ?bool $verified = null;

    public ?string $method = null;

    public ?string $token = null;

    public function __construct(array $data = [])
    {
        $this->verified = array_key_exists('verified', $data)
            ? $data['verified']
            : null;
        $this->method = array_key_exists('method', $data)
            ? $data['method']
            : null;
        $this->token = array_key_exists('token', $data)
            ? $data['token']
            : null;
    }

    public static function fromArray(?array $data): ?self
    {
        return $data === null ? null : new self($data);
    }

    public function toArray(): array
    {
        return [
            'verified' => $this->verified,
            'method' => $this->method,
            'token' => $this->token,
        ];
    }
}
