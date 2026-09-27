<?php

namespace App\Services\Nexus;

use App\Models\Nexus\NexusPersona;
use App\Models\Nexus\NexusUser;
use App\Models\User;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Str;

/**
 * Provisions Nexus accounts.
 */
class NexusProvisioner
{
    public function __construct(
        protected NexusIdGenerator $ids,
    ) {}

    public function ensureForWebsiteUser(User $websiteUser, ?string $displayName = null): NexusUser
    {
        if (! filled($websiteUser->discord_id)) {
            throw new \InvalidArgumentException('Nexus requires a linked Discord account.');
        }

        $discordId = (string) $websiteUser->discord_id;
        $name = $this->sanitizeDisplayName(
            $displayName
                ?: $websiteUser->discord_username
                ?: $websiteUser->name
                ?: 'Player'
        );
        $email = (string) ($websiteUser->email ?: "{$discordId}@discord.local");

        $connection = config('nexus.connection', 'nexus');

        return DB::connection($connection)->transaction(function () use ($websiteUser, $discordId, $name, $email) {
            $nexusUser = NexusUser::query()
                ->where('discord_id', $discordId)
                ->first();

            if ($nexusUser === null && $websiteUser->id) {
                $nexusUser = NexusUser::query()
                    ->where('web_user_id', $websiteUser->id)
                    ->first();
            }

            if ($nexusUser === null) {
                $nexusUser = NexusUser::query()->create([
                    'id' => $this->ids->nextUserId(),
                    'username' => $this->ids->uniqueUsername($name, $discordId),
                    'email' => $email,
                    'discord_id' => $discordId,
                    'web_user_id' => $websiteUser->id,
                    'secret_hash' => '',
                    'secret_salt' => '',
                    'created_at' => now()->utc()->format('Y-m-d H:i:s'),
                ]);

                NexusPersona::query()->create([
                    'id' => $this->ids->nextPersonaId(),
                    'user_id' => $nexusUser->id,
                    'display_name' => $this->allocateDisplayName($name),
                    'created_at' => now()->utc()->format('Y-m-d H:i:s'),
                ]);

                return $nexusUser->fresh(['personas']);
            }

            $nexusUser->forceFill([
                'email' => $email,
                'discord_id' => $discordId,
                'web_user_id' => $websiteUser->id,
            ])->save();

            $persona = $nexusUser->defaultPersona();
            if ($persona === null) {
                NexusPersona::query()->create([
                    'id' => $this->ids->nextPersonaId(),
                    'user_id' => $nexusUser->id,
                    'display_name' => $this->allocateDisplayName($name),
                    'created_at' => now()->utc()->format('Y-m-d H:i:s'),
                ]);
            }

            return $nexusUser->fresh(['personas']);
        });
    }

    public function findForWebsiteUser(User $websiteUser): ?NexusUser
    {
        if (filled($websiteUser->discord_id)) {
            $found = NexusUser::query()->where('discord_id', (string) $websiteUser->discord_id)->first();
            if ($found) {
                return $found;
            }
        }

        if ($websiteUser->id) {
            return NexusUser::query()->where('web_user_id', $websiteUser->id)->first();
        }

        return null;
    }

    public function updateDisplayName(NexusUser $nexusUser, string $displayName, bool $bypassCooldown = false): NexusPersona
    {
        $name = $this->sanitizeDisplayName($displayName);
        $persona = $nexusUser->defaultPersona();

        if ($persona === null) {
            throw new \RuntimeException('Nexus user has no persona.');
        }

        if (strcasecmp($persona->display_name, $name) === 0) {
            return $persona;
        }

        if (! $bypassCooldown) {
            $next = $persona->nextDisplayNameChangeAt();
            if ($next !== null) {
                $days = $persona->displayNameCooldownDays();
                throw \Illuminate\Validation\ValidationException::withMessages([
                    'display_name' => "Display name can only be changed every {$days} days. Next change available {$next->utc()->format('Y-m-d')} UTC.",
                ]);
            }
        }

        $this->assertDisplayNameAvailable($name, (int) $persona->id);

        $persona->forceFill([
            'display_name' => $name,
            'display_name_changed_at' => now()->utc()->format('Y-m-d H:i:s'),
        ])->save();

        return $persona->fresh();
    }

    protected function sanitizeDisplayName(string $name): string
    {
        $name = trim(preg_replace('/\s+/', ' ', $name) ?? '');
        $name = Str::limit($name, 64, '');

        return $name !== '' ? $name : 'Player';
    }

    protected function assertDisplayNameAvailable(string $name, ?int $ignorePersonaId = null): void
    {
        if ($this->displayNameTaken($name, $ignorePersonaId)) {
            throw \Illuminate\Validation\ValidationException::withMessages([
                'display_name' => 'That display name is already taken.',
            ]);
        }
    }

    protected function displayNameTaken(string $name, ?int $ignorePersonaId = null): bool
    {
        $query = NexusPersona::query()
            ->whereRaw('LOWER(display_name) = ?', [Str::lower($name)]);

        if ($ignorePersonaId) {
            $query->where('id', '!=', $ignorePersonaId);
        }

        return $query->exists();
    }

    /** Pick a free display name for first-time provision (suffix if needed). */
    protected function allocateDisplayName(string $desired): string
    {
        $name = $this->sanitizeDisplayName($desired);
        if (! $this->displayNameTaken($name)) {
            return $name;
        }

        for ($i = 2; $i < 1000; $i++) {
            $suffix = (string) $i;
            $base = Str::limit($name, 64 - strlen($suffix), '');
            $candidate = $base.$suffix;
            if (! $this->displayNameTaken($candidate)) {
                return $candidate;
            }
        }

        throw new \RuntimeException('Unable to allocate a unique display name.');
    }
}
