<?php

namespace App\Models\Nexus;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * Nexus account.
 */
class NexusUser extends Model
{
    public $incrementing = false;

    public $timestamps = false;

    protected $table = 'users';

    protected $keyType = 'int';

    protected $fillable = [
        'id',
        'username',
        'email',
        'discord_id',
        'web_user_id',
        'secret_hash',
        'secret_salt',
        'created_at',
    ];

    protected $hidden = [
        'secret_hash',
        'secret_salt',
    ];

    protected function casts(): array
    {
        return [
            'id' => 'integer',
            'web_user_id' => 'integer',
            'created_at' => 'datetime',
        ];
    }

    public function getConnectionName(): ?string
    {
        return config('nexus.connection', 'nexus');
    }

    public function personas(): HasMany
    {
        return $this->hasMany(NexusPersona::class, 'user_id');
    }

    public function authSessions(): HasMany
    {
        return $this->hasMany(NexusAuthSession::class, 'user_id');
    }

    public function defaultPersona(): ?NexusPersona
    {
        return $this->personas()->orderBy('id')->first();
    }
}
