<?php

declare(strict_types=1);

namespace SDKWork\Web\AppSdk\Models;

final class SiteResponse
{
    public ?string $id = null;

    public ?string $name = null;

    public ?string $slug = null;

    public ?string $description = null;

    public ?int $siteType = null;

    public ?int $status = null;

    public array $runtimeConfig = [];

    public ?string $createdAt = null;

    public ?string $updatedAt = null;

    public function __construct(array $data = [])
    {
        $this->id = array_key_exists('id', $data)
            ? $data['id']
            : null;
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
        $this->status = array_key_exists('status', $data)
            ? $data['status']
            : null;
        $this->runtimeConfig = array_key_exists('runtimeConfig', $data)
            ? is_array($data['runtimeConfig']) ? $data['runtimeConfig'] : []
            : [];
        $this->createdAt = array_key_exists('createdAt', $data)
            ? $data['createdAt']
            : null;
        $this->updatedAt = array_key_exists('updatedAt', $data)
            ? $data['updatedAt']
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
            'name' => $this->name,
            'slug' => $this->slug,
            'description' => $this->description,
            'siteType' => $this->siteType,
            'status' => $this->status,
            'runtimeConfig' => $this->runtimeConfig,
            'createdAt' => $this->createdAt,
            'updatedAt' => $this->updatedAt,
        ];
    }
}
