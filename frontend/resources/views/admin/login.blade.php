<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}" class="dark">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="color-scheme" content="dark">
    <title>Admin login — {{ config('app.name', 'Refracted') }}</title>
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

    <main class="mx-auto flex min-h-[100svh] w-full max-w-sm flex-col justify-center px-5 py-16">
        <div class="ref-panel px-6 py-7 sm:px-7 sm:py-8">
            <div class="flex items-center gap-3">
                <img
                    src="{{ asset('images/brand/refracted-icon.png') }}"
                    alt=""
                    class="h-9 w-9 shrink-0 rounded-lg object-contain"
                >
                <div class="flex h-9 items-center pt-[0.1em]">
                    <p class="font-display text-xl font-semibold leading-none tracking-[-0.02em]">Admin</p>
                </div>
            </div>

            <p class="mt-5 text-sm leading-relaxed text-grit-mist">
                Sign in to manage games and communities.
            </p>

            @if ($errors->any())
                <div class="mt-5 border border-red-500/30 bg-red-500/10 px-3.5 py-3 text-sm text-red-200">
                    {{ $errors->first() }}
                </div>
            @endif

            @if (app()->environment('local'))
                <form method="POST" action="{{ route('admin.login.dev') }}" class="mt-7 space-y-5">
                    @csrf
                    <div class="space-y-2">
                        <label class="block text-[11px] uppercase tracking-[0.16em] text-grit-mist" for="email">Email</label>
                        <input
                            id="email"
                            name="email"
                            type="email"
                            value="{{ old('email', 'admin@refracted.local') }}"
                            required
                            autocomplete="username"
                            class="w-full border border-grit-line bg-grit-bg px-3 py-2.5 text-sm text-grit-text outline-none transition focus:border-signal"
                        >
                    </div>
                    <div class="space-y-2">
                        <label class="block text-[11px] uppercase tracking-[0.16em] text-grit-mist" for="password">Password</label>
                        <input
                            id="password"
                            name="password"
                            type="password"
                            value="password"
                            required
                            autocomplete="current-password"
                            class="w-full border border-grit-line bg-grit-bg px-3 py-2.5 text-sm text-grit-text outline-none transition focus:border-signal"
                        >
                    </div>
                    <button
                        type="submit"
                        class="inline-flex w-full items-center justify-center bg-signal px-5 py-3 font-display text-sm font-semibold tracking-[-0.01em] text-white transition hover:bg-signal-soft"
                    >
                        Login
                    </button>
                </form>

                <div class="my-5 flex items-center gap-3">
                    <span class="h-px flex-1 bg-grit-line"></span>
                    <span class="text-[11px] uppercase tracking-[0.16em] text-grit-mist/50">or</span>
                    <span class="h-px flex-1 bg-grit-line"></span>
                </div>
            @endif

            <a
                href="{{ route('discord.login') }}"
                class="{{ app()->environment('local') ? '' : 'mt-7 ' }}inline-flex w-full items-center justify-center gap-2 border border-[#5865F2]/50 bg-[#5865F2]/15 px-5 py-3 font-display text-sm font-semibold tracking-[-0.01em] text-[#dee0ff] transition hover:border-[#5865F2] hover:bg-[#5865F2]/25 hover:text-white"
            >
                Discord
            </a>

            <p class="mt-7 text-center text-xs text-grit-mist/70">
                <a href="{{ url('/') }}" class="underline decoration-grit-line underline-offset-2 hover:text-grit-text">Back to site</a>
            </p>
        </div>
    </main>
</body>
</html>
