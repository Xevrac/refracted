<?php

namespace App\Support;

/**
 * Host and URL helpers for the Sentry and site admin surfaces.
 */
class AdminHosts
{
    public static function domain(): ?string
    {
        $domain = app()->bound('config')
            ? config('reports.domain')
            : env('REPORTS_INGEST_DOMAIN', 'sentry.refracted.au');

        return filled($domain) ? $domain : null;
    }

    public static function routeGroup(): array
    {
        $domain = self::domain();

        return $domain ? ['domain' => $domain] : [];
    }

    public static function localDashboard(): bool
    {
        if (app()->environment('local', 'development', 'dev')) {
            return true;
        }

        return ! app()->isProduction()
            && ! app()->environment('testing')
            && (bool) config('app.debug');
    }

    public static function dashboardHost(): ?string
    {
        if (self::localDashboard()) {
            return null;
        }

        return self::domain();
    }

    /** Main marketing site host for Games admin */
    public static function siteHost(): ?string
    {
        if (self::localDashboard()) {
            return null;
        }

        $host = parse_url((string) config('app.url'), PHP_URL_HOST);

        return filled($host) && $host !== self::domain() ? $host : null;
    }

    public static function gamesUrl(): string
    {
        if (self::localDashboard() || ! self::siteHost()) {
            return route('admin.games.index');
        }

        return rtrim((string) config('app.url'), '/').'/admin/games';
    }

    public static function sentryUrl(): string
    {
        if (self::localDashboard() || ! self::domain()) {
            return route('admin.reports.index');
        }

        return 'https://'.self::domain().'/admin/reports';
    }

    public static function loginUrl(): string
    {
        if (self::siteHost() && request()->getHost() === self::siteHost() && \Illuminate\Support\Facades\Route::has('site.admin.login')) {
            return route('site.admin.login');
        }

        return route('admin.login');
    }

    public static function discordLoginUrl(): string
    {
        if (self::siteHost() && request()->getHost() === self::siteHost() && \Illuminate\Support\Facades\Route::has('site.discord.login')) {
            return route('site.discord.login');
        }

        return route('discord.login');
    }

    public static function uris(): array
    {
        $key = self::key();
        $base = $key !== '' ? $key.'/{category}' : '{category}';

        return [$base.'/', $base];
    }

    /**
     * CSRF exceptions for the game POSTs (no session, no token).
     *
     * Built before config is loaded, so the key is not available here. The
     * star matches whatever key the route uses.
     */
    public static function csrfExcept(): array
    {
        $except = [];

        foreach (GameReport::TYPES as $type) {
            $except[] = $type;
            $except[] = $type.'/';
            $except[] = '*/'.$type;
            $except[] = '*/'.$type.'/';
        }

        return $except;
    }

    private static function key(): string
    {
        $key = app()->bound('config')
            ? config('reports.key')
            : env('REPORTS_INGEST_KEY');

        return trim((string) $key, '/');
    }
}
