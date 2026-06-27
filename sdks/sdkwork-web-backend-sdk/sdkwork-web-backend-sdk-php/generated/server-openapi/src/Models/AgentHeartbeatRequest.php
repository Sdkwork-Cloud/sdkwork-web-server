<?php

declare(strict_types=1);

namespace SDKWork\Web\BackendSdk\Models;

final class AgentHeartbeatRequest
{
    public ?string $agentVersion = null;

    public ?bool $nginxEnabled = null;

    /** Number of active nginx configs reported by the agent as a string. */
    public ?string $activeConfigs = null;

    /** Last successfully applied syncVersion reported by the edge agent. */
    public ?string $lastSyncVersion = null;

    public function __construct(array $data = [])
    {
        $this->agentVersion = array_key_exists('agentVersion', $data)
            ? $data['agentVersion']
            : null;
        $this->nginxEnabled = array_key_exists('nginxEnabled', $data)
            ? $data['nginxEnabled']
            : null;
        $this->activeConfigs = array_key_exists('activeConfigs', $data)
            ? $data['activeConfigs']
            : null;
        $this->lastSyncVersion = array_key_exists('lastSyncVersion', $data)
            ? $data['lastSyncVersion']
            : null;
    }

    public static function fromArray(?array $data): ?self
    {
        return $data === null ? null : new self($data);
    }

    public function toArray(): array
    {
        return [
            'agentVersion' => $this->agentVersion,
            'nginxEnabled' => $this->nginxEnabled,
            'activeConfigs' => $this->activeConfigs,
            'lastSyncVersion' => $this->lastSyncVersion,
        ];
    }
}
