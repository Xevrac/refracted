<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}" class="dark">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="color-scheme" content="dark">
    <meta name="csrf-token" content="{{ csrf_token() }}">
    <title>@yield('title', 'Admin') — {{ config('app.name', 'Refracted') }}</title>
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

    <div class="mx-auto w-full max-w-5xl px-5 py-10 sm:px-8">
        <header class="mb-10 flex flex-wrap items-center justify-between gap-4 border-b border-grit-line pb-6">
            <div class="flex items-center gap-3">
                <img src="{{ asset('images/brand/refracted-icon.png') }}" alt="" class="h-9 w-9 rounded-lg object-contain">
                <div>
                    <p class="font-display text-lg font-semibold tracking-[-0.02em]">Refracted Admin</p>
                    <p class="text-xs text-grit-mist">{{ auth()->user()?->discord_username ?? auth()->user()?->name }}</p>
                </div>
            </div>
            <div class="flex items-center gap-3 text-sm">
                <a href="{{ url('/') }}" class="text-grit-mist underline decoration-grit-line underline-offset-2 hover:text-grit-text">View site</a>
                <form method="POST" action="{{ route('admin.logout') }}">
                    @csrf
                    <button type="submit" class="text-signal-soft underline decoration-signal/30 underline-offset-2 hover:text-signal">Log out</button>
                </form>
            </div>
        </header>

        @if (session('status'))
            <p class="mb-6 border border-signal/30 bg-signal/10 px-4 py-3 text-sm text-signal-soft">{{ session('status') }}</p>
        @endif

        @if ($errors->any())
            <div class="mb-6 border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-200">
                <ul class="list-disc space-y-1 ps-4">
                    @foreach ($errors->all() as $error)
                        <li>{{ $error }}</li>
                    @endforeach
                </ul>
            </div>
        @endif

        @yield('content')
    </div>
</body>
</html>
