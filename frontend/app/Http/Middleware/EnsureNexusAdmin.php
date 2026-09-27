<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

/** Nexus control panel — website admins or Discord admin allowlist. */
class EnsureNexusAdmin
{
    public function handle(Request $request, Closure $next): Response
    {
        $user = $request->user();
        if (! $user) {
            abort(403, 'Forbidden');
        }

        if ($user->isAdmin()) {
            return $next($request);
        }

        $discordId = (string) ($user->discord_id ?? '');
        $allowed = collect(explode(',', (string) config('services.discord.admin_ids', '')))
            ->map(fn (string $id) => trim($id))
            ->filter()
            ->contains($discordId);

        if (! $allowed) {
            abort(403, 'Nexus admin access required.');
        }

        return $next($request);
    }
}
