<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

/**
 * Ensures launcher GETs present the Refracted client headers the CDN expects.
 */
class VerifyCdnLauncherHeaders
{
    public function handle(Request $request, Closure $next): Response
    {
        if (! config('cdn.require_launcher_headers')) {
            return $next($request);
        }

        $client = strtolower(trim((string) $request->header('X-Refracted-Client', '')));
        $allowed = array_map('strtolower', config('cdn.allowed_clients', ['launcher']));

        if ($client === '' || ! in_array($client, $allowed, true)) {
            return response()->json([
                'error' => 'missing_or_invalid_client',
                'message' => 'Send X-Refracted-Client: launcher',
            ], 403);
        }

        $title = trim((string) $request->header('X-Refracted-Title', ''));
        if ($title === '') {
            return response()->json([
                'error' => 'missing_title',
                'message' => 'Send X-Refracted-Title (e.g. cnc)',
            ], 403);
        }

        $channel = strtolower(trim((string) $request->header('X-Refracted-Channel', '')));
        $channels = config('cdn.channels', ['release', 'debug']);
        if ($channel === '' || ! in_array($channel, $channels, true)) {
            return response()->json([
                'error' => 'missing_or_invalid_channel',
                'message' => 'Send X-Refracted-Channel: release|debug',
            ], 403);
        }

        $keys = config('cdn.client_keys', []);
        if (count($keys) > 0) {
            $provided = (string) $request->header('X-Refracted-Client-Key', '');
            if ($provided === '' || ! in_array($provided, $keys, true)) {
                return response()->json([
                    'error' => 'invalid_client_key',
                    'message' => 'Send a valid X-Refracted-Client-Key',
                ], 403);
            }
        }

        $launcherVersion = trim((string) $request->header('X-Refracted-Launcher-Version', ''));
        if ($launcherVersion === '') {
            return response()->json([
                'error' => 'missing_launcher_version',
                'message' => 'Send X-Refracted-Launcher-Version',
            ], 403);
        }

        return $next($request);
    }
}
