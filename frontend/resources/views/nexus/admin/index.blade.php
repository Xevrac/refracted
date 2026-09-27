@extends('nexus.admin.layout', ['title' => 'Admin', 'active' => 'admin'])

@section('content')
    <div>
        <h1 class="font-display text-3xl font-semibold tracking-[-0.03em]">Admin</h1>
        <p class="mt-2 max-w-xl text-sm text-grit-mist">
            Manage bans and recent accounts.
        </p>
    </div>

    <section class="border border-grit-line bg-grit-surface/80 px-5 py-5">
        <h2 class="font-display text-xl font-semibold tracking-[-0.02em]">Bans</h2>
        <p class="mt-2 text-sm text-grit-mist">
            Temporary or permanent. Active game tickets are revoked.
        </p>

        <form method="POST" action="{{ route('nexus.admin.bans.store') }}" class="mt-6 grid gap-3 sm:grid-cols-2">
            @csrf
            <input name="user_id" type="number" placeholder="Nexus user id (optional)" class="border border-grit-line bg-grit-bg px-3 py-2 text-sm outline-none focus:border-signal">
            <input name="discord_id" placeholder="Discord id (optional if user id set)" class="border border-grit-line bg-grit-bg px-3 py-2 text-sm outline-none focus:border-signal">
            <input name="reason" placeholder="Reason" class="border border-grit-line bg-grit-bg px-3 py-2 text-sm outline-none focus:border-signal sm:col-span-2">
            <select name="duration" class="border border-grit-line bg-grit-bg px-3 py-2 text-sm outline-none focus:border-signal">
                <option value="permanent">Permanent</option>
                <option value="1h">1 hour</option>
                <option value="24h">24 hours</option>
                <option value="7d">7 days</option>
                <option value="30d">30 days</option>
            </select>
            <button type="submit" class="bg-red-600/90 px-4 py-2 font-display text-sm font-semibold text-white hover:bg-red-500">Ban</button>
        </form>

        <ul class="mt-6 divide-y divide-grit-line/60 text-sm">
            @forelse ($activeBans as $ban)
                <li class="flex flex-wrap items-start justify-between gap-3 py-3">
                    <div>
                        <p class="font-display flex flex-wrap items-center gap-2">
                            @if ($ban->user_id)
                                <span class="inline-flex items-center gap-1">user <x-reveal :value="$ban->user_id" /></span>
                            @endif
                            @if ($ban->discord_id)
                                <span class="inline-flex items-center gap-1">discord <x-reveal :value="$ban->discord_id" /></span>
                            @endif
                        </p>
                        <p class="text-xs text-grit-mist">
                            {{ $ban->isPermanent() ? 'Permanent' : 'Until '.$ban->banned_until?->utc()->toDateTimeString().' UTC' }}
                            @if ($ban->reason) — {{ $ban->reason }} @endif
                        </p>
                    </div>
                    <form method="POST" action="{{ route('nexus.admin.bans.lift', $ban) }}">
                        @csrf
                        <button type="submit" class="text-xs text-signal-soft underline underline-offset-2">Unban</button>
                    </form>
                </li>
            @empty
                <li class="py-3 text-grit-mist">No active bans.</li>
            @endforelse
        </ul>
    </section>

    <section class="border border-grit-line bg-grit-surface/80 px-5 py-5">
        <h2 class="font-display text-xl font-semibold tracking-[-0.02em]">Recent Nexus users</h2>

        <div class="mt-4 overflow-x-auto">
            <table class="w-full min-w-[36rem] text-left text-sm">
                <thead>
                    <tr class="border-b border-grit-line/80 text-[11px] uppercase tracking-[0.14em] text-grit-mist">
                        <th class="pb-3 pr-3 font-medium">User id</th>
                        <th class="pb-3 pr-3 font-medium">Username</th>
                        <th class="pb-3 pr-3 font-medium">Discord id</th>
                        <th class="pb-3 pr-3 font-medium">Last seen</th>
                        <th class="pb-3 font-medium"><span class="sr-only">Actions</span></th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-grit-line/60">
                    @forelse ($users as $row)
                        @php
                            $u = $row['user'];
                        @endphp
                        <tr>
                            <td class="py-2.5 pr-3"><x-reveal :value="$u->id" class="font-display" /></td>
                            <td class="py-2.5 pr-3">{{ $u->username }}</td>
                            <td class="py-2.5 pr-3"><x-reveal :value="$u->discord_id" /></td>
                            <td class="py-2.5 pr-3 text-grit-mist">{{ $row['last_seen'] }}</td>
                            <td class="py-2.5 text-right">
                                <a
                                    href="{{ route('nexus.admin.lookup', ['q' => $u->id]) }}"
                                    class="inline-flex items-center justify-center border border-grit-line text-grit-mist transition hover:border-signal hover:text-signal-soft"
                                    style="width: 2rem; height: 2rem;"
                                    aria-label="Look up {{ $u->username }}"
                                    data-tippy-content="Lookup"
                                >
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        viewBox="0 0 24 24"
                                        width="14"
                                        height="14"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="1.75"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        aria-hidden="true"
                                        style="width: 0.875rem; height: 0.875rem; display: block;"
                                    >
                                        <circle cx="11" cy="11" r="7" />
                                        <path d="m20 20-3.5-3.5" />
                                    </svg>
                                </a>
                            </td>
                        </tr>
                    @empty
                        <tr>
                            <td colspan="5" class="py-3 text-grit-mist">No users yet.</td>
                        </tr>
                    @endforelse
                </tbody>
            </table>
        </div>
    </section>
@endsection
