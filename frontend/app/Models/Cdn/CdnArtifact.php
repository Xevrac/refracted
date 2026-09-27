<?php

namespace App\Models\Cdn;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Support\Facades\Storage;

class CdnArtifact extends Model
{
    protected $table = 'cdn_artifacts';

    protected $fillable = [
        'package_id',
        'path',
        'storage_path',
        'sha256',
        'size_bytes',
        'content_type',
    ];

    protected function casts(): array
    {
        return [
            'size_bytes' => 'integer',
        ];
    }

    public function package(): BelongsTo
    {
        return $this->belongsTo(CdnPackage::class, 'package_id');
    }

    public function absolutePath(): string
    {
        return Storage::disk(config('cdn.disk', 'cdn'))->path($this->storage_path);
    }

    public function existsOnDisk(): bool
    {
        return Storage::disk(config('cdn.disk', 'cdn'))->exists($this->storage_path);
    }
}
