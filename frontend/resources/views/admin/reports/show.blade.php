@extends('admin.layout')

@section('title', $issue->title)

@section('content')
    <div class="flex flex-wrap items-end justify-between gap-4">
        <div class="min-w-0 flex-1">
            <p class="font-display text-xs font-semibold uppercase tracking-[0.2em] text-signal">
                <a href="{{ route('admin.reports.index') }}" class="hover:text-signal-soft">Sentry</a>
            </p>
            <h1 class="ref-title mt-2 font-display text-3xl font-semibold tracking-[-0.02em]">{{ $issue->title }}</h1>
            <p class="mt-2 font-mono text-sm text-grit-mist">
                {{ \App\Support\GameReport::gameLabel($issue->game ?: 'unknown') }}
                · {{ $issue->type }}
                @if ($issue->culprit)
                    · {{ $issue->culprit }}
                @endif
                · {{ $issue->events_count }} event{{ $issue->events_count === 1 ? '' : 's' }}
            </p>
        </div>
        <div class="flex shrink-0 items-center gap-2">
            <a
                href="{{ route('admin.reports.index', ['tab' => 'issues']) }}"
                class="flex h-9 w-9 items-center justify-center border border-grit-line text-grit-text hover:text-signal"
                title="Back"
                aria-label="Back to issues"
            >←</a>
            <button
                type="button"
                id="copy-issue"
                class="flex h-9 w-9 items-center justify-center border border-grit-line text-grit-text hover:text-signal"
                aria-label="Copy issue to clipboard"
            >
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                    <rect x="9" y="9" width="11" height="11" rx="1"></rect>
                    <path d="M5 15V5h10"></path>
                </svg>
            </button>
            <form
                method="POST"
                action="{{ route('admin.reports.destroy', $issue) }}"
                onsubmit="return confirm('Delete this issue and all of its events?');"
            >
                @csrf
                @method('DELETE')
                <button
                    type="submit"
                    class="flex h-9 w-9 items-center justify-center border border-grit-line text-grit-text hover:border-signal hover:text-signal"
                    title="Delete issue"
                    aria-label="Delete issue"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                        <path d="M4 7h16"></path>
                        <path d="M9 7V5h6v2"></path>
                        <path d="M6 7l1 13h10l1-13"></path>
                        <path d="M10 11v6M14 11v6"></path>
                    </svg>
                </button>
            </form>
        </div>
    </div>

    <form method="POST" action="{{ route('admin.reports.update', $issue) }}" class="ref-panel mt-8 p-4">
        @csrf
        @method('PUT')
        <div class="flex flex-wrap items-end gap-4">
            <label class="flex flex-col gap-1.5">
                <span class="text-xs uppercase tracking-[0.14em] text-grit-mist">Status</span>
                <select name="status" class="border border-grit-line bg-grit-surface px-3 py-2 text-sm text-grit-text focus:border-signal/60 focus:outline-none">
                    @foreach (\App\Models\ReportIssue::STATUSES as $status)
                        <option value="{{ $status }}" @selected($issue->status === $status)>{{ $status }}</option>
                    @endforeach
                </select>
            </label>
            <button type="submit" class="bg-signal px-4 py-2.5 font-display text-xs font-semibold uppercase tracking-[0.14em] text-white hover:bg-signal-soft">
                Update
            </button>
        </div>
        <label class="mt-8 flex flex-col gap-1.5">
            <span class="text-xs uppercase tracking-[0.14em] text-grit-mist">Notes</span>
            <textarea
                name="notes"
                rows="3"
                class="border border-grit-line bg-grit-surface px-3 py-2 text-sm text-grit-text focus:border-signal/60 focus:outline-none"
            >{{ $issue->notes }}</textarea>
        </label>
    </form>

    @if ($event)
        <section class="ref-panel mt-8 space-y-4 p-4">
            <div class="flex flex-wrap items-center justify-between gap-3">
                <h2 class="font-display text-lg font-semibold">Event #{{ $event->id }}</h2>
                <div class="flex gap-3 text-sm">
                    @if ($event->hasScreenshot())
                        <a href="{{ route('admin.reports.screenshot', $event) }}" class="text-signal-soft underline decoration-signal/30 underline-offset-2">Screenshot</a>
                    @endif
                    @if (filled($event->raw_path))
                        <a href="{{ route('admin.reports.raw', $event) }}" class="text-signal-soft underline decoration-signal/30 underline-offset-2">Raw XML</a>
                    @endif
                </div>
            </div>
            @if ($event->hasScreenshot())
                <img
                    src="{{ route('admin.reports.screenshot', $event) }}"
                    alt="Screenshot"
                    style="display:block;max-width:100%;max-height:28rem"
                >
            @endif
            <dl class="grid gap-3 text-sm sm:grid-cols-2">
                @foreach ([
                    'Received' => $event->received_at?->toDateTimeString(),
                    'Client time' => $event->client_created_at?->toDateTimeString(),
                    'Session' => $event->session_id,
                    'SKU' => $event->sku,
                    'Build' => $event->build_signature,
                    'Server' => $event->server_name,
                    'IP' => $event->remote_ip,
                ] as $label => $value)
                    @if (filled($value))
                        <div>
                            <dt class="text-xs uppercase tracking-[0.14em] text-grit-mist">{{ $label }}</dt>
                            @if ($label === 'IP')
                                <dd class="mt-1">
                                    <button
                                        type="button"
                                        class="ref-spoiler font-mono text-grit-text"
                                        data-ref-spoiler
                                        aria-expanded="false"
                                        aria-label="Hidden IP address. Click to reveal."
                                        title="Click to reveal"
                                    >
                                        <span class="ref-spoiler__value">{{ $value }}</span>
                                        <span class="ref-spoiler__hint" aria-hidden="true">Click to reveal</span>
                                    </button>
                                </dd>
                            @else
                                <dd class="mt-1 font-mono text-grit-text">{{ $value }}</dd>
                            @endif
                        </div>
                    @endif
                @endforeach
            </dl>
            @if (filled($event->context_data))
                <div>
                    <h3 class="text-xs uppercase tracking-[0.14em] text-grit-mist">Context</h3>
                    <pre class="ref-pre mt-2 font-mono text-xs text-grit-text">{{ $event->context_data }}</pre>
                </div>
            @endif
            @if ($event->stackFrames())
                <div>
                    <h3 class="text-xs uppercase tracking-[0.14em] text-grit-mist">Callstack</h3>
                    <pre class="ref-pre mt-2 font-mono text-xs text-grit-text">{{ implode("\n", $event->stackFrames()) }}</pre>
                </div>
            @endif
            @if (filled($event->prism_log))
                <div>
                    <h3 class="text-xs uppercase tracking-[0.14em] text-grit-mist">Prism</h3>
                    <pre class="ref-pre mt-2 font-mono text-xs text-grit-text">{{ $event->prism_log }}</pre>
                </div>
            @endif
        </section>
    @endif

    <div class="mt-8 space-y-2">
        @foreach ($events as $row)
            <a
                href="{{ route('admin.reports.show', ['issue' => $issue, 'event' => $row->id]) }}"
                class="ref-panel block px-4 py-3 text-sm {{ $event && $event->id === $row->id ? 'border-signal/50' : '' }}"
            >
                <span class="font-mono">#{{ $row->id }}</span>
                <span class="text-grit-mist">{{ $row->received_at?->diffForHumans() }}</span>
                @if ($row->build_signature)
                    <span class="text-grit-mist">· {{ $row->build_signature }}</span>
                @endif
            </a>
        @endforeach
    </div>

    <div class="mt-8">
        {{ $events->links() }}
    </div>

    @php
        $copy = [
            $issue->title,
            \App\Support\GameReport::gameLabel($issue->game ?: 'unknown'),
            $issue->type,
            $issue->status,
        ];
        if ($issue->culprit) {
            $copy[] = $issue->culprit;
        }
        if (filled($issue->notes)) {
            $copy[] = "Notes:\n".$issue->notes;
        }
        if ($event) {
            foreach ([
                'Received' => $event->received_at?->toDateTimeString(),
                'Session' => $event->session_id,
                'SKU' => $event->sku,
                'Build' => $event->build_signature,
                'IP' => $event->remote_ip,
            ] as $label => $value) {
                if (filled($value)) {
                    $copy[] = $label.': '.$value;
                }
            }
            if (filled($event->context_data)) {
                $copy[] = "Context:\n".$event->context_data;
            }
            if ($frames = $event->stackFrames()) {
                $copy[] = "Callstack:\n".implode("\n", $frames);
            }
            if (filled($event->prism_log)) {
                $copy[] = "Prism:\n".$event->prism_log;
            }
        }
    @endphp
    <link rel="stylesheet" href="https://unpkg.com/tippy.js@6/dist/tippy.css">
    <script src="https://unpkg.com/@popperjs/core@2"></script>
    <script src="https://unpkg.com/tippy.js@6"></script>
    <style>
        .tippy-box[data-theme~='refracted'] {
            background: #121417;
            color: #e8eaed;
            border: 1px solid #2a3038;
            font-family: Sora, ui-sans-serif, system-ui, sans-serif;
            font-size: 12px;
            padding: 2px 2px;
        }
        .tippy-box[data-theme~='refracted'][data-placement^='top'] > .tippy-arrow::before {
            border-top-color: #2a3038;
        }

        /* Self-contained so spoilers work before a Vite rebuild ships. */
        .ref-spoiler {
            position: relative;
            display: inline-flex;
            min-width: 11.5rem;
            max-width: 100%;
            align-items: center;
            justify-content: center;
            margin: 0;
            padding: 0.25rem 0.65rem;
            border: 1px solid rgba(42, 48, 56, 0.95);
            border-radius: 0.375rem;
            background: rgba(10, 11, 13, 0.92);
            cursor: pointer;
            user-select: none;
            vertical-align: baseline;
            overflow: hidden;
            white-space: nowrap;
        }
        .ref-spoiler:focus-visible {
            outline: 1px solid rgba(74, 163, 255, 0.7);
            outline-offset: 2px;
        }
        .ref-spoiler__value {
            filter: blur(5px);
            opacity: 0.55;
        }
        .ref-spoiler__hint {
            position: absolute;
            inset: 0;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 0 0.5rem;
            background: rgba(18, 20, 23, 0.72);
            color: rgba(154, 163, 173, 0.25);
            font-family: Sora, ui-sans-serif, system-ui, sans-serif;
            font-size: 0.65rem;
            font-weight: 500;
            letter-spacing: 0.14em;
            text-transform: uppercase;
            white-space: nowrap;
            pointer-events: none;
        }
        .ref-spoiler.is-revealed {
            cursor: text;
            user-select: text;
            border-color: transparent;
            background: transparent;
            padding-inline: 0;
        }
        .ref-spoiler.is-revealed .ref-spoiler__value {
            filter: none;
            opacity: 1;
        }
        .ref-spoiler.is-revealed .ref-spoiler__hint {
            display: none;
        }
    </style>
    <script>
        document.querySelectorAll('[data-ref-spoiler]').forEach(function (node) {
            const reveal = function () {
                if (node.classList.contains('is-revealed')) {
                    return;
                }
                node.classList.add('is-revealed');
                node.setAttribute('aria-expanded', 'true');
                node.removeAttribute('tabindex');
            };
            node.addEventListener('click', reveal);
            node.addEventListener('keydown', function (event) {
                if (event.key !== 'Enter' && event.key !== ' ') {
                    return;
                }
                event.preventDefault();
                reveal();
            });
        });

        document.getElementById('copy-issue')?.addEventListener('click', function () {
            const text = @json(implode("\n\n", $copy));
            const button = this;
            const tip = button._copiedTip || (button._copiedTip = tippy(button, {
                content: 'Copied to clipboard!',
                theme: 'refracted',
                trigger: 'manual',
                placement: 'top',
                animation: 'fade',
                arrow: true,
                hideOnClick: false,
            }));
            const done = () => {
                tip.show();
                window.clearTimeout(button._copiedTimer);
                button._copiedTimer = window.setTimeout(() => tip.hide(), 1400);
            };
            if (navigator.clipboard && navigator.clipboard.writeText) {
                navigator.clipboard.writeText(text).then(done);
                return;
            }
            const area = document.createElement('textarea');
            area.value = text;
            document.body.appendChild(area);
            area.select();
            document.execCommand('copy');
            area.remove();
            done();
        });
    </script>
@endsection
