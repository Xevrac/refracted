@extends('nexus.admin.layout', ['title' => 'Sessions', 'active' => 'sessions'])

@section('content')
    <div>
        <h1 class="font-display text-3xl font-semibold tracking-[-0.03em]">Sessions</h1>
        <p class="mt-2 max-w-xl text-sm text-grit-mist">
            Launcher tickets issued by Nexus: state, lifetime and where they were used. Times are UTC.
        </p>
    </div>

    <section class="grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
        @foreach ([
            'Active' => number_format($stats['active']),
            'Issued · 24h' => number_format($stats['issued_24h']),
            'Revoked · 24h' => number_format($stats['revoked_24h']),
            'Expired' => number_format($stats['expired']),
            'Avg lifetime · 30d' => $stats['avg_lifetime'],
        ] as $label => $value)
            <div class="border border-grit-line bg-grit-surface/80 px-4 py-4">
                <p class="text-[11px] uppercase tracking-[0.14em] text-grit-mist">{{ $label }}</p>
                <p class="mt-2 font-display text-2xl font-semibold tabular-nums tracking-[-0.02em]">{{ $value }}</p>
            </div>
        @endforeach
    </section>

    <section class="border border-grit-line bg-grit-surface/80 px-5 py-5">
        <form method="GET" action="{{ route('nexus.admin.sessions') }}" class="grid gap-3 sm:grid-cols-[1fr_auto_auto_auto]">
            <label class="sr-only" for="q">Search</label>
            <input
                id="q"
                name="q"
                type="search"
                value="{{ $query }}"
                maxlength="64"
                placeholder="user id · persona · username · discord · jti · ip"
                class="h-11 min-w-0 border border-grit-line bg-grit-bg px-3 text-sm outline-none focus:border-signal"
            >
            <select name="state" class="h-11 border border-grit-line bg-grit-bg px-3 text-sm outline-none focus:border-signal">
                @foreach (['all' => 'All states', 'active' => 'Active', 'revoked' => 'Revoked', 'expired' => 'Expired'] as $value => $label)
                    <option value="{{ $value }}" @selected($state === $value)>{{ $label }}</option>
                @endforeach
            </select>
            <select name="game" class="h-11 border border-grit-line bg-grit-bg px-3 text-sm outline-none focus:border-signal">
                <option value="">All games</option>
                @foreach ($games as $id => $label)
                    <option value="{{ $id }}" @selected($game === $id)>{{ $label }}</option>
                @endforeach
            </select>
            <button
                type="submit"
                class="inline-flex h-11 items-center justify-center bg-signal px-6 font-display text-sm font-semibold text-white hover:bg-signal-soft"
            >
                Filter
            </button>
        </form>

        <ul class="mt-6 divide-y divide-grit-line/60 text-sm">
            @forelse ($sessions as $s)
                @php
                    $sessionState = $s->state();
                    $banned = in_array((int) $s->user_id, array_map("intval", $bannedUsers), true);
                    $now = now()->utc();
                    $end = $s->revoked_at ?? ($sessionState === 'expired' ? $s->expires_at : $now);
                    $lifetime = $s->created_at ? \App\Support\RelativeTime::duration((int) $s->created_at->diffInSeconds($end)) : '—';
                    $badge = match ($sessionState) {
                        'active' => ['Active', 'text-emerald-300'],
                        'revoked' => ['Revoked'.($s->revoked_reason ? ' · '.$s->revoked_reason : ''), 'text-red-300'],
                        default => ['Expired', 'text-grit-mist'],
                    };
                @endphp
                <li class="space-y-3 py-4">
                    <div class="flex flex-wrap items-baseline justify-between gap-2">
                        <p class="font-display text-base tracking-[-0.02em]">
                            {{ $s->persona?->display_name ?? $s->user?->username ?? 'Unknown' }}
                            <span class="text-grit-mist">· user</span>
                            <x-reveal :value="$s->user_id" label="id" class="ms-1" />
                            <span class="text-grit-mist">· persona</span>
                            <x-reveal :value="$s->persona_id" label="id" class="ms-1" />
                        </p>
                        <p class="flex items-center gap-3 text-xs uppercase tracking-[0.14em]">
                            @if ($banned)
                                <span class="text-red-300">User banned</span>
                            @endif
                            <span class="{{ $badge[1] }}">{{ $badge[0] }}</span>
                        </p>
                    </div>

                    <dl class="grid gap-x-6 gap-y-2 text-xs sm:grid-cols-3">
                        <div>
                            <dt class="uppercase tracking-[0.14em] text-grit-mist">Issued</dt>
                            <dd class="mt-0.5 tabular-nums">{{ $s->created_at?->utc()->toDateTimeString() ?? '—' }} <span class="text-grit-mist">({{ \App\Support\RelativeTime::ago($s->created_at) }})</span></dd>
                        </div>
                        <div>
                            <dt class="uppercase tracking-[0.14em] text-grit-mist">Last seen</dt>
                            <dd class="mt-0.5 tabular-nums">{{ $s->last_seen_at?->utc()->toDateTimeString() ?? '—' }} <span class="text-grit-mist">({{ \App\Support\RelativeTime::ago($s->last_seen_at) }})</span></dd>
                        </div>
                        <div>
                            <dt class="uppercase tracking-[0.14em] text-grit-mist">Expires</dt>
                            <dd class="mt-0.5 tabular-nums">
                                {{ $s->expires_at?->utc()->toDateTimeString() ?? '—' }}
                                @if ($s->expires_at && $s->expires_at->isFuture())
                                    <span class="text-grit-mist">(in {{ \App\Support\RelativeTime::duration((int) $now->diffInSeconds($s->expires_at)) }})</span>
                                @endif
                            </dd>
                        </div>
                        <div>
                            <dt class="uppercase tracking-[0.14em] text-grit-mist">Lifetime</dt>
                            <dd class="mt-0.5 tabular-nums">{{ $lifetime }}{{ $sessionState === 'active' ? ' so far' : '' }}</dd>
                        </div>
                        <div>
                            <dt class="uppercase tracking-[0.14em] text-grit-mist">Revoked</dt>
                            <dd class="mt-0.5 tabular-nums">
                                @if ($s->revoked_at)
                                    {{ $s->revoked_at->utc()->toDateTimeString() }} <span class="text-grit-mist">· {{ $s->revoked_reason ?? 'unrecorded' }}</span>
                                @else
                                    —
                                @endif
                            </dd>
                        </div>
                        <div>
                            <dt class="uppercase tracking-[0.14em] text-grit-mist">Game · country</dt>
                            <dd class="mt-0.5">{{ \App\Support\ClientGeo::gameLabel($s->game_id) }} · {{ \App\Support\ClientGeo::countryLabel($s->country_code) }}</dd>
                        </div>
                    </dl>

                    <div class="flex flex-wrap items-center justify-between gap-3">
                        <p class="flex flex-wrap items-center gap-2 text-xs text-grit-mist">
                            <x-reveal :value="$s->jwt_id" label="jti" />
                            <x-reveal :value="$s->token_hash ? substr($s->token_hash, 0, 12) : null" label="hash" />
                            <x-reveal :value="$s->client_ip" label="ip" />
                            <x-reveal :value="$s->user?->discord_id" label="discord" />
                        </p>
                        @if ($sessionState === 'active')
                            <form method="POST" action="{{ route('nexus.admin.sessions.revoke', $s->id) }}">
                                @csrf
                                <button type="submit" class="text-xs text-red-300 underline underline-offset-2">Revoke</button>
                            </form>
                        @endif
                    </div>
                </li>
            @empty
                <li class="py-4 text-grit-mist">No sessions match these filters.</li>
            @endforelse
        </ul>

        @if ($sessions->hasPages())
            <div class="mt-6 flex items-center justify-between text-xs uppercase tracking-[0.14em] text-grit-mist">
                @if ($sessions->onFirstPage())
                    <span>Newer</span>
                @else
                    <a href="{{ $sessions->previousPageUrl() }}" class="hover:text-grit-text">Newer</a>
                @endif
                <span class="tabular-nums">Page {{ $sessions->currentPage() }} / {{ $sessions->lastPage() }}</span>
                @if ($sessions->hasMorePages())
                    <a href="{{ $sessions->nextPageUrl() }}" class="hover:text-grit-text">Older</a>
                @else
                    <span>Older</span>
                @endif
            </div>
        @endif
    </section>
@endsection
