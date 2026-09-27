<?php

namespace App\Models\Nexus;

use Illuminate\Database\Eloquent\Model;

/** Key/value settings for Nexus access control. */
class NexusSetting extends Model
{
    public $timestamps = false;

    public $incrementing = false;

    protected $table = 'settings';

    protected $primaryKey = 'key';

    protected $keyType = 'string';

    protected $fillable = [
        'key',
        'value',
        'updated_at',
    ];

    protected function casts(): array
    {
        return [
            'updated_at' => 'datetime',
        ];
    }

    public function getConnectionName(): ?string
    {
        return config('nexus.connection', 'nexus');
    }

    public static function getBool(string $key, bool $default = false): bool
    {
        $row = static::query()->find($key);
        if ($row === null) {
            return $default;
        }

        return in_array(strtolower(trim((string) $row->value)), ['1', 'true', 'yes', 'on'], true);
    }

    public static function setBool(string $key, bool $value): void
    {
        static::query()->updateOrCreate(
            ['key' => $key],
            [
                'value' => $value ? '1' : '0',
                'updated_at' => now()->utc()->format('Y-m-d H:i:s'),
            ]
        );
    }
}
