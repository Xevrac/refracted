<?php

namespace App\Support;

/** Nexus host helpers. */
class NexusHosts
{
    public static function domain(): ?string
    {
        $domain = app()->bound('config')
            ? config('nexus.domain')
            : env('NEXUS_DOMAIN', 'nexus.refracted.au');

        return filled($domain) ? $domain : null;
    }

    public static function localDashboard(): bool
    {
        return AdminHosts::localDashboard();
    }

    /** @return array{domain?: string, prefix?: string} */
    public static function routeGroup(): array
    {
        if (! self::localDashboard() && self::domain()) {
            return ['domain' => self::domain()];
        }

        return ['prefix' => 'nexus'];
    }

    public static function isNexusHost(?string $host = null): bool
    {
        $domain = self::domain();
        if (! $domain) {
            return false;
        }

        $host ??= request()->getHost();

        return strcasecmp((string) $host, $domain) === 0;
    }

    public static function homeUrl(): string
    {
        if (self::localDashboard() || ! self::domain()) {
            return route('nexus.home');
        }

        return 'https://'.self::domain().'/';
    }

    public static function loginUrl(): string
    {
        if (self::localDashboard() || ! self::domain()) {
            return route('nexus.login');
        }

        return 'https://'.self::domain().'/login';
    }

    public static function discordLoginUrl(): string
    {
        if (self::localDashboard() || ! self::domain()) {
            return route('nexus.discord.login');
        }

        return 'https://'.self::domain().'/auth/discord';
    }

    public static function profileUrl(): string
    {
        if (self::localDashboard() || ! self::domain()) {
            return route('nexus.profile');
        }

        return 'https://'.self::domain().'/profile';
    }

    public static function gamesUrl(): string
    {
        if (self::localDashboard() || ! self::domain()) {
            return route('nexus.profile.games');
        }

        return 'https://'.self::domain().'/profile/games';
    }

    public static function deviceUrl(?string $userCode = null): string
    {
        if (self::localDashboard() || ! self::domain()) {
            return route('nexus.device', array_filter(['code' => $userCode]));
        }

        $url = 'https://'.self::domain().'/device';

        return filled($userCode) ? $url.'?code='.urlencode($userCode) : $url;
    }
}
