@extends('nexus.admin.layout', ['title' => 'Lookup', 'active' => 'lookup'])

@section('content')
    <div>
        <h1 class="font-display text-3xl font-semibold tracking-[-0.03em]">Player lookup</h1>
        <p class="mt-2 max-w-xl text-sm text-grit-mist">
            Search by Nexus user id, username, Discord id, or persona display name.
        </p>
    </div>

    <section class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        <div class="border border-grit-line bg-grit-surface/80 px-4 py-4">
            <p class="text-[11px] uppercase tracking-[0.14em] text-grit-mist">Players</p>
            <p class="mt-2 font-display text-2xl font-semibold tabular-nums tracking-[-0.02em]">{{ number_format($stats['users']) }}</p>
        </div>
        <div class="border border-grit-line bg-grit-surface/80 px-4 py-4">
            <p class="text-[11px] uppercase tracking-[0.14em] text-grit-mist">Active sessions</p>
            <p class="mt-2 font-display text-2xl font-semibold tabular-nums tracking-[-0.02em]">{{ number_format($stats['active_sessions']) }}</p>
        </div>
        <div class="border border-grit-line bg-grit-surface/80 px-4 py-4">
            <p class="text-[11px] uppercase tracking-[0.14em] text-grit-mist">Tickets · 24h</p>
            <p class="mt-2 font-display text-2xl font-semibold tabular-nums tracking-[-0.02em]">{{ number_format($stats['sessions_24h']) }}</p>
        </div>
        <div class="border border-grit-line bg-grit-surface/80 px-4 py-4">
            <p class="text-[11px] uppercase tracking-[0.14em] text-grit-mist">Active bans</p>
            <p class="mt-2 font-display text-2xl font-semibold tabular-nums tracking-[-0.02em]">{{ number_format($stats['active_bans']) }}</p>
        </div>
        <div class="border border-grit-line bg-grit-surface/80 px-4 py-4">
            <p class="text-[11px] uppercase tracking-[0.14em] text-grit-mist">Registrations</p>
            <p class="mt-2 font-display text-xl font-semibold tracking-[-0.02em]">{{ $stats['registrations_open'] ? 'Open' : 'Closed' }}</p>
        </div>
        <div class="border border-grit-line bg-grit-surface/80 px-4 py-4">
            <p class="text-[11px] uppercase tracking-[0.14em] text-grit-mist">Whitelist</p>
            <p class="mt-2 font-display text-xl font-semibold tracking-[-0.02em]">{{ $stats['whitelist_enabled'] ? 'Enforced' : 'Off' }}</p>
        </div>
    </section>

    <section class="border border-grit-line bg-grit-surface/80 px-5 py-5">
        <form method="GET" action="{{ route('nexus.admin.lookup') }}" class="flex flex-col gap-3 sm:flex-row sm:items-stretch">
            <label class="sr-only" for="q">Search</label>
            <input
                id="q"
                name="q"
                type="search"
                value="{{ $query }}"
                required
                minlength="1"
                maxlength="64"
                placeholder="id · username · discord · display name"
                class="h-11 min-w-0 flex-1 border border-grit-line bg-grit-bg px-3 text-sm outline-none focus:border-signal"
                autofocus
            >
            <button
                type="submit"
                class="inline-flex h-11 shrink-0 items-center justify-center bg-signal px-6 font-display text-sm font-semibold text-white hover:bg-signal-soft"
            >
                Search
            </button>
        </form>

        @if ($query !== '')
            <ul class="mt-6 divide-y divide-grit-line/60 text-sm">
                @forelse ($lookupResults as $row)
                    @php
                        $u = $row['user'];
                        $persona = $row['persona'];
                        $ban = $row['ban'];
                    @endphp
                    <li class="space-y-3 py-4">
                        <div class="flex flex-wrap items-baseline justify-between gap-2">
                            <div>
                                <p class="font-display text-base tracking-[-0.02em]">
                                    {{ $persona?->display_name ?? $u->username }}
                                    <span class="text-grit-mist">· user</span>
                                    <x-reveal :value="$u->id" label="id" class="ms-1" />
                                </p>
                                <p class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-grit-mist">
                                    <span>username {{ $u->username }}</span>
                                    <span>·</span>
                                    <span class="inline-flex items-center gap-1">discord <x-reveal :value="$u->discord_id" /></span>
                                    <span>·</span>
                                    <span>persona {{ $persona?->id ?? '—' }}</span>
                                    @if (filled($u->email))
                                        <span>·</span>
                                        <span class="inline-flex items-center gap-1">email <x-reveal :value="$u->email" /></span>
                                    @endif
                                </p>
                                <p class="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-grit-mist">
                                    <span>last seen {{ $row['last_seen'] }}</span>
                                    <span>·</span>
                                    <span>last game {{ $row['last_game'] }}</span>
                                    <span>·</span>
                                    <span>country {{ $row['country'] }}</span>
                                    @if (! empty($row['client_ip']))
                                        <span>·</span>
                                        <span class="inline-flex items-center gap-1">ip <x-reveal :value="$row['client_ip']" /></span>
                                    @endif
                                </p>
                            </div>
                            @if ($ban)
                                <span class="text-xs uppercase tracking-[0.14em] text-red-300">Banned</span>
                            @else
                                <span class="text-xs uppercase tracking-[0.14em] text-grit-mist">Active</span>
                            @endif
                        </div>

                        @if ($ban)
                            <p class="text-xs text-grit-mist">
                                {{ $ban->isPermanent() ? 'Permanent' : 'Until '.$ban->banned_until?->utc()->toDateTimeString().' UTC' }}
                                @if ($ban->reason) — {{ $ban->reason }} @endif
                            </p>
                            <form method="POST" action="{{ route('nexus.admin.bans.lift', $ban) }}">
                                @csrf
                                <input type="hidden" name="return_q" value="{{ $query }}">
                                <button type="submit" class="text-xs text-signal-soft underline underline-offset-2">Unban</button>
                            </form>
                        @else
                            <form method="POST" action="{{ route('nexus.admin.bans.store') }}" class="grid gap-2 sm:grid-cols-[1fr_auto_auto]">
                                @csrf
                                <input type="hidden" name="user_id" value="{{ $u->id }}">
                                <input type="hidden" name="discord_id" value="{{ $u->discord_id }}">
                                <input type="hidden" name="return_q" value="{{ $query }}">
                                <input name="reason" placeholder="Reason" class="border border-grit-line bg-grit-bg px-3 py-2 text-sm outline-none focus:border-signal">
                                <select name="duration" class="border border-grit-line bg-grit-bg px-3 py-2 text-sm outline-none focus:border-signal">
                                    <option value="permanent">Permanent</option>
                                    <option value="1h">1 hour</option>
                                    <option value="24h">24 hours</option>
                                    <option value="7d">7 days</option>
                                    <option value="30d">30 days</option>
                                </select>
                                <button type="submit" class="bg-red-600/90 px-4 py-2 font-display text-sm font-semibold text-white hover:bg-red-500">Ban</button>
                            </form>
                        @endif
                    </li>
                @empty
                    <li class="py-4 text-grit-mist">No players matched “{{ $query }}”.</li>
                @endforelse
            </ul>
        @endif
    </section>
@endsection
