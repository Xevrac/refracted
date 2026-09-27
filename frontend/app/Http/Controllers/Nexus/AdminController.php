<?php

namespace App\Http\Controllers\Nexus;

use App\Http\Controllers\Controller;
use App\Models\Nexus\NexusAuthSession;
use App\Models\Nexus\NexusBan;
use App\Models\Nexus\NexusPersona;
use App\Models\Nexus\NexusSignupWhitelistEntry;
use App\Models\Nexus\NexusUser;
use App\Services\Nexus\NexusAccessControl;
use App\Support\ClientGeo;
use App\Support\RelativeTime;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Carbon;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\DB;
use Illuminate\View\View;

/** Nexus access-control admin UI. */
class AdminController extends Controller
{
    public function index(NexusAccessControl $access): View
    {
        return view('nexus.admin.index', [
            'activeBans' => NexusBan::query()->active()->orderByDesc('id')->limit(100)->get(),
            'users' => $this->recentUsers(),
        ]);
    }

    public function lookup(Request $request, NexusAccessControl $access): View
    {
        $query = trim((string) $request->query('q', ''));

        return view('nexus.admin.lookup', [
            'query' => $query,
            'stats' => $this->lookupStats($access),
            'lookupResults' => $query !== '' ? $this->lookupPlayers($query, $access) : collect(),
        ]);
    }

    public function gatekeeper(NexusAccessControl $access): View
    {
        return view('nexus.admin.gatekeeper', [
            'registrationsOpen' => $access->registrationsOpen(),
            'whitelistEnabled' => $access->whitelistEnabled(),
            'whitelist' => NexusSignupWhitelistEntry::query()->orderByDesc('id')->limit(200)->get(),
        ]);
    }

    public function sessions(Request $request, NexusAccessControl $access): View
    {
        $filters = $request->validate([
            'q' => ['nullable', 'string', 'max:64'],
            'state' => ['nullable', 'in:all,active,revoked,expired'],
            'game' => ['nullable', 'string', 'max:64'],
        ]);
        $query = trim((string) ($filters['q'] ?? ''));
        $state = $filters['state'] ?? 'all';
        $game = trim((string) ($filters['game'] ?? ''));
        $now = now()->utc()->format('Y-m-d H:i:s');

        $sessions = NexusAuthSession::query()
            ->with(['user', 'persona'])
            ->when($state === 'active', fn ($q) => $q->whereNull('revoked_at')->where('expires_at', '>', $now))
            ->when($state === 'revoked', fn ($q) => $q->whereNotNull('revoked_at'))
            ->when($state === 'expired', fn ($q) => $q->whereNull('revoked_at')->where('expires_at', '<=', $now))
            ->when($game !== '', fn ($q) => $q->where('game_id', $game))
            ->when($query !== '', function ($q) use ($query) {
                $like = '%'.str_replace(['%', '_'], ['\\%', '\\_'], $query).'%';
                $q->where(function ($w) use ($query, $like) {
                    if (ctype_digit($query)) {
                        $w->orWhere('user_id', (int) $query)->orWhere('persona_id', (int) $query);
                    }
                    $w->orWhere('jwt_id', 'like', $query.'%')
                        ->orWhere('client_ip', 'like', $like)
                        ->orWhereHas('user', fn ($u) => $u->where('username', 'like', $like)->orWhere('discord_id', 'like', $like))
                        ->orWhereHas('persona', fn ($p) => $p->where('display_name', 'like', $like));
                });
            })
            ->orderByDesc('id')
            ->paginate(50)
            ->withQueryString();

        $bannedUsers = collect($sessions->items())
            ->pluck('user')
            ->filter()
            ->unique('id')
            ->filter(fn (NexusUser $u) => $access->activeBanForNexusUser((int) $u->id) !== null
                || (filled($u->discord_id) && $access->activeBanForDiscord((string) $u->discord_id) !== null))
            ->pluck('id')
            ->all();

        return view('nexus.admin.sessions', [
            'query' => $query,
            'state' => $state,
            'game' => $game,
            'games' => config('nexus.games', []),
            'sessions' => $sessions,
            'bannedUsers' => $bannedUsers,
            'stats' => $this->sessionStats(),
        ]);
    }

    public function revokeSession(Request $request, int $session): RedirectResponse
    {
        $row = NexusAuthSession::query()->findOrFail($session);
        if ($row->revoked_at === null) {
            $row->forceFill([
                'revoked_at' => now()->utc()->format('Y-m-d H:i:s'),
                'revoked_reason' => 'admin',
            ])->save();
        }

        return back()->with('status', 'Session revoked.');
    }

