<?php

declare(strict_types=1);

namespace SDKWork\Web\AppSdk\Models;

final class DeploymentResponse
{
    public ?string $id = null;

    public ?string $siteId = null;

    public ?int $deployType = null;

    public ?string $versionTag = null;

    public ?int $status = null;

    public ?string $startedAt = null;

    public ?string $completedAt = null;

    /** Deployment duration in milliseconds as a string to avoid JavaScript precision loss. */
    public ?string $durationMs = null;

    public ?string $createdAt = null;

    public function __construct(array $data = [])
    {
        $this->id = array_key_exists('id', $data)
            ? $data['id']
            : null;
        $this->siteId = array_key_exists('siteId', $data)
            ? $data['siteId']
            : null;
        $this->deployType = array_key_exists('deployType', $data)
            ? $data['deployType']
            : null;
        $this->versionTag = array_key_exists('versionTag', $data)
            ? $data['versionTag']
            : null;
        $this->status = array_key_exists('status', $data)
            ? $data['status']
            : null;
        $this->startedAt = array_key_exists('startedAt', $data)
            ? $data['startedAt']
            : null;
        $this->completedAt = array_key_exists('completedAt', $data)
            ? $data['completedAt']
            : null;
        $this->durationMs = array_key_exists('durationMs', $data)
            ? $data['durationMs']
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
            'siteId' => $this->siteId,
            'deployType' => $this->deployType,
            'versionTag' => $this->versionTag,
            'status' => $this->status,
            'startedAt' => $this->startedAt,
            'completedAt' => $this->completedAt,
            'durationMs' => $this->durationMs,
            'createdAt' => $this->createdAt,
        ];
    }
}
