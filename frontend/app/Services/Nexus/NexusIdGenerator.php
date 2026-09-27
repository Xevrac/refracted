<?php

namespace App\Services\Nexus;

use App\Models\Nexus\NexusPersona;
use App\Models\Nexus\NexusUser;
use Illuminate\Support\Str;

/** Nexus id allocation. */
class NexusIdGenerator
{
    public const MIN = 1_000_000_000_000;

    public const MAX = 9_999_999_999_999;

    public function nextUserId(): int
    {
        return $this->uniqueId(fn (int $id) => NexusUser::query()->whereKey($id)->exists());
    }

    public function nextPersonaId(): int
    {
        return $this->uniqueId(fn (int $id) => NexusPersona::query()->whereKey($id)->exists());
    }

    /** @param  callable(int): bool  $exists */
    protected function uniqueId(callable $exists): int
    {
        for ($attempt = 0; $attempt < 32; $attempt++) {
            $id = random_int(self::MIN, self::MAX);
            if (! $exists($id)) {
                return $id;
            }
        }

        throw new \RuntimeException('Unable to allocate a unique Nexus id.');
    }

    public function uniqueUsername(string $displayName, string $discordId): string
    {
        $base = Str::lower(preg_replace('/[^a-zA-Z0-9_]/', '', $displayName) ?: '');
        if ($base === '' || strlen($base) < 2) {
            $base = 'player'.$discordId;
        }
        $base = Str::limit($base, 48, '');

        $candidate = $base;
        $n = 0;
        while (NexusUser::query()->where('username', $candidate)->exists()) {
            $n++;
            $suffix = '_'.$n;
            $candidate = Str::limit($base, 64 - strlen($suffix), '').$suffix;
        }

        return $candidate;
    }
}