    public function updateRegistrations(Request $request, NexusAccessControl $access): RedirectResponse
    {
        $validated = $request->validate([
            'registrations_open' => ['required', 'in:0,1'],
            'confirm' => ['required', 'accepted'],
        ]);

        $open = $validated['registrations_open'] === '1';
        $access->setRegistrationsOpen($open);

        return redirect()
            ->route('nexus.admin.gatekeeper')
            ->with('status', $open
                ? 'Registrations are open.'
                : 'Registrations are closed. New Discord accounts cannot join.');
    }

    public function updateWhitelistMode(Request $request, NexusAccessControl $access): RedirectResponse
    {
        $validated = $request->validate([
            'whitelist_enabled' => ['required', 'in:0,1'],
        ]);

        $enabled = $validated['whitelist_enabled'] === '1';
        $access->setWhitelistEnabled($enabled);

        return redirect()
            ->route('nexus.admin.gatekeeper')
            ->with('status', $enabled
                ? 'Discord whitelist enforced for new sign-ups.'
                : 'Discord whitelist disabled — any allowed registration may join.');
    }

    public function storeWhitelist(Request $request, NexusAccessControl $access): RedirectResponse
    {
        $validated = $request->validate([
            'discord_id' => ['required', 'string', 'max:32'],
            'note' => ['nullable', 'string', 'max:255'],
        ]);

        try {
            $access->addWhitelist(
                $validated['discord_id'],
                $validated['note'] ?? '',
                $request->user()
            );
        } catch (\InvalidArgumentException $e) {
            return back()->withErrors(['discord_id' => $e->getMessage()]);
        }

        return redirect()->route('nexus.admin.gatekeeper')->with('status', 'Whitelist entry saved.');
    }

    public function destroyWhitelist(Request $request, NexusAccessControl $access): RedirectResponse
    {
        $validated = $request->validate([
            'discord_id' => ['required', 'string', 'max:32'],
        ]);
        $access->removeWhitelist($validated['discord_id']);

        return redirect()->route('nexus.admin.gatekeeper')->with('status', 'Whitelist entry removed.');
    }

    public function storeBan(Request $request, NexusAccessControl $access): RedirectResponse
    {
        $validated = $request->validate([
            'user_id' => ['nullable', 'integer'],
            'discord_id' => ['nullable', 'string', 'max:32'],
            'reason' => ['nullable', 'string', 'max:512'],
            'duration' => ['required', 'in:permanent,1h,24h,7d,30d'],
            'return_q' => ['nullable', 'string', 'max:64'],
        ]);

        $nexusUser = null;
        if (! empty($validated['user_id'])) {
            $nexusUser = NexusUser::query()->find($validated['user_id']);
            if ($nexusUser === null) {
                return back()->withErrors(['user_id' => 'Unknown Nexus user id.']);
            }
        }

        $until = match ($validated['duration']) {
            '1h' => now()->utc()->addHour(),
            '24h' => now()->utc()->addDay(),
            '7d' => now()->utc()->addDays(7),
            '30d' => now()->utc()->addDays(30),
            default => null,
        };

        try {
            $access->ban(
                $nexusUser,
                $validated['discord_id'] ?? null,
                $validated['reason'] ?? '',
                $until,
                $request->user()
            );
        } catch (\InvalidArgumentException $e) {
            return back()->withErrors(['user_id' => $e->getMessage()]);
        }

        return $this->redirectAdmin($validated['return_q'] ?? null, 'Ban applied. Active sessions revoked.');
    }

    public function liftBan(Request $request, NexusAccessControl $access, int $ban): RedirectResponse
    {
        $validated = $request->validate([
            'return_q' => ['nullable', 'string', 'max:64'],
        ]);

        $row = NexusBan::query()->findOrFail($ban);
        $access->lift($row, $request->user());

        return $this->redirectAdmin($validated['return_q'] ?? null, 'Ban lifted.');
    }

    /** @return Collection<int, array{user: NexusUser, persona: ?NexusPersona, ban: ?NexusBan, last_seen: string, last_game: string, country: string, client_ip: ?string}> */
    protected function lookupPlayers(string $query, NexusAccessControl $access): Collection
    {
        $like = '%'.str_replace(['%', '_'], ['\\%', '\\_'], $query).'%';

        $users = NexusUser::query()
            ->with('personas')
            ->where(function ($builder) use ($query, $like) {
                if (ctype_digit($query)) {
                    $builder->orWhere('id', (int) $query);
                }
                $builder
                    ->orWhere('username', 'like', $like)
                    ->orWhere('discord_id', 'like', $like)
                    ->orWhere('email', 'like', $like)
                    ->orWhereHas('personas', function ($personas) use ($like) {
                        $personas->where('display_name', 'like', $like);
                    });
            })
            ->orderByDesc('id')
            ->limit(25)
            ->get();

        $latestSessions = $this->latestSessionsForUsers($users->pluck('id')->all());

        return $users->map(function (NexusUser $user) use ($access, $latestSessions) {
            $session = $latestSessions->get($user->id);
            $lastSeenAt = $session?->last_seen_at ?? $user->created_at;

            return [
                'user' => $user,
                'persona' => $user->defaultPersona(),
                'ban' => $access->activeBanForNexusUser((int) $user->id)
                    ?? ($user->discord_id ? $access->activeBanForDiscord((string) $user->discord_id) : null),
                'last_seen' => RelativeTime::ago($lastSeenAt),
                'last_game' => ClientGeo::gameLabel($session?->game_id),
                'country' => ClientGeo::countryLabel($session?->country_code),
                'client_ip' => $session?->client_ip,
            ];
        });
    }

