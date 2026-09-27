<?php

namespace App\Models\Nexus;

use Illuminate\Database\Eloquent\Model;

/** Discord snowflake allowed to register when whitelist mode is on, or when registrations are closed. */
class NexusSignupWhitelistEntry extends Model
{
    public $timestamps = false;

    protected $table = 'signup_whitelist';

    protected $fillable = [
        'discord_id',
        'note',
        'created_by_web_user_id',
        'created_at',
    ];

    protected function casts(): array
    {
        return [
            'created_by_web_user_id' => 'integer',
            'created_at' => 'datetime',
        ];
    }

    public function getConnectionName(): ?string
    {
        return config('nexus.connection', 'nexus');
    }
}
