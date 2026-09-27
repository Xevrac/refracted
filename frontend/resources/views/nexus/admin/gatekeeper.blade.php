@extends('nexus.admin.layout', ['title' => 'Gatekeeper', 'active' => 'gatekeeper'])

@section('content')
    <div>
        <h1 class="font-display text-3xl font-semibold tracking-[-0.03em]">Gatekeeper</h1>
        <p class="mt-2 max-w-xl text-sm text-grit-mist">
            Registration lock and Discord playtest whitelist.
        </p>
    </div>

    <section class="border border-grit-line bg-grit-surface/80 px-5 py-5">
        <h2 class="font-display text-xl font-semibold tracking-[-0.02em]">Registrations</h2>
        <p class="mt-2 text-sm text-grit-mist">
            Status:
            <span class="text-grit-text">{{ $registrationsOpen ? 'Open' : 'Closed' }}</span>
        </p>
        <button
            type="button"
            class="mt-5 bg-signal px-4 py-2.5 font-display text-sm font-semibold text-white hover:bg-signal-soft"
            onclick="document.getElementById('registrations-confirm').showModal()"
        >
            {{ $registrationsOpen ? 'Close registrations' : 'Open registrations' }}
        </button>
    </section>

    <dialog
        id="registrations-confirm"
        class="w-[min(100%,28rem)] border border-grit-line bg-grit-surface p-0 text-grit-text shadow-xl backdrop:bg-black/70"
    >
        <form method="POST" action="{{ route('nexus.admin.registrations') }}" class="px-5 py-5 sm:px-6">
            @csrf
            <input type="hidden" name="registrations_open" value="{{ $registrationsOpen ? '0' : '1' }}">
            <input type="hidden" name="confirm" value="1">

            <h3 class="font-display text-lg font-semibold tracking-[-0.02em]">
                {{ $registrationsOpen ? 'Close registrations?' : 'Open registrations?' }}
            </h3>
            <p class="mt-3 text-sm leading-relaxed text-grit-mist">
                @if ($registrationsOpen)
                    No new Discord accounts will be able to create a Nexus profile until you open registrations again.
                    Discord ids already on the playtest whitelist can still register.
                @else
                    New Discord accounts may register, subject to the whitelist setting.
                @endif
            </p>

            <div class="mt-6 flex flex-wrap justify-end gap-3">
                <button
                    type="button"
                    class="border border-grit-line px-4 py-2 text-sm text-grit-mist hover:border-signal hover:text-grit-text"
                    onclick="this.closest('dialog').close()"
                >
                    Cancel
                </button>
                <button
                    type="submit"
                    class="bg-signal px-4 py-2 font-display text-sm font-semibold text-white hover:bg-signal-soft"
                >
                    {{ $registrationsOpen ? 'Close registrations' : 'Open registrations' }}
                </button>
            </div>
        </form>
    </dialog>

    <section class="border border-grit-line bg-grit-surface/80 px-5 py-5">
        <h2 class="font-display text-xl font-semibold tracking-[-0.02em]">Discord playtest whitelist</h2>
        <p class="mt-2 text-sm text-grit-mist">
            When enabled, only listed Discord user ids may create a new Nexus account.
            Listed ids also bypass a closed registration lock.
            Existing accounts are unaffected (unless banned).
        </p>
        <form method="POST" action="{{ route('nexus.admin.whitelist.mode') }}" class="mt-4 flex flex-wrap items-center gap-3">
            @csrf
            <input type="hidden" name="whitelist_enabled" value="{{ $whitelistEnabled ? '0' : '1' }}">
            <button type="submit" class="border border-grit-line px-4 py-2 text-sm hover:border-signal">
                {{ $whitelistEnabled ? 'Disable whitelist' : 'Enable whitelist' }}
            </button>
            <span class="text-xs uppercase tracking-[0.14em] text-grit-mist">
                {{ $whitelistEnabled ? 'Enforced' : 'Off' }}
            </span>
        </form>

        <form method="POST" action="{{ route('nexus.admin.whitelist.store') }}" class="mt-6 grid gap-3 sm:grid-cols-[1fr_1fr_auto]">
            @csrf
            <input name="discord_id" required placeholder="Discord user id" class="border border-grit-line bg-grit-bg px-3 py-2 text-sm outline-none focus:border-signal">
            <input name="note" placeholder="Note (optional)" class="border border-grit-line bg-grit-bg px-3 py-2 text-sm outline-none focus:border-signal">
            <button type="submit" class="bg-signal px-4 py-2 font-display text-sm font-semibold text-white hover:bg-signal-soft">Add</button>
        </form>

        <ul class="mt-6 divide-y divide-grit-line/60 text-sm">
            @forelse ($whitelist as $entry)
                <li class="flex flex-wrap items-center justify-between gap-3 py-3">
                        <div>
                            <p class="font-display"><x-reveal :value="$entry->discord_id" /></p>
                            @if ($entry->note)
                                <p class="text-xs text-grit-mist">{{ $entry->note }}</p>
                            @endif
                        </div>
                    <form method="POST" action="{{ route('nexus.admin.whitelist.destroy') }}">
                        @csrf
                        @method('DELETE')
                        <input type="hidden" name="discord_id" value="{{ $entry->discord_id }}">
                        <button
                            type="submit"
                            class="inline-flex items-center justify-center transition"
                            style="width: 2rem; height: 2rem; border: 1px solid rgba(248, 113, 113, 0.45); color: #f87171; background: transparent; cursor: pointer;"
                            onmouseover="this.style.borderColor='#f87171'; this.style.background='rgba(239, 68, 68, 0.12)'; this.style.color='#fca5a5';"
                            onmouseout="this.style.borderColor='rgba(248, 113, 113, 0.45)'; this.style.background='transparent'; this.style.color='#f87171';"
                            aria-label="Remove {{ $entry->discord_id }}"
                            data-tippy-content="Remove"
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 0 16 16"
                                width="14"
                                height="14"
                                fill="currentColor"
                                aria-hidden="true"
                                style="width: 0.875rem; height: 0.875rem; display: block;"
                            >
                                <path d="M9.38 0C10.04 0 10.65 0.37 10.95 0.97L11.96 3H14.78C15.2 3 15.53 3.34 15.53 3.75C15.53 4.16 15.2 4.5 14.78 4.5H14V12.25C14 13.77 12.77 15 11.25 15H4.75C3.23 15 2 13.77 2 12.25V4.5H1.25C0.84 4.5 0.5 4.16 0.5 3.75C0.5 3.34 0.84 3 1.25 3H4.04L5.05 0.97C5.35 0.37 5.96 0 6.62 0H9.38ZM3.5 12.25C3.5 12.94 4.06 13.5 4.75 13.5H11.25C11.94 13.5 12.5 12.94 12.5 12.25V4.5H3.5V12.25ZM6.25 6C6.66 6 7 6.34 7 6.75V11.25C7 11.66 6.66 12 6.25 12C5.84 12 5.5 11.66 5.5 11.25V6.75C5.5 6.34 5.84 6 6.25 6ZM9.75 6C10.16 6 10.5 6.34 10.5 6.75V11.25C10.5 11.66 10.16 12 9.75 12C9.34 12 9 11.66 9 11.25V6.75C9 6.34 9.34 6 9.75 6ZM6.62 1.5C6.52 1.5 6.44 1.55 6.39 1.64L5.71 3H10.29L9.61 1.64C9.56 1.55 9.48 1.5 9.38 1.5H6.62Z" />
                            </svg>
                        </button>
                    </form>
                </li>
            @empty
                <li class="py-3 text-grit-mist">No whitelist entries.</li>
            @endforelse
        </ul>
    </section>
@endsection
