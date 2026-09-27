<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}" class="dark">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="color-scheme" content="dark">
    <meta name="csrf-token" content="{{ csrf_token() }}">
    <title>Nexus — {{ $title ?? 'Admin' }}</title>
    <link rel="icon" href="{{ asset('images/brand/refracted-icon.png') }}" type="image/png">
    <link rel="preconnect" href="https://fonts.bunny.net">
    <link href="https://fonts.bunny.net/css?family=sora:400,500,600|space-grotesk:500,600,700&display=swap" rel="stylesheet" />
    <link rel="stylesheet" href="https://unpkg.com/tippy.js@6/dist/tippy.css">
    @vite(['resources/css/app.css', 'resources/js/app.js'])
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
        .ref-spoiler {
            position: relative;
            display: inline-flex;
            max-width: 100%;
            align-items: center;
            justify-content: center;
            margin: 0;
            padding: 0.25rem 0.65rem;
            border: 1px solid rgba(42, 48, 56, 0.95);
            background: rgba(10, 11, 13, 0.92);
            cursor: pointer;
            user-select: none;
            vertical-align: baseline;
            overflow: hidden;
            white-space: nowrap;
        }
        .ref-spoiler__value {
            filter: blur(5px);
            opacity: 0.55;
            transition: filter 0.15s ease, opacity 0.15s ease;
        }
        .ref-spoiler.is-revealed .ref-spoiler__value {
            filter: none;
            opacity: 1;
        }
        .ref-spoiler--plate {
            min-width: 0 !important;
            gap: 0.35rem;
            padding: 0.2rem 0.45rem !important;
            border-style: dashed !important;
            border-radius: 0 !important;
            font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
            font-size: 0.6875rem;
            line-height: 1.2;
        }
        .ref-spoiler--plate .ref-spoiler__label {
            flex-shrink: 0;
            font-family: Sora, ui-sans-serif, system-ui, sans-serif;
            font-size: 0.625rem;
            letter-spacing: 0.12em;
            text-transform: uppercase;
            color: rgba(154, 163, 173, 0.8);
        }
        .ref-spoiler--plate.is-revealed {
            cursor: pointer !important;
            user-select: text !important;
            border-color: rgba(42, 48, 56, 0.95) !important;
            background: rgba(10, 11, 13, 0.55) !important;
            padding: 0.2rem 0.45rem !important;
        }
        .ref-menu {
            position: relative;
            z-index: 50;
        }
        .ref-menu__toggle {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            width: 2rem;
            height: 2rem;
            padding: 0;
            border: 0;
            background: transparent;
            color: #9aa3ad;
            cursor: pointer;
            transition: color 0.12s ease;
        }
        .ref-menu__toggle:hover,
        .ref-menu__toggle:focus-visible {
            color: #e8eaed;
            outline: none;
        }
        .ref-menu__panel {
            position: absolute;
            top: calc(100% + 0.5rem);
            right: 0;
            z-index: 60;
            min-width: 11rem;
            border: 1px solid #2a3038;
            background: #0c0e11;
            box-shadow: 0 12px 28px rgba(0, 0, 0, 0.45);
            opacity: 0;
            visibility: hidden;
            transform: translateY(-0.35rem);
            transition: opacity 0.16s ease, transform 0.16s ease, visibility 0.16s ease;
            pointer-events: none;
        }
        .ref-menu.is-open .ref-menu__panel {
            opacity: 1;
            visibility: visible;
            transform: translateY(0);
            pointer-events: auto;
        }
        .ref-menu__item,
        .ref-menu__panel a.ref-menu__item,
        .ref-menu__panel button.ref-menu__item {
            display: block;
            box-sizing: border-box;
            width: 100%;
            margin: 0;
            border: 0;
            border-radius: 0;
            background: transparent;
            padding: 0.7rem 0.95rem;
            text-align: left;
            font-family: Sora, ui-sans-serif, system-ui, sans-serif;
            font-size: 0.6875rem;
            letter-spacing: 0.14em;
            text-transform: uppercase;
            color: #9aa3ad;
            text-decoration: none;
            cursor: pointer;
            transition: color 0.12s ease, background-color 0.12s ease;
        }
        .ref-menu__item:hover,
        .ref-menu__item:focus-visible,
        .ref-menu__panel a.ref-menu__item:hover,
        .ref-menu__panel a.ref-menu__item:focus-visible,
        .ref-menu__panel button.ref-menu__item:hover,
        .ref-menu__panel button.ref-menu__item:focus-visible {
            color: #e8eaed !important;
            background-color: rgba(42, 48, 56, 0.55) !important;
            outline: none;
            cursor: pointer;
        }
        .ref-menu__panel form {
            margin: 0;
        }
        .ref-menu__panel form + a,
        .ref-menu__panel a + a,
        .ref-menu__panel a + form,
        .ref-menu__panel form + form {
            border-top: 1px solid rgba(42, 48, 56, 0.85);
        }
    </style>
