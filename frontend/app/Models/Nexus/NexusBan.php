<?php

namespace App\Models\Nexus;

use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/** Account / Discord ban (temp or permanent). */
class NexusBan extends Model
{
    public $timestamps = false;

    protected $table = 'bans';

    protected $fillable = [
        'user_id',
        'discord_id',
        'reason',
        'banned_until',
        'created_by_web_user_id',
        'created_at',
        'lifted_by_web_user_id',
        'lifted_at',
    ];

    protected function casts(): array
    {
        return [
            'user_id' => 'integer',
            'created_by_web_user_id' => 'integer',
            'lifted_by_web_user_id' => 'integer',
            'banned_until' => 'datetime',
            'created_at' => 'datetime',
            'lifted_at' => 'datetime',
        ];
    }

    public function getConnectionName(): ?string
    {
        return config('nexus.connection', 'nexus');
    }

    public function nexusUser(): BelongsTo
    {
        return $this->belongsTo(NexusUser::class, 'user_id');
    }

    public function scopeActive(Builder $query): Builder
    {
        $now = now()->utc()->format('Y-m-d H:i:s');

        return $query
            ->whereNull('lifted_at')
            ->where(function (Builder $q) use ($now) {
                $q->whereNull('banned_until')
                    ->orWhere('banned_until', '>', $now);
            });
    }

    public function isPermanent(): bool
    {
        return $this->banned_until === null;
    }

    public function isActive(): bool
    {
        if ($this->lifted_at !== null) {
            return false;
        }

        if ($this->banned_until === null) {
            return true;
        }

        return $this->banned_until->isFuture();
    }
}
