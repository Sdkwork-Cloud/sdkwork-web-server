<?php

declare(strict_types=1);

namespace SDKWork\Web\AppSdk\Models;

final class CreateSiteRequest
{
    public ?string $name = null;

    public ?string $slug = null;

    public ?string $description = null;

    public ?int $siteType = null;

    public array $runtimeConfig = [];

    public function __construct(array $data = [])
    {
        $this->name = array_key_exists('name', $data)
            ? $data['name']
            : null;
        $this->slug = array_key_exists('slug', $data)
            ? $data['slug']
            : null;
        $this->description = array_key_exists('description', $data)
            ? $data['description']
            : null;
        $this->siteType = array_key_exists('siteType', $data)
            ? $data['siteType']
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
            'slug' => $this->slug,
            'description' => $this->description,
            'siteType' => $this->siteType,
            'runtimeConfig' => $this->runtimeConfig,
        ];
    }
}
