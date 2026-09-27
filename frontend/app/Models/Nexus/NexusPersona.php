<?php

namespace App\Models\Nexus;

use Carbon\CarbonInterface;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Support\Carbon;

class NexusPersona extends Model
{
    public $incrementing = false;

    public $timestamps = false;

    protected $table = 'personas';

    protected $keyType = 'int';

    protected $fillable = [
        'id',
        'user_id',
        'display_name',
        'display_name_changed_at',
        'created_at',
    ];

    protected function casts(): array
    {
        return [
            'id' => 'integer',
            'user_id' => 'integer',
            'display_name_changed_at' => 'datetime',
            'created_at' => 'datetime',
        ];
    }

    public function getConnectionName(): ?string
    {
        return config('nexus.connection', 'nexus');
    }

    public function user(): BelongsTo
    {
        return $this->belongsTo(NexusUser::class, 'user_id');
    }

    public function authSessions(): HasMany
    {
        return $this->hasMany(NexusAuthSession::class, 'persona_id');
    }

    public function displayNameCooldownDays(): int
    {
        return max(1, (int) config('nexus.display_name_cooldown_days', 30));
    }

    public function canChangeDisplayName(?CarbonInterface $now = null): bool
    {
        return $this->nextDisplayNameChangeAt($now) === null;
    }

    /** @return Carbon|null Next eligible instant, or null when a change is allowed now. */
    public function nextDisplayNameChangeAt(?CarbonInterface $now = null): ?Carbon
    {
        if ($this->display_name_changed_at === null) {
            return null;
        }

        $now = Carbon::parse($now ?? now()->utc());
        $eligible = $this->display_name_changed_at->copy()->utc()->addDays($this->displayNameCooldownDays());

        return $eligible->greaterThan($now) ? $eligible : null;
    }
}
