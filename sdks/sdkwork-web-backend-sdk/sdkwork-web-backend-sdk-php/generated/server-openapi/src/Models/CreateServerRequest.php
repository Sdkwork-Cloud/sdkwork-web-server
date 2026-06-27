<?php

declare(strict_types=1);

namespace SDKWork\Web\BackendSdk\Models;

final class CreateServerRequest
{
    public ?string $name = null;

    public ?string $host = null;

    public ?int $sshPort = null;

    public ?string $sshUser = null;

    public ?string $sshKeyPath = null;

    public ?string $description = null;

    public function __construct(array $data = [])
    {
        $this->name = array_key_exists('name', $data)
            ? $data['name']
            : null;
        $this->host = array_key_exists('host', $data)
            ? $data['host']
            : null;
        $this->sshPort = array_key_exists('sshPort', $data)
            ? $data['sshPort']
            : null;
        $this->sshUser = array_key_exists('sshUser', $data)
            ? $data['sshUser']
            : null;
        $this->sshKeyPath = array_key_exists('sshKeyPath', $data)
            ? $data['sshKeyPath']
            : null;
        $this->description = array_key_exists('description', $data)
            ? $data['description']
            : null;
    }

    public static function fromArray(?array $data): ?self
    {
        return $data === null ? null : new self($data);
    }

    public function toArray(): array
    {
        return [
            'name' => $this->name,
            'host' => $this->host,
            'sshPort' => $this->sshPort,
            'sshUser' => $this->sshUser,
            'sshKeyPath' => $this->sshKeyPath,
            'description' => $this->description,
        ];
    }
}
