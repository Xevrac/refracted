<?php

namespace App\Services\Nexus;

use App\Models\Nexus\NexusAuthSession;
use App\Models\Nexus\NexusBan;
use App\Models\Nexus\NexusSetting;
use App\Models\Nexus\NexusSignupWhitelistEntry;
use App\Models\Nexus\NexusUser;
use App\Models\User;
use Illuminate\Auth\AuthenticationException;
use Illuminate\Support\Carbon;

/** Signup lock, Discord whitelist, and bans. */
class NexusAccessControl
{
    public const SETTING_REGISTRATIONS_OPEN = 'registrations_open';

    public const SETTING_WHITELIST_ENABLED = 'whitelist_enabled';

    public function registrationsOpen(): bool
    {
        return NexusSetting::getBool(self::SETTING_REGISTRATIONS_OPEN, true);
    }

    public function setRegistrationsOpen(bool $open): void
    {
        NexusSetting::setBool(self::SETTING_REGISTRATIONS_OPEN, $open);
    }

    public function whitelistEnabled(): bool
    {
        return NexusSetting::getBool(self::SETTING_WHITELIST_ENABLED, false);
    }

    public function setWhitelistEnabled(bool $enabled): void
    {
        NexusSetting::setBool(self::SETTING_WHITELIST_ENABLED, $enabled);
    }

    public function isDiscordWhitelisted(string $discordId): bool
    {
        $discordId = trim($discordId);
        if ($discordId === '') {
            return false;
        }

        return NexusSignupWhitelistEntry::query()
            ->where('discord_id', $discordId)
            ->exists();
    }

    public function addWhitelist(string $discordId, string $note, ?User $actor): NexusSignupWhitelistEntry
    {
        $discordId = trim($discordId);
        if ($discordId === '' || ! preg_match('/^\d{5,32}$/', $discordId)) {
            throw new \InvalidArgumentException('Discord user id must be a numeric snowflake.');
        }

        return NexusSignupWhitelistEntry::query()->updateOrCreate(
            ['discord_id' => $discordId],
            [
                'note' => mb_substr(trim($note), 0, 255),
                'created_by_web_user_id' => $actor?->id,
                'created_at' => now()->utc()->format('Y-m-d H:i:s'),
            ]
        );
    }

    public function removeWhitelist(string $discordId): void
    {
        NexusSignupWhitelistEntry::query()->where('discord_id', trim($discordId))->delete();
    }

    /**
     * Gate Nexus Discord sign-up / first provision.
     * Existing Nexus accounts may still sign in unless banned (checked separately).
     * Whitelist entries bypass a closed registration lock.
     */
    public function assertMayRegister(string $discordId, bool $alreadyProvisioned): void
    {
        if ($alreadyProvisioned) {
            return;
        }

        $whitelisted = $this->isDiscordWhitelisted($discordId);

        if (! $this->registrationsOpen() && ! $whitelisted) {
            throw new AuthenticationException(
                'Nexus registrations are currently closed.'
            );
        }

        if ($this->whitelistEnabled() && ! $whitelisted) {
            throw new AuthenticationException(
                'This Discord account is not on the Nexus playtest whitelist.'
            );
        }
    }

    public function activeBanForDiscord(string $discordId): ?NexusBan
    {
        $discordId = trim($discordId);
        if ($discordId === '') {
            return null;
        }

        return NexusBan::query()->active()->where('discord_id', $discordId)->latest('id')->first();
    }

    public function activeBanForNexusUser(int $userId): ?NexusBan
    {
        return NexusBan::query()->active()->where('user_id', $userId)->latest('id')->first();
    }

    public function assertNotBanned(?string $discordId, ?int $nexusUserId = null): void
    {
        $ban = null;
        if ($nexusUserId) {
            $ban = $this->activeBanForNexusUser($nexusUserId);
        }
        if ($ban === null && filled($discordId)) {
            $ban = $this->activeBanForDiscord((string) $discordId);
        }
        if ($ban === null) {
            return;
        }

        $suffix = $ban->isPermanent()
            ? 'permanently'
            : 'until '.$ban->banned_until?->utc()->toIso8601String();
        $reason = filled($ban->reason) ? ' Reason: '.$ban->reason : '';

        throw new AuthenticationException("This Nexus account is banned {$suffix}.{$reason}");
    }

    public function ban(
        ?NexusUser $nexusUser,
        ?string $discordId,
        string $reason,
        ?Carbon $until,
        User $actor,
    ): NexusBan {
        $discordId = trim((string) ($discordId ?: $nexusUser?->discord_id ?: ''));
        if ($nexusUser === null && $discordId === '') {
            throw new \InvalidArgumentException('Ban requires a Nexus user or Discord id.');
        }

        $ban = NexusBan::query()->create([
            'user_id' => $nexusUser?->id,
            'discord_id' => $discordId !== '' ? $discordId : null,
            'reason' => mb_substr(trim($reason), 0, 512),
            'banned_until' => $until?->utc()->format('Y-m-d H:i:s'),
            'created_by_web_user_id' => $actor->id,
            'created_at' => now()->utc()->format('Y-m-d H:i:s'),
        ]);

        if ($nexusUser !== null) {
            $this->revokeAllSessions((int) $nexusUser->id);
        }

        return $ban;
    }

    public function lift(NexusBan $ban, User $actor): void
    {
        if ($ban->lifted_at !== null) {
            return;
        }

        $ban->forceFill([
            'lifted_at' => now()->utc()->format('Y-m-d H:i:s'),
            'lifted_by_web_user_id' => $actor->id,
        ])->save();
    }

    public function revokeAllSessions(int $nexusUserId, string $reason = 'ban'): void
    {
        NexusAuthSession::query()
            ->where('user_id', $nexusUserId)
            ->whereNull('revoked_at')
            ->update(['revoked_at' => now()->utc()->format('Y-m-d H:i:s'), 'revoked_reason' => $reason]);
    }
}
