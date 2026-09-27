<?php

namespace App\Support;

/** CDN host helpers (cdn.refracted.au). */
class CdnHosts
{
    public static function domain(): ?string
    {
        $domain = app()->bound('config')
            ? config('cdn.domain')
            : env('CDN_DOMAIN', 'cdn.refracted.au');

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

        return ['prefix' => 'cdn'];
    }

    public static function isCdnHost(?string $host = null): bool
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
            return url('/cdn');
        }

        return 'https://'.self::domain().'/';
    }

    public static function adminUrl(): string
    {
        if (self::localDashboard() || ! self::domain()) {
            return route('cdn.admin.dashboard');
        }

        return 'https://'.self::domain().'/admin';
    }

    public static function loginUrl(): string
    {
        if (self::localDashboard() || ! self::domain()) {
            return route('cdn.admin.login');
        }

        return 'https://'.self::domain().'/admin/login';
    }

    public static function discordLoginUrl(): string
    {
        if (self::localDashboard() || ! self::domain()) {
            return route('cdn.discord.login');
        }

        return 'https://'.self::domain().'/auth/discord';
    }
}
