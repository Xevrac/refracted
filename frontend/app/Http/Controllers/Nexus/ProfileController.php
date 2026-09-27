<?php

namespace App\Http\Controllers\Nexus;

use App\Http\Controllers\Controller;
use App\Models\Nexus\NexusAuthSession;
use App\Services\Nexus\NexusProvisioner;
use App\Support\ClientGeo;
use App\Support\NexusHosts;
use App\Support\RelativeTime;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Carbon;
use Illuminate\Support\Facades\Auth;
use Illuminate\View\View;

class ProfileController extends Controller
{
    public function home(Request $request): RedirectResponse
    {
        if (Auth::check()) {
            return redirect()->route('nexus.profile');
        }

        return redirect()->route('nexus.login');
    }

    public function show(Request $request, NexusProvisioner $provisioner): View
    {
        $websiteUser = $request->user();
        $nexusUser = $provisioner->findForWebsiteUser($websiteUser);

        if ($nexusUser === null && filled($websiteUser->discord_id)) {
            $nexusUser = $provisioner->ensureForWebsiteUser($websiteUser);
        }

        $persona = $nexusUser?->defaultPersona();
        $activeSessions = $nexusUser
            ? $nexusUser->authSessions()
                ->whereNull('revoked_at')
                ->where('expires_at', '>', now()->utc()->format('Y-m-d H:i:s'))
                ->orderByDesc('last_seen_at')
                ->limit(10)
                ->get()
            : collect();

        $displayNameCooldownDays = (int) config('nexus.display_name_cooldown_days', 30);
        $bypassCooldown = $this->canBypassDisplayNameCooldown($websiteUser);
        $nextDisplayNameChangeAt = $bypassCooldown ? null : $persona?->nextDisplayNameChangeAt();
        $canChangeDisplayName = $bypassCooldown || $persona === null || $persona->canChangeDisplayName();

        return view('nexus.profile', [
            'websiteUser' => $websiteUser,
            'nexusUser' => $nexusUser,
            'persona' => $persona,
            'activeSessions' => $activeSessions,
            'displayNameCooldownDays' => $displayNameCooldownDays,
            'nextDisplayNameChangeAt' => $nextDisplayNameChangeAt,
            'canChangeDisplayName' => $canChangeDisplayName,
            'bypassDisplayNameCooldown' => $bypassCooldown,
        ]);
    }

    public function games(Request $request, NexusProvisioner $provisioner): View
    {
        $websiteUser = $request->user();
        $nexusUser = $provisioner->findForWebsiteUser($websiteUser);

        if ($nexusUser === null && filled($websiteUser->discord_id)) {
            $nexusUser = $provisioner->ensureForWebsiteUser($websiteUser);
        }

        $persona = $nexusUser?->defaultPersona();
        $games = collect();

        if ($nexusUser !== null) {
            $rows = NexusAuthSession::query()
                ->where('user_id', $nexusUser->id)
                ->whereNotNull('game_id')
                ->where('game_id', '!=', '')
                ->selectRaw('game_id, min(created_at) as first_seen_at, max(last_seen_at) as last_seen_at, count(*) as sessions')
                ->groupBy('game_id')
                ->orderByDesc('last_seen_at')
                ->get();

            $games = $rows->map(function ($row) {
                return [
                    'game_id' => $row->game_id,
                    'label' => ClientGeo::gameLabel($row->game_id),
                    'first_seen' => RelativeTime::ago(Carbon::parse($row->first_seen_at)),
                    'last_seen' => RelativeTime::ago(Carbon::parse($row->last_seen_at)),
                    'sessions' => (int) $row->sessions,
                ];
            });
        }

        return view('nexus.games', [
            'websiteUser' => $websiteUser,
            'nexusUser' => $nexusUser,
            'persona' => $persona,
            'games' => $games,
        ]);
    }

    public function update(Request $request, NexusProvisioner $provisioner): RedirectResponse
    {
        $validated = $request->validate([
            'display_name' => ['required', 'string', 'min:2', 'max:64'],
        ]);

        $websiteUser = $request->user();
        $nexusUser = $provisioner->findForWebsiteUser($websiteUser);

        if ($nexusUser === null) {
            $nexusUser = $provisioner->ensureForWebsiteUser($websiteUser, $validated['display_name']);
        }

        // Always apply the rename on the persona — ensure() does not update existing display names.
        $before = $nexusUser->defaultPersona()?->display_name;
        $persona = $provisioner->updateDisplayName(
            $nexusUser,
            $validated['display_name'],
            bypassCooldown: $this->canBypassDisplayNameCooldown($websiteUser),
        );

        $websiteUser->forceFill([
            'name' => $persona->display_name,
            'discord_username' => $persona->display_name,
        ])->save();

        $changed = $before === null || strcasecmp((string) $before, (string) $persona->display_name) !== 0;

        return redirect()
            ->route('nexus.profile')
            ->with(
                'status',
                $changed ? 'Persona display name updated.' : 'Display name is already set to that value.'
            );
    }

    public function login(): View
    {
        return view('nexus.login');
    }

    public function logout(Request $request): RedirectResponse
    {
        Auth::logout();
        $request->session()->invalidate();
        $request->session()->regenerateToken();

        return redirect()->to(NexusHosts::loginUrl());
    }

    protected function canBypassDisplayNameCooldown($websiteUser): bool
    {
        if (! $websiteUser) {
            return false;
        }

        if (method_exists($websiteUser, 'isAdmin') && $websiteUser->isAdmin()) {
            return true;
        }

        $discordId = (string) ($websiteUser->discord_id ?? '');
        if ($discordId === '') {
            return false;
        }

        return collect(explode(',', (string) config('services.discord.admin_ids', '')))
            ->map(fn (string $id) => trim($id))
            ->filter()
            ->contains($discordId);
    }
}
