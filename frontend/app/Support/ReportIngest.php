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

    public static function uris(): array
    {
        $key = self::key();
        $base = $key !== '' ? $key.'/{category}' : '{category}';

        return [$base.'/', $base];
    }

    /**
     * CSRF exceptions for the game POSTs (no session, no token).
     */
    public static function csrfExcept(): array
    {
        $key = self::key();
        $except = [];

        foreach (GameReport::TYPES as $type) {
            $path = $key !== '' ? $key.'/'.$type : $type;
            $except[] = $path;
            $except[] = $path.'/';
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
