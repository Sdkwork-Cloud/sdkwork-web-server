<?php

declare(strict_types=1);

namespace SDKWork\Web\BackendSdk\Models;

final class ProblemDetail
{
    public ?string $type = null;

    public ?string $title = null;

    public ?int $status = null;

    public ?string $detail = null;

    public ?string $instance = null;

    public ?string $requestId = null;

    public function __construct(array $data = [])
    {
        $this->type = array_key_exists('type', $data)
            ? $data['type']
            : null;
        $this->title = array_key_exists('title', $data)
            ? $data['title']
            : null;
        $this->status = array_key_exists('status', $data)
            ? $data['status']
            : null;
        $this->detail = array_key_exists('detail', $data)
            ? $data['detail']
            : null;
        $this->instance = array_key_exists('instance', $data)
            ? $data['instance']
            : null;
        $this->requestId = array_key_exists('requestId', $data)
            ? $data['requestId']
            : null;
    }

    public static function fromArray(?array $data): ?self
    {
        return $data === null ? null : new self($data);
    }

    public function toArray(): array
    {
        return [
            'type' => $this->type,
            'title' => $this->title,
            'status' => $this->status,
            'detail' => $this->detail,
            'instance' => $this->instance,
            'requestId' => $this->requestId,
        ];
    }
}
