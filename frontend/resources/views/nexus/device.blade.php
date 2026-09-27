<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}" class="dark">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="color-scheme" content="dark">
    <title>Nexus — Approve launcher</title>
    <link rel="icon" href="{{ asset('images/brand/refracted-icon.png') }}" type="image/png">
    <link rel="preconnect" href="https://fonts.bunny.net">
    <link href="https://fonts.bunny.net/css?family=sora:400,500,600|space-grotesk:500,600,700&display=swap" rel="stylesheet" />
    @vite(['resources/css/app.css', 'resources/js/app.js'])
</head>
<body class="ref-canvas min-h-[100svh] font-sans text-grit-text antialiased">
    <div
        class="ref-wallpaper"
        style="--ref-wallpaper: url('{{ asset('images/brand/refracted-wallpaper.png') }}')"
        aria-hidden="true"
    ></div>

    <main class="mx-auto flex min-h-[100svh] w-full max-w-md flex-col justify-center px-5 py-16">
        <div class="ref-panel animate-rise px-6 py-7 sm:px-7 sm:py-8">
            <p class="font-display text-xl font-semibold tracking-[-0.02em]">Nexus</p>
            <p class="mt-2 text-sm text-grit-mist">Sign-in to Refracted Launcher</p>

            @if (session('status'))
                <div class="mt-6 border border-signal/30 bg-signal/10 px-4 py-3 text-sm text-signal-soft">
                    {{ session('status') }}
                </div>
            @else
                @if ($errors->any())
                    <div class="mt-6 border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-200">
                        {{ $errors->first() }}
                    </div>
                @endif

                @if ($expired)
                    <p class="mt-6 text-sm leading-relaxed text-grit-mist">Code expired. Start sign-in again from the launcher and try again.</p>
                @elseif ($userCode === '')
                    <p class="mt-6 text-sm leading-relaxed text-grit-mist">Enter the code shown in the Refracted Launcher.</p>
                    <form method="GET" action="{{ route('nexus.device') }}" class="mt-5 space-y-4">
                        <input
                            name="code"
                            type="text"
                            required
                            maxlength="16"
                            placeholder="ABCDE-FGHIJ"
                            autocomplete="one-time-code"
                            class="w-full border border-grit-line bg-grit-bg px-3 py-2.5 font-display text-lg tracking-[0.2em] text-grit-text outline-none focus:border-signal"
                        >
                        <button type="submit" class="inline-flex w-full items-center justify-center bg-signal px-5 py-3 font-display text-sm font-semibold text-white hover:bg-signal-soft">
                            Continue
                        </button>
                    </form>
                @elseif (($pending['status'] ?? '') === 'approved' || ($pending['status'] ?? '') === 'consumed')
                    <p class="mt-6 text-sm leading-relaxed text-grit-mist">This launcher is already approved.</p>
                @else
                    <p class="mt-6 text-sm leading-relaxed text-grit-mist">
                        Approving signs
                        <span class="font-medium text-grit-text">{{ auth()->user()->name }}</span>
                        into the Refracted Launcher on this device.
                    </p>
                    <p class="mt-3 text-sm leading-relaxed text-grit-mist">
                        Only continue if you started sign-in from your launcher.
                    </p>

                    <div
                        class="mx-auto mt-8 mb-8 w-full max-w-xs text-center"
                        style="border: 1px solid #2a3038; background: #0a0b0d; box-shadow: inset 0 2px 8px rgba(0, 0, 0, 0.55), 0 1px 0 rgba(255, 255, 255, 0.04); padding: 1rem 1.25rem;"
                    >
                        <p class="font-display text-2xl font-semibold tracking-[0.18em] tabular-nums text-grit-text sm:text-3xl">
                            {{ $userCode }}
                        </p>
                    </div>

                    <form method="POST" action="{{ route('nexus.device.approve') }}" class="space-y-4">
                        @csrf
                        <input type="hidden" name="user_code" value="{{ $userCode }}">
                        <div class="space-y-2">
                            <label class="block text-[11px] uppercase tracking-[0.16em] text-grit-mist" for="confirm_code">Type the code to confirm</label>
                            <input
                                id="confirm_code"
                                name="confirm_code"
                                type="text"
                                required
                                maxlength="16"
                                autocomplete="off"
                                spellcheck="false"
                                class="w-full border border-grit-line bg-grit-bg px-3 py-2.5 font-display text-lg tracking-[0.2em] text-grit-text outline-none focus:border-signal"
                            >
                        </div>
                        <button type="submit" class="inline-flex w-full items-center justify-center bg-signal px-5 py-3 font-display text-sm font-semibold text-white hover:bg-signal-soft">
                            Approve Refracted Launcher
                        </button>
                    </form>
                @endif
            @endif

            <p class="mt-7 text-center text-xs text-grit-mist/70">
                <a href="{{ route('nexus.profile') }}" class="underline decoration-grit-line underline-offset-2 hover:text-grit-text">Back to profile</a>
            </p>
        </div>
    </main>
</body>
</html>
