<?php

namespace App\Support;

/**
 * Host and path helpers for the Prism report collector.
 */
class ReportIngest
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