    /** @param  list<int|string>  $userIds */
    protected function latestSessionsForUsers(array $userIds): Collection
    {
        if ($userIds === []) {
            return collect();
        }

        $connection = config('nexus.connection', 'nexus');
        $latestIds = DB::connection($connection)
            ->table('auth_sessions')
            ->selectRaw('max(id) as id')
            ->whereIn('user_id', $userIds)
            ->groupBy('user_id')
            ->pluck('id');

        if ($latestIds->isEmpty()) {
            return collect();
        }

        return NexusAuthSession::query()
            ->whereIn('id', $latestIds)
            ->get()
            ->keyBy('user_id');
    }

    /** @return array{users: int, active_sessions: int, sessions_24h: int, active_bans: int, registrations_open: bool, whitelist_enabled: bool} */
    protected function lookupStats(NexusAccessControl $access): array
    {
        $now = now()->utc()->format('Y-m-d H:i:s');
        $dayAgo = now()->utc()->subDay()->format('Y-m-d H:i:s');

        return [
            'users' => NexusUser::query()->count(),
            'active_sessions' => NexusAuthSession::query()
                ->whereNull('revoked_at')
                ->where('expires_at', '>', $now)
                ->count(),
            'sessions_24h' => NexusAuthSession::query()
                ->where('created_at', '>=', $dayAgo)
                ->count(),
            'active_bans' => NexusBan::query()->active()->count(),
            'registrations_open' => $access->registrationsOpen(),
            'whitelist_enabled' => $access->whitelistEnabled(),
        ];
    }

    /** @return array{active: int, issued_24h: int, revoked_24h: int, expired: int, avg_lifetime: string} */
    protected function sessionStats(): array
    {
        $now = now()->utc()->format('Y-m-d H:i:s');
        $dayAgo = now()->utc()->subDay()->format('Y-m-d H:i:s');
        $avgSecs = (int) NexusAuthSession::query()
            ->whereNotNull('revoked_at')
            ->where('created_at', '>=', now()->utc()->subDays(30)->format('Y-m-d H:i:s'))
            ->selectRaw('avg(timestampdiff(second, created_at, revoked_at)) as secs')
            ->value('secs');

        return [
            'active' => NexusAuthSession::query()->whereNull('revoked_at')->where('expires_at', '>', $now)->count(),
            'issued_24h' => NexusAuthSession::query()->where('created_at', '>=', $dayAgo)->count(),
            'revoked_24h' => NexusAuthSession::query()->where('revoked_at', '>=', $dayAgo)->count(),
            'expired' => NexusAuthSession::query()->whereNull('revoked_at')->where('expires_at', '<=', $now)->count(),
            'avg_lifetime' => $avgSecs > 0 ? RelativeTime::duration($avgSecs) : '—',
        ];
    }

    /**
     * Most recently seen first (latest session activity, else account creation).
     *
     * @return Collection<int, array{user: NexusUser, last_seen: string}>
     */
    protected function recentUsers(): Collection
    {
        $lastSeen = NexusAuthSession::query()
            ->selectRaw('user_id, max(last_seen_at) as last_seen_at')
            ->groupBy('user_id');

        $users = NexusUser::query()
            ->select('users.*', 'seen.last_seen_at as session_last_seen_at')
            ->leftJoinSub($lastSeen, 'seen', 'seen.user_id', '=', 'users.id')
            ->orderByRaw('coalesce(seen.last_seen_at, users.created_at) desc')
            ->limit(100)
            ->get();

        return $users->map(function (NexusUser $user) {
            $raw = $user->getAttribute('session_last_seen_at');

            return [
                'user' => $user,
                'last_seen' => RelativeTime::ago($raw !== null ? Carbon::parse($raw) : $user->created_at),
            ];
        });
    }

    protected function redirectAdmin(?string $returnQ, string $status): RedirectResponse
    {
        $returnQ = trim((string) $returnQ);

        if ($returnQ !== '') {
            return redirect()
                ->route('nexus.admin.lookup', ['q' => $returnQ])
                ->with('status', $status);
        }

        return redirect()->route('nexus.admin')->with('status', $status);
    }
}
