<?php

declare(strict_types=1);

namespace SDKWork\Web\AppSdk\Models;

final class UpdateSiteRequest
{
    public ?string $name = null;

    public ?string $description = null;

    public array $runtimeConfig = [];

    public function __construct(array $data = [])
    {
        $this->name = array_key_exists('name', $data)
            ? $data['name']
            : null;
        $this->description = array_key_exists('description', $data)
            ? $data['description']
            : null;
        $this->runtimeConfig = array_key_exists('runtimeConfig', $data)
            ? is_array($data['runtimeConfig']) ? $data['runtimeConfig'] : []
            : [];
    }

    public static function fromArray(?array $data): ?self
    {
        return $data === null ? null : new self($data);
    }

    public function toArray(): array
    {
        return [
            'name' => $this->name,
            'description' => $this->description,
            'runtimeConfig' => $this->runtimeConfig,
        ];
    }
}
