<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}" class="dark">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="color-scheme" content="dark">
    <title>CDN Login</title>
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
                <img src="{{ asset('images/brand/refracted-icon.png') }}" alt="" class="h-9 w-9 shrink-0 rounded-lg object-contain">
                <div>
                    <p class="font-display text-xl font-semibold leading-none tracking-[-0.02em]">CDN</p>
                    <p class="mt-1 text-xs text-grit-mist">{{ config('cdn.domain') }}</p>
                </div>
            </div>

            @if ($errors->any())
                <div class="mt-5 border border-red-500/30 bg-red-500/10 px-3.5 py-3 text-sm text-red-200">
                    {{ $errors->first() }}
                </div>
            @endif

            @if (\App\Support\CdnHosts::localDashboard())
                <form method="POST" action="{{ route('cdn.admin.login.dev') }}" class="mt-7">
                    @csrf
                    <button type="submit" class="inline-flex w-full items-center justify-center border border-grit-line bg-grit-surface px-5 py-3 text-sm hover:border-signal/40">
                        Continue as local admin
                    </button>
                </form>
            @endif

            <a
                href="{{ \App\Support\CdnHosts::discordLoginUrl() }}"
                class="{{ \App\Support\CdnHosts::localDashboard() ? 'mt-4' : 'mt-7' }} inline-flex w-full items-center justify-center gap-2 border border-[#5865F2]/50 bg-[#5865F2]/15 px-5 py-3 font-display text-sm font-semibold tracking-[-0.01em] text-[#dee0ff] transition hover:border-[#5865F2] hover:bg-[#5865F2]/25 hover:text-white"
            >
                Continue with Discord
            </a>
        </div>
    </main>
</body>
</html>
