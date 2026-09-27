<?php

namespace App\Support;

use Illuminate\Http\Request;

/** Client IP + country estimate for Nexus session telemetry. */
class ClientGeo
{
    public static function ip(?Request $request = null): ?string
    {
        $request ??= request();
        $ip = $request->ip();

        return filled($ip) ? (string) $ip : null;
    }

    /** ISO 3166-1 alpha-2 from edge headers when present. */
    public static function countryCode(?Request $request = null): ?string
    {
        $request ??= request();

        foreach (['CF-IPCountry', 'CloudFront-Viewer-Country', 'X-Country-Code'] as $header) {
            $raw = strtoupper(trim((string) $request->headers->get($header, '')));
            if (preg_match('/^[A-Z]{2}$/', $raw) === 1 && ! in_array($raw, ['XX', 'T1'], true)) {
                return $raw;
            }
        }

        return null;
    }

    public static function countryLabel(?string $code): string
    {
        $code = strtoupper(trim((string) $code));
        if ($code === '' || in_array($code, ['XX', 'T1'], true)) {
            return '—';
        }

        $name = null;
        if (class_exists(\Locale::class)) {
            $resolved = \Locale::getDisplayRegion('-'.$code, 'en');
            if (is_string($resolved) && $resolved !== '' && strcasecmp($resolved, $code) !== 0) {
                $name = $resolved;
            }
        }

        return $name ? "{$name} ({$code})" : $code;
    }

    public static function gameLabel(?string $gameId): string
    {
        $gameId = strtolower(trim((string) $gameId));
        if ($gameId === '') {
            return '—';
        }

        $labels = (array) config('nexus.games', []);

        return (string) ($labels[$gameId] ?? strtoupper($gameId));
    }
}
