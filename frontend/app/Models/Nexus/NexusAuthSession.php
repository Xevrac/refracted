<?php

namespace App\Models\Nexus;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class NexusAuthSession extends Model
{
    public $timestamps = false;

    protected $table = 'auth_sessions';

    protected $fillable = [
        'user_id',
        'persona_id',
        'token_hash',
        'jwt_id',
        'expires_at',
        'revoked_at',
        'created_at',
        'last_seen_at',
        'client_ip',
        'country_code',
        'game_id',
    ];

    protected $hidden = [
        'token_hash',
        'jwt_id',
    ];

    protected function casts(): array
    {
        return [
            'user_id' => 'integer',
            'persona_id' => 'integer',
            'expires_at' => 'datetime',
            'revoked_at' => 'datetime',
            'created_at' => 'datetime',
            'last_seen_at' => 'datetime',
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

    public function persona(): BelongsTo
    {
        return $this->belongsTo(NexusPersona::class, 'persona_id');
    }

    public function isActive(): bool
    {
        if ($this->revoked_at !== null) {
            return false;
        }

        return $this->expires_at === null || $this->expires_at->isFuture();
    }
}
