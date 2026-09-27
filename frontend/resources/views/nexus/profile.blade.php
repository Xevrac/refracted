@extends('nexus.layout-account', ['title' => ($persona?->display_name ?? $websiteUser->name), 'active' => 'profile'])

@section('content')
    <div class="animate-fade">
        <p class="text-[11px] uppercase tracking-[0.18em] text-grit-mist">Nexus</p>
        <h1 class="mt-2 font-display text-3xl font-semibold tracking-[-0.03em] sm:text-4xl">
            {{ $persona?->display_name ?? $websiteUser->name }}
        </h1>
        <p class="mt-3 max-w-xl text-sm leading-relaxed text-grit-mist">
            Your Discord account is linked to a Nexus profile and persona.
        </p>
    </div>

    @if (session('status'))
        <div class="animate-rise mt-8 border border-signal/30 bg-signal/10 px-4 py-3 text-sm text-signal-soft">
            {{ session('status') }}
        </div>
    @endif

    @if ($errors->any())
        <div class="mt-8 border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-200">
            {{ $errors->first() }}
        </div>
    @endif

    @if (! $nexusUser)
        <div class="animate-rise-2 mt-12 border border-amber-500/30 bg-amber-500/10 px-4 py-4 text-sm text-amber-100">
            No Nexus profile yet. Sign out and sign in again with Discord.
        </div>
    @else
        <div class="mt-12 flex flex-col gap-8">
            <section class="animate-rise-2 grid gap-4 sm:grid-cols-2">
                <div class="border border-grit-line bg-grit-surface/80 px-5 py-5">
                    <p class="text-[11px] uppercase tracking-[0.16em] text-grit-mist">User id</p>
                    <div class="mt-3">
                        <x-reveal :value="$nexusUser->id" class="font-display text-base" />
                    </div>
                    <p class="mt-2 text-xs text-grit-mist/70">username · {{ $nexusUser->username }}</p>
                </div>
                <div class="border border-grit-line bg-grit-surface/80 px-5 py-5">
                    <p class="text-[11px] uppercase tracking-[0.16em] text-grit-mist">Persona id</p>
                    <p class="mt-3 font-display text-xl font-semibold tabular-nums tracking-[-0.02em]">{{ $persona?->id ?? '—' }}</p>
                    <p class="mt-2 text-xs text-grit-mist/70">shared across titles</p>
                </div>
            </section>

            <section class="animate-rise-3 border border-grit-line bg-grit-surface/80 px-5 py-6 sm:px-6">
                <h2 class="font-display text-lg font-semibold tracking-[-0.02em]">Display name</h2>
                <p
                    class="mt-3 border border-dashed px-3 py-2 text-xs leading-snug"
                    style="border-color: #ca8a04; background: rgba(250, 204, 21, 0.12); color: #facc15;"
                >
                    @if (! empty($bypassDisplayNameCooldown))
                        Staff can change display names without the {{ $displayNameCooldownDays }}-day cooldown.
                    @else
                        Can only be changed every {{ $displayNameCooldownDays }} days.
                    @endif
                </p>
                @if (! $canChangeDisplayName && $nextDisplayNameChangeAt)
                    <div
                        class="mt-2 flex items-center gap-2.5 border border-signal/30 bg-signal/10 px-3 py-2.5 text-xs leading-snug text-signal-soft"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            width="16"
                            height="16"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1.75"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            aria-hidden="true"
                            style="width: 1rem; height: 1rem; flex-shrink: 0;"
                        >
                            <circle cx="12" cy="12" r="9" />
                            <path d="M12 7v5l3 2" />
                        </svg>
                        <span>Next change available {{ $nextDisplayNameChangeAt->utc()->format('Y-m-d') }} UTC.</span>
                    </div>
                @endif
                <p class="mt-3 text-sm text-grit-mist">
                    Shown in-game as your persona name.
                </p>

                <form
                    method="POST"
                    action="{{ route('nexus.profile.update') }}"
                    class="mt-6 flex flex-col gap-3 sm:flex-row sm:items-stretch"
                >
                    @csrf
                    @method('PUT')
                    <div class="min-w-0 flex-1">
                        <label class="sr-only" for="display_name">Display name</label>
                        <input
                            id="display_name"
                            name="display_name"
                            type="text"
                            value="{{ old('display_name', $persona?->display_name) }}"
                            required
                            minlength="2"
                            maxlength="64"
                            @disabled(! $canChangeDisplayName)
                            class="h-11 w-full border border-grit-line bg-grit-bg px-3 text-sm text-grit-text outline-none transition focus:border-signal disabled:cursor-not-allowed disabled:opacity-60"
                        >
                    </div>
                    <button
                        type="submit"
                        @disabled(! $canChangeDisplayName)
                        class="inline-flex h-11 shrink-0 items-center justify-center bg-signal px-6 font-display text-sm font-semibold tracking-[-0.01em] text-white transition hover:bg-signal-soft disabled:cursor-not-allowed disabled:opacity-60"
                    >
                        Save
                    </button>
                </form>
            </section>

            <section class="border border-grit-line bg-grit-surface/80 px-5 py-6 sm:px-6">
                <div class="flex flex-wrap items-baseline justify-between gap-2">
                    <h2 class="font-display text-lg font-semibold tracking-[-0.02em]">Game sessions</h2>
                    <a
                        href="{{ route('nexus.profile.games') }}"
                        class="inline-flex items-center justify-center border border-grit-line text-signal-soft transition hover:border-signal hover:text-white"
                        style="width: 2rem; height: 2rem;"
                        aria-label="Jump to My Games"
                        data-tippy-content="Jump to My Games"
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
                            <path d="M7 17 17 7" />
                            <path d="M8 7h9v9" />
                        </svg>
                    </a>
                </div>
                <p class="mt-2 text-sm text-grit-mist">
                    Active sessions from the launcher appear here.
                </p>

                @if ($activeSessions->isEmpty())
                    <p class="mt-6 text-sm text-grit-mist/80">No active game sessions.</p>
                @else
                    <ul class="mt-6 divide-y divide-grit-line/80">
                        @foreach ($activeSessions as $session)
                            <li class="flex flex-wrap items-baseline justify-between gap-2 py-3 text-sm">
                                <span class="text-grit-text">
                                    {{ \App\Support\ClientGeo::gameLabel($session->game_id) }}
                                    <span class="text-grit-mist">· persona {{ $session->persona_id }}</span>
                                </span>
                                <span class="text-grit-mist">expires {{ $session->expires_at?->utc()->format('Y-m-d H:i') }} UTC</span>
                            </li>
                        @endforeach
                    </ul>
                @endif
            </section>

            <section class="border border-grit-line/70 px-5 py-5 text-sm text-grit-mist sm:px-6">
                <p class="flex flex-wrap items-center gap-2">
                    <span class="text-grit-text">Discord</span>
                    <span>· {{ $websiteUser->discord_username ?: $websiteUser->name }}</span>
                    <span class="inline-flex items-center gap-1">· id <x-reveal :value="$websiteUser->discord_id" /></span>
                </p>
                <p class="mt-2 flex flex-wrap items-center gap-2">
                    <span class="text-grit-text">Nexus</span>
                    <span class="inline-flex items-center gap-1">· user <x-reveal :value="$nexusUser->id" /></span>
                </p>
                @if (filled($websiteUser->email) || filled($nexusUser->email))
                    <p class="mt-2 flex flex-wrap items-center gap-2">
                        <span class="text-grit-text">Email</span>
                        <x-reveal :value="$websiteUser->email ?: $nexusUser->email" />
                    </p>
                @endif
            </section>
        </div>
    @endif
@endsection
