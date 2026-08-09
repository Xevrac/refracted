<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Facades\Storage;

class Community extends Model
{
    protected $fillable = [
        'name',
        'tagline',
        'blurb',
        'image_path',
        'icon_path',
        'url',
        'sort_order',
        'is_published',
    ];

    protected function casts(): array
    {
        return [
            'is_published' => 'boolean',
            'sort_order' => 'integer',
        ];
    }

    public function scopePublished(Builder $query): Builder
    {
        return $query->where('is_published', true);
    }

    public function scopeOrdered(Builder $query): Builder
    {
        return $query->orderBy('sort_order')->orderBy('name');
    }

    public function imageUrl(): ?string
    {
        return $this->publicUrl($this->image_path);
    }

    public function iconUrl(): ?string
    {
        return $this->publicUrl($this->icon_path);
    }

    public function linkLabel(): ?string
    {
        if (! filled($this->url)) {
            return null;
        }

        $host = parse_url($this->url, PHP_URL_HOST);

        return $host ? preg_replace('/^www\./i', '', $host) : 'Visit';
    }

    public function deleteMedia(): void
    {
        $this->deleteStoredFile($this->image_path);
        $this->deleteStoredFile($this->icon_path);
    }

    protected function publicUrl(?string $path): ?string
    {
        if (! filled($path)) {
            return null;
        }

        if (str_starts_with($path, 'images/')) {
            return asset($path);
        }

        return Storage::disk('public')->url($path);
    }

    protected function deleteStoredFile(?string $path): void
    {
        if (filled($path) && ! str_starts_with($path, 'images/') && Storage::disk('public')->exists($path)) {
            Storage::disk('public')->delete($path);
        }
    }
}
