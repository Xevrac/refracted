<?php

namespace App\Services;

use App\Models\User;
use Illuminate\Auth\AuthenticationException;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Str;
use Laravel\Socialite\Contracts\User as SocialiteUser;

class DiscordAuthService
{
    public function resolveForLogin(SocialiteUser $discordUser): User
    {
        $discordId = (string) $discordUser->getId();

        if (! $this->isAllowedAdmin($discordId)) {
            throw new AuthenticationException('This Discord account is not allowed to administer Refracted.');
        }

        $user = User::query()->updateOrCreate(
            ['discord_id' => $discordId],
            [
                'name' => $this->resolveDisplayName($discordUser),
                'email' => $discordUser->getEmail() ?: "{$discordId}@discord.local",
                'password' => Hash::make(Str::random(40)),
                'email_verified_at' => now(),
                'is_admin' => true,
                'discord_username' => $this->resolveDisplayName($discordUser),
                'discord_avatar' => $discordUser->getAvatar(),
                'last_login_at' => now(),
            ],
        );

        return $user;
    }

    protected function isAllowedAdmin(string $discordId): bool
    {
        $allowed = collect(explode(',', (string) config('services.discord.admin_ids', '')))
            ->map(fn (string $id) => trim($id))
            ->filter()
            ->values();

        return $allowed->contains($discordId);
    }

    protected function resolveDisplayName(SocialiteUser $discordUser): string
    {
        return $discordUser->getNickname()
            ?: $discordUser->getName()
            ?: 'Admin';
    }
}