</head>
<body class="ref-canvas min-h-[100svh] font-sans text-grit-text antialiased">
    <div
        class="ref-wallpaper"
        style="--ref-wallpaper: url('{{ asset('images/brand/refracted-wallpaper.png') }}')"
        aria-hidden="true"
    ></div>

    <header class="relative border-b border-grit-line/60 bg-grit-bg/70 backdrop-blur-sm" style="z-index: 40;">
        <div class="mx-auto flex max-w-4xl items-center gap-4 px-5 py-4 sm:px-8">
            <a href="{{ route('nexus.profile') }}" class="flex items-center gap-3 text-grit-text no-underline">
                <img src="{{ asset('images/brand/refracted-icon.png') }}" alt="" class="h-8 w-8 shrink-0 rounded-lg object-contain">
                <span class="font-display text-lg font-semibold tracking-[-0.02em]">Nexus</span>
            </a>

            <div class="ml-auto flex items-center gap-5">
                <nav class="flex items-center gap-5 text-[11px] uppercase tracking-[0.16em]">
                    <a
                        href="{{ route('nexus.admin') }}"
                        class="{{ ($active ?? '') === 'admin' ? 'text-grit-text' : 'text-grit-mist hover:text-grit-text' }}"
                    >Admin</a>
                    <a
                        href="{{ route('nexus.admin.lookup') }}"
                        class="{{ ($active ?? '') === 'lookup' ? 'text-grit-text' : 'text-grit-mist hover:text-grit-text' }}"
                    >Lookup</a>
                    <a
                        href="{{ route('nexus.admin.gatekeeper') }}"
                        class="{{ ($active ?? '') === 'gatekeeper' ? 'text-grit-text' : 'text-grit-mist hover:text-grit-text' }}"
                    >Gatekeeper</a>
                    <a
                        href="{{ route('nexus.admin.sessions') }}"
                        class="{{ ($active ?? '') === 'sessions' ? 'text-grit-text' : 'text-grit-mist hover:text-grit-text' }}"
                    >Sessions</a>
                </nav>

                <div class="ref-menu" data-ref-menu>
                    <button
                        type="button"
                        class="ref-menu__toggle"
                        aria-label="Menu"
                        aria-expanded="false"
                        aria-haspopup="true"
                        data-ref-menu-toggle
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            width="20"
                            height="20"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1.75"
                            stroke-linecap="round"
                            aria-hidden="true"
                            style="width: 1.25rem; height: 1.25rem;"
                        >
                            <path d="M4 7h16" />
                            <path d="M4 12h16" />
                            <path d="M4 17h16" />
                        </svg>
                    </button>
                    <div class="ref-menu__panel" role="menu" data-ref-menu-panel>
                        <a href="{{ route('nexus.profile') }}" class="ref-menu__item" role="menuitem">Profile</a>
                        <form method="POST" action="{{ route('nexus.logout') }}">
                            @csrf
                            <button type="submit" class="ref-menu__item" role="menuitem">Sign out</button>
                        </form>
                    </div>
                </div>
            </div>
        </div>
    </header>

    <main class="relative mx-auto w-full max-w-4xl space-y-10 px-5 py-12 sm:px-8" style="z-index: 1;">
        @if (session('status'))
            <div class="border border-signal/30 bg-signal/10 px-4 py-3 text-sm text-signal-soft">{{ session('status') }}</div>
        @endif
        @if ($errors->any())
            <div class="border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-200">{{ $errors->first() }}</div>
        @endif

        @yield('content')
    </main>
    <script src="https://unpkg.com/@popperjs/core@2"></script>
    <script src="https://unpkg.com/tippy.js@6"></script>
    <script>
        tippy('a[data-tippy-content]:not([data-ref-spoiler])', {
            theme: 'refracted',
            placement: 'top',
            animation: 'fade',
            arrow: true,
        });
        (function () {
            document.querySelectorAll('[data-ref-menu]').forEach(function (menu) {
                var toggle = menu.querySelector('[data-ref-menu-toggle]');
                if (! toggle) {
                    return;
                }

                function setOpen(open) {
                    menu.classList.toggle('is-open', open);
                    toggle.setAttribute('aria-expanded', open ? 'true' : 'false');
                }

                toggle.addEventListener('click', function (event) {
                    event.stopPropagation();
                    setOpen(! menu.classList.contains('is-open'));
                });

                document.addEventListener('click', function (event) {
                    if (! menu.contains(event.target)) {
                        setOpen(false);
                    }
                });

                document.addEventListener('keydown', function (event) {
                    if (event.key === 'Escape') {
                        setOpen(false);
                    }
                });
            });
        })();
        (function () {
            document.querySelectorAll('[data-ref-spoiler]').forEach(function (node) {
                if (node.dataset.spoilerBound === '1') {
                    return;
                }
                node.dataset.spoilerBound = '1';

                var canToggle = node.hasAttribute('data-ref-spoiler-toggle');
                var tip = null;
                if (typeof tippy === 'function' && node.hasAttribute('data-tippy-content')) {
                    tip = node._tippy || tippy(node, {
                        theme: 'refracted',
                        placement: 'top',
                        animation: 'fade',
                        arrow: true,
                        content: node.getAttribute('data-tippy-content') || 'Click to reveal',
                    });
                }

                function setTip(text) {
                    if (tip && typeof tip.setContent === 'function') {
                        tip.setContent(text);
                    }
                }

                node.addEventListener('click', function () {
                    if (node.classList.contains('is-revealed')) {
                        if (! canToggle) {
                            return;
                        }
                        node.classList.remove('is-revealed');
                        node.setAttribute('aria-expanded', 'false');
                        setTip(node.getAttribute('data-tippy-content') || 'Click to reveal');
                        return;
                    }

                    node.classList.add('is-revealed');
                    node.setAttribute('aria-expanded', 'true');
                    setTip('Click to Hide');
                });

                node.addEventListener('keydown', function (event) {
                    if (event.key !== 'Enter' && event.key !== ' ') {
                        return;
                    }
                    event.preventDefault();
                    node.click();
                });
            });
        })();
    </script>
</body>
</html>
