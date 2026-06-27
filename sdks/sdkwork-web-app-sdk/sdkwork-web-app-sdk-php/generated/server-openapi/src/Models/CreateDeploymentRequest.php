<?php

declare(strict_types=1);

namespace SDKWork\Web\AppSdk\Models;

final class CreateDeploymentRequest
{
    public ?int $deployType = null;

    public ?string $versionTag = null;

    public ?string $commitHash = null;

    public ?string $sourceRef = null;

    public ?string $environment = null;

    public ?string $idempotencyKey = null;

    public function __construct(array $data = [])
    {
        $this->deployType = array_key_exists('deployType', $data)
            ? $data['deployType']
            : null;
        $this->versionTag = array_key_exists('versionTag', $data)
            ? $data['versionTag']
            : null;
        $this->commitHash = array_key_exists('commitHash', $data)
            ? $data['commitHash']
            : null;
        $this->sourceRef = array_key_exists('sourceRef', $data)
            ? $data['sourceRef']
            : null;
        $this->environment = array_key_exists('environment', $data)
            ? $data['environment']
            : null;
        $this->idempotencyKey = array_key_exists('idempotencyKey', $data)
            ? $data['idempotencyKey']
            : null;
    }

    public static function fromArray(?array $data): ?self
    {
        return $data === null ? null : new self($data);
    }

    public function toArray(): array
    {
        return [
            'deployType' => $this->deployType,
            'versionTag' => $this->versionTag,
            'commitHash' => $this->commitHash,
            'sourceRef' => $this->sourceRef,
            'environment' => $this->environment,
            'idempotencyKey' => $this->idempotencyKey,
        ];
    }
}
