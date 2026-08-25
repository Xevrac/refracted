<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}" class="dark">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="description" content="Refracted helps keep classic online games playable when official servers go away.">
    <meta name="color-scheme" content="dark">

    <title>{{ config('app.name', 'Refracted') }}</title>

    <link rel="icon" href="{{ asset('images/brand/refracted-icon.png') }}" type="image/png">
    <link rel="apple-touch-icon" href="{{ asset('images/brand/refracted-icon.png') }}">

    <link rel="preconnect" href="https://fonts.bunny.net">
    <link href="https://fonts.bunny.net/css?family=sora:400,500,600|space-grotesk:500,600,700&display=swap" rel="stylesheet" />

    @vite(['resources/css/app.css', 'resources/js/app.js'])
    <link rel="stylesheet" href="https://unpkg.com/tippy.js@6/dist/tippy.css">
</head>
<body class="ref-canvas min-h-[100svh] font-sans text-grit-text antialiased">
    <div
        class="ref-wallpaper"
        style="--ref-wallpaper: url('{{ asset('images/brand/refracted-wallpaper.png') }}')"
        aria-hidden="true"
    ></div>

    <main class="mx-auto w-full max-w-6xl px-5 py-16 sm:px-8 lg:px-10 lg:py-20">
        {{-- Hero --}}
        <section class="grid w-full items-center gap-10 lg:min-h-[78svh] lg:grid-cols-2 lg:gap-14">
            <div class="min-w-0">
                <div class="animate-fade flex items-center gap-3.5 sm:gap-4">
                    <img
                        src="{{ asset('images/brand/refracted-icon.png') }}"
                        alt=""
                        class="h-[4.25rem] w-[4.25rem] shrink-0 rounded-[1.05rem] object-contain shadow-[0_0_40px_rgba(0,136,255,0.22)] sm:h-[5rem] sm:w-[5rem] sm:rounded-[1.2rem]"
                        draggable="false"
                    >
                    <span class="flex h-[4.25rem] items-center font-display text-[clamp(1.85rem,4.8vw,3.15rem)] font-bold uppercase leading-none tracking-[0.08em] text-grit-text sm:h-[5rem]">
                        Refracted
                    </span>
                </div>

                <h1 class="animate-rise mt-10 max-w-lg font-sans text-2xl font-semibold leading-snug tracking-[-0.02em] text-grit-text sm:text-3xl">
                    Games & builds that deserve to keep living.
                </h1>

                <p class="animate-rise-2 mt-5 max-w-md text-base leading-relaxed text-grit-mist sm:text-lg">
                    When official online services or support shuts down, Refracted helps bring those games back so they're not lost to time.
                </p>

                <div class="animate-rise-3 mt-8 flex flex-wrap items-center gap-3" style="display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem; margin-top: 2rem;">
                    <span style="display: inline-block;" data-tippy-content="Coming soon">
                        <button
                            type="button"
                            disabled
                            style="cursor: not-allowed; border: 1px solid #2a3038; background: #121417; color: #9aa3ad; opacity: 0.5; padding: 0.625rem 1.25rem; font-family: 'Space Grotesk', sans-serif; font-size: 0.75rem; font-weight: 600; letter-spacing: 0.14em; text-transform: uppercase;"
                        >
                            Make account
                        </button>
                    </span>

                    <span style="display: inline-block;" data-tippy-content="Coming soon">
                        <button
                            type="button"
                            disabled
                            style="cursor: not-allowed; border: 1px solid #2a3038; background: #121417; color: #9aa3ad; opacity: 0.5; padding: 0.625rem 1.25rem; font-family: 'Space Grotesk', sans-serif; font-size: 0.75rem; font-weight: 600; letter-spacing: 0.14em; text-transform: uppercase;"
                        >
                            Refracted Launcher
                        </button>
                    </span>

                    <a
                        href="https://github.com/Xevrac/refracted"
                        target="_blank"
                        rel="noopener noreferrer"
                        aria-label="GitHub repository"
                        title="GitHub"
                        style="display: inline-flex; align-items: center; justify-content: center; width: 2.625rem; height: 2.625rem; border: 1px solid #2a3038; color: #c5ccd3; text-decoration: none; box-sizing: border-box;"
                    >
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                            <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/>
                        </svg>
                    </a>
                </div>
            </div>

            <aside
                class="animate-rise-3 ref-panel relative w-full min-w-0 overflow-hidden"
                data-ref-slider
                aria-roledescription="carousel"
                aria-label="Games we support"
            >
                @if ($titles->isEmpty())
                    <div class="flex aspect-[4/3] items-center justify-center px-6 text-center sm:aspect-[5/4]">
                        <p class="text-sm text-grit-mist">Games will appear here soon.</p>
                    </div>
                @else
                <div class="relative aspect-[4/3] w-full overflow-hidden bg-grit-surface sm:aspect-[5/4]" style="position: relative;">
                    @foreach ($titles as $index => $title)
                        <article
                            class="ref-slide {{ $index === 0 ? 'is-active' : '' }}"
                            data-ref-slide
                            aria-hidden="{{ $index === 0 ? 'false' : 'true' }}"
                            style="position: absolute; inset: 0;"
                        >
                            @if ($title->imageUrl())
                                <img
                                    src="{{ $title->imageUrl() }}"
                                    alt=""
                                    draggable="false"
                                    style="position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; object-position: center;"
                                >
                            @endif
                            <div
                                aria-hidden="true"
                                style="position: absolute; inset: 0; pointer-events: none; background: linear-gradient(to top, rgba(10,11,13,0.75) 0%, rgba(10,11,13,0.2) 45%, transparent 70%);"
                            ></div>

                            <div
                                style="position: absolute; left: 0; right: 0; bottom: 0; z-index: 10; width: 100%; box-sizing: border-box; border-top: 1px solid rgba(42,48,56,0.95); background-color: rgba(12, 14, 16, 0.94); padding: 1rem 1.25rem;"
                            >
                                <div class="flex items-start justify-between gap-3">
                                    <p class="font-display text-[11px] font-semibold uppercase tracking-[0.2em] text-signal">
                                        Games we support
                                    </p>
                                    <span class="shrink-0 text-[11px] uppercase leading-none tracking-[0.16em] text-grit-mist">
                                        {{ $title->status }}
                                    </span>
                                </div>
                                <h2 class="mt-2 font-display text-2xl font-semibold leading-none tracking-[-0.02em] text-grit-text">
                                    {{ $title->name }}
                                </h2>
                                @if (filled($title->blurb))
                                    <p class="mt-2 text-sm leading-relaxed text-grit-mist">
                                        {{ $title->blurb }}
                                    </p>
                                @endif
                                @if (filled($title->url))
                                    <p class="mt-3 inline-flex items-center gap-2 text-signal-soft">
                                        <a
                                            href="{{ $title->url }}"
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            class="font-medium underline decoration-signal/40 underline-offset-[0.35em] transition hover:text-signal hover:decoration-signal"
                                        >
                                            {{ $title->linkLabel() }}
                                        </a>
                                        <span aria-hidden="true">→</span>
                                    </p>
                                @endif
                            </div>
                        </article>
                    @endforeach
                </div>

                @if ($titles->count() > 1)
                    <div class="flex items-center justify-center gap-2 border-t border-grit-line px-6 py-4" role="tablist" aria-label="Choose a game">
                        @foreach ($titles as $index => $title)
                            <button
                                type="button"
                                data-ref-dot
                                class="h-1.5 w-6 transition {{ $index === 0 ? 'bg-signal' : 'bg-grit-line' }}"
                                aria-label="Show {{ $title->name }}"
                                aria-current="{{ $index === 0 ? 'true' : 'false' }}"
                            ></button>
                        @endforeach
                    </div>
                @endif
                @endif
            </aside>
        </section>

        {{-- Communities --}}
        @if ($communities->isNotEmpty())
            <section class="mt-20 border-t border-grit-line/80 pt-14 sm:mt-24 sm:pt-16" aria-labelledby="communities-heading">
                <div class="max-w-xl" data-ref-reveal>
                    <p class="font-display text-xs font-semibold uppercase tracking-[0.2em] text-signal">
                        Communities
                    </p>
                    <h2 id="communities-heading" class="mt-3 font-display text-2xl font-semibold tracking-[-0.02em] text-grit-text sm:text-3xl">
                        Projects built on Refracted.
                    </h2>
                    <p class="mt-4 text-base leading-relaxed text-grit-mist">
                        Have a look and see what Refracted has enabled so far..
                    </p>
                </div>

                <div class="mt-10 grid gap-5 md:grid-cols-2">
                    @foreach ($communities as $i => $community)
                        <article
                            class="ref-panel group relative overflow-hidden"
                            data-ref-reveal
                            style="transition-delay: {{ $i * 80 }}ms"
                        >
                            <div class="relative aspect-[16/9] overflow-hidden bg-grit-bg">
                                @if ($community->imageUrl())
                                    <img
                                        src="{{ $community->imageUrl() }}"
                                        alt=""
                                        class="h-full w-full object-contain object-center transition duration-700 group-hover:opacity-95"
                                        draggable="false"
                                    >
                                @endif
                                <div class="pointer-events-none absolute inset-0 bg-gradient-to-t from-grit-panel via-transparent to-transparent"></div>
                            </div>

                            <div class="relative px-6 py-6 sm:px-7">
                                <div class="flex items-center gap-3">
                                    @if ($community->iconUrl())
                                        <img
                                            src="{{ $community->iconUrl() }}"
                                            alt=""
                                            class="h-8 w-8 object-contain"
                                            draggable="false"
                                        >
                                    @endif
                                    <div>
                                        <h3 class="font-display text-xl font-semibold tracking-[-0.02em] text-grit-text">
                                            {{ $community->name }}
                                        </h3>
                                        <p class="mt-0.5 text-[11px] uppercase tracking-[0.16em] text-signal">
                                            {{ $community->tagline }}
                                        </p>
                                    </div>
                                </div>
                                @if (filled($community->blurb))
                                    <p class="mt-4 text-sm leading-relaxed text-grit-mist">
                                        {{ $community->blurb }}
                                    </p>
                                @endif
                                @if (filled($community->url))
                                    <p class="mt-5 inline-flex items-center gap-2 text-signal-soft">
                                        <a
                                            href="{{ $community->url }}"
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            class="font-medium underline decoration-signal/40 underline-offset-[0.35em] transition hover:text-signal hover:decoration-signal"
                                        >
                                            {{ $community->linkLabel() }}
                                        </a>
                                        <span aria-hidden="true">→</span>
                                    </p>
                                @endif
                            </div>
                        </article>
                    @endforeach
                </div>
            </section>
        @endif

        <p class="mt-16 text-xs text-grit-mist/70 sm:mt-20">
            A community project. Not affiliated with Electronic Arts Inc.
            <a href="{{ route('legal') }}" class="underline decoration-grit-line underline-offset-2 hover:text-grit-text">Legal</a>
        </p>
    </main>

    <script src="https://unpkg.com/@popperjs/core@2"></script>
    <script src="https://unpkg.com/tippy.js@6"></script>
    <script>
        tippy('[data-tippy-content]', {
            theme: 'refracted',
            placement: 'top',
            animation: 'fade',
            arrow: true,
        });
    </script>
    <style>
        .tippy-box[data-theme~='refracted'] {
            background: #121417;
            color: #e8eaed;
            border: 1px solid #2a3038;
            font-family: Sora, ui-sans-serif, system-ui, sans-serif;
            font-size: 12px;
            letter-spacing: 0.04em;
            text-transform: uppercase;
            padding: 2px 2px;
        }
        .tippy-box[data-theme~='refracted'][data-placement^='top'] > .tippy-arrow::before {
            border-top-color: #2a3038;
        }
    </style>
</body>
</html>
