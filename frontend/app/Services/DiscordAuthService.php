<?php

namespace App\Services;

use App\Models\Nexus\NexusUser;
use App\Models\User;
use App\Services\Nexus\NexusAccessControl;
use App\Services\Nexus\NexusProvisioner;
use Illuminate\Auth\AuthenticationException;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Str;
use Laravel\Socialite\Contracts\User as SocialiteUser;

class DiscordAuthService
{
    public const INTENT_ADMIN = 'admin';

    public const INTENT_NEXUS = 'nexus';

    public function __construct(
        protected NexusProvisioner $nexusProvisioner,
        protected NexusAccessControl $nexusAccess,
    ) {}

    public function resolveForLogin(SocialiteUser $discordUser, string $intent): User
    {
        if ($intent !== self::INTENT_ADMIN && $intent !== self::INTENT_NEXUS) {
            throw new AuthenticationException('Unknown sign-in intent.');
        }

        $discordId = (string) $discordUser->getId();
        $allowedAdmin = $this->isAllowedAdmin($discordId);
        $displayName = $this->resolveDisplayName($discordUser, $intent);

        if ($intent === self::INTENT_ADMIN && ! $allowedAdmin) {
            throw new AuthenticationException('This Discord account is not allowed to administer Refracted.');
        }

        if ($intent === self::INTENT_NEXUS) {
            $this->nexusAccess->assertNotBanned($discordId);
            $already = NexusUser::query()->where('discord_id', $discordId)->exists();
            $this->nexusAccess->assertMayRegister($discordId, $already);
        }

        $user = User::query()->updateOrCreate(
            ['discord_id' => $discordId],
            [
                'name' => $displayName,
                'email' => $discordUser->getEmail() ?: "{$discordId}@discord.local",
                'password' => Hash::make(Str::random(40)),
                'email_verified_at' => now(),
                'discord_username' => $displayName,
                'discord_avatar' => $discordUser->getAvatar(),
                'last_login_at' => now(),
            ],
        );

        // Admin elevation only on the admin sign-in path.
        if ($intent === self::INTENT_ADMIN && $allowedAdmin && ! $user->is_admin) {
            $user->forceFill(['is_admin' => true])->save();
        }

        if ($intent === self::INTENT_NEXUS) {
            $this->nexusProvisioner->ensureForWebsiteUser($user, $displayName);
        }

        return $user->fresh();
    }

    protected function isAllowedAdmin(string $discordId): bool
    {
        $allowed = collect(explode(',', (string) config('services.discord.admin_ids', '')))
            ->map(fn (string $id) => trim($id))
            ->filter()
            ->values();

        return $allowed->contains($discordId);
    }

    protected function resolveDisplayName(SocialiteUser $discordUser, string $intent): string
    {
        $name = $discordUser->getNickname()
            ?: $discordUser->getName()
            ?: ($intent === self::INTENT_NEXUS ? 'Player' : 'Admin');

        return Str::limit(trim($name), 64, '');
    }
}
