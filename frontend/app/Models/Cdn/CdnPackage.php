<?php

namespace App\Models\Cdn;

use App\Models\User;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

class CdnPackage extends Model
{
    public const KIND_PRISM = 'prism';

    /** Launcher builds. Title id is fixed to this value. */
    public const KIND_REFRACTED = 'refracted';

    public const KINDS = [self::KIND_PRISM, self::KIND_REFRACTED];

    protected $table = 'cdn_packages';

    protected $fillable = [
        'kind',
        'title_id',
        'channel',
        'version',
        'revision',
        'label',
        'notes',
        'status',
        'is_latest',
        'published_at',
        'created_by',
    ];

    protected function casts(): array
    {
        return [
            'revision' => 'integer',
            'is_latest' => 'boolean',
            'published_at' => 'datetime',
        ];
    }

    public function artifacts(): HasMany
    {
        return $this->hasMany(CdnArtifact::class, 'package_id');
    }

    public function creator(): BelongsTo
    {
        return $this->belongsTo(User::class, 'created_by');
    }

    public function isRefracted(): bool
    {
        return $this->kind === self::KIND_REFRACTED;
    }

    /** Public path prefix: prism/{title} or refracted. */
    public function urlPrefix(): string
    {
        return $this->isRefracted() ? self::KIND_REFRACTED : "prism/{$this->title_id}";
    }

    public function isPublished(): bool
    {
        return $this->status === 'published';
    }

    public function displayName(): string
    {
        $label = $this->label ? " ({$this->label})" : '';

        return "{$this->title_id}/{$this->channel} {$this->version} r{$this->revision}{$label}";
    }

    public function scopePublished($query)
    {
        return $query->where('status', 'published');
    }

    public function scopeOfKind($query, string $kind)
    {
        return $query->where('kind', $kind);
    }

    public function scopeForTitleChannel($query, string $titleId, string $channel)
    {
        return $query
            ->where('title_id', $titleId)
            ->where('channel', $channel);
    }
}
