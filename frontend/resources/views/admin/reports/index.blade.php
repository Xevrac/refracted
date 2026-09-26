@extends('admin.layout')

@section('title', 'Sentry')

@php
    $statuses = ['unresolved' => 'Open', 'resolved' => 'Resolved', 'ignored' => 'Ignored', 'all' => 'All'];
    $query = array_filter([
        'q' => $filters['q'] ?: null,
        'type' => $filters['type'] ?: null,
        'game' => $filters['game'] ?: null,
    ]);
    $tabClass = 'px-3 py-1.5 text-xs uppercase tracking-[0.12em]';
@endphp

@section('content')
    <div class="flex flex-wrap items-end justify-between gap-4">
        <h1 class="font-display text-3xl font-semibold tracking-[-0.02em]">Sentry</h1>
    </div>

    <div class="mt-6 flex flex-wrap items-center justify-between gap-4">
        <div class="flex items-center gap-4">
            <a
                href="{{ route('admin.reports.index', ['tab' => 'reporting']) }}"
                class="{{ $tabClass }} {{ $tab === 'reporting' ? 'bg-signal text-white' : 'text-grit-mist hover:text-grit-text' }}"
            >Reporting</a>
            <a
                href="{{ route('admin.reports.index', ['tab' => 'issues']) }}"
                class="{{ $tabClass }} {{ $tab === 'issues' ? 'bg-signal text-white' : 'text-grit-mist hover:text-grit-text' }}"
            >Issues</a>
        </div>
        <div class="ref-bell-slot relative" data-sentry-bell data-inbox-url="{{ route('admin.reports.inbox') }}">
            <button type="button" class="ref-bell" aria-label="Unread incidents" aria-expanded="false" data-bell-toggle>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                    <path d="M6 9a6 6 0 1 1 12 0c0 7 3 7 3 9H3c0-2 3-2 3-9" stroke="currentColor" stroke-width="1.6"/>
                    <path d="M10 20a2 2 0 0 0 4 0" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
                </svg>
                <span class="ref-bell-count" data-bell-count hidden>0</span>
            </button>
            <div class="ref-bell-menu" data-bell-menu hidden>
                <div class="ref-bell-head">
                    <div class="ref-bell-head-title">
                        <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">Unread</p>
                        <span class="ref-bell-rule" aria-hidden="true"></span>
                        <button
                            type="button"
                            class="ref-bell-mark"
                            data-mark-all="{{ route('admin.reports.read-all') }}"
                            aria-label="Mark all as read"
                            disabled
                        >
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                                <path d="M2.5 13 7 17.5 16 7" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
                                <path d="M9.5 13 14 17.5 22.5 7" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
                            </svg>
                        </button>
                    </div>
                    <p class="text-xs tabular-nums text-grit-mist" data-bell-heading-count></p>
                </div>
                <div class="ref-bell-list" data-bell-list>
                    <p class="px-4 py-5 text-sm text-grit-mist" data-bell-empty>No unread incidents.</p>
                </div>
            </div>
        </div>
    </div>

    @if ($tab === 'reporting')
        <div class="mt-6 flex flex-wrap gap-4">
            @foreach ([
                'Open issues' => $report['open'],
                'Issues' => $report['issues'],
                'Events' => $report['events'],
                'Games' => $report['games'],
            ] as $label => $value)
                <div class="ref-panel min-w-40 flex-1 px-4 py-4">
                    <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">{{ $label }}</p>
                    <p class="mt-2 font-display text-3xl font-semibold tabular-nums">{{ $value }}</p>
                </div>
            @endforeach
        </div>

        <div class="ref-panel mt-6 overflow-hidden">
            <h2 class="border-b border-grit-line px-4 py-3 font-display text-lg font-semibold">By game</h2>
            <div class="overflow-x-auto">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="border-b border-grit-line text-left text-xs uppercase tracking-[0.14em] text-grit-mist">
                            <th class="px-4 py-2 font-medium">Game</th>
                            <th class="px-4 py-2 text-right font-medium">Open</th>
                            <th class="px-4 py-2 text-right font-medium">Issues</th>
                            <th class="px-4 py-2 text-right font-medium">Events</th>
                            <th class="px-4 py-2 text-right font-medium">Last seen</th>
                        </tr>
                    </thead>
                    <tbody>
                        @forelse ($report['byGame'] as $row)
                            @php $slug = $row->game ?: 'unknown'; @endphp
                            <tr class="border-b border-grit-line last:border-b-0">
                                <td class="px-4 py-3">
                                    <a href="{{ route('admin.reports.index', ['tab' => 'issues', 'game' => $slug]) }}" class="text-grit-text hover:text-signal">
                                        {{ \App\Support\GameReport::gameLabel($slug) }}
                                    </a>
                                </td>
                                <td class="px-4 py-3 text-right tabular-nums">{{ (int) $row->open_issues }}</td>
                                <td class="px-4 py-3 text-right tabular-nums">{{ (int) $row->issues }}</td>
                                <td class="px-4 py-3 text-right tabular-nums">{{ (int) $row->events }}</td>
                                <td class="px-4 py-3 text-right text-xs text-grit-mist">{{ $row->last_seen_at?->diffForHumans() ?? '—' }}</td>
                            </tr>
                        @empty
                            <tr>
                                <td colspan="5" class="px-4 py-8 text-sm text-grit-mist">No reports yet.</td>
                            </tr>
                        @endforelse
                    </tbody>
                </table>
            </div>
        </div>

        <div class="ref-panel mt-6 overflow-hidden">
            <h2 class="border-b border-grit-line px-4 py-3 font-display text-lg font-semibold">By type</h2>
            <div class="overflow-x-auto">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="border-b border-grit-line text-left text-xs uppercase tracking-[0.14em] text-grit-mist">
                            <th class="px-4 py-2 font-medium">Type</th>
                            <th class="px-4 py-2 text-right font-medium">Issues</th>
                            <th class="px-4 py-2 text-right font-medium">Events</th>
                        </tr>
                    </thead>
                    <tbody>
                        @forelse ($report['byType'] as $row)
                            <tr class="border-b border-grit-line last:border-b-0">
                                <td class="px-4 py-3 text-xs uppercase tracking-[0.08em] text-grit-mist">
                                    <a href="{{ route('admin.reports.index', ['tab' => 'issues', 'type' => $row->type]) }}" class="hover:text-signal">{{ $row->type }}</a>
                                </td>
                                <td class="px-4 py-3 text-right tabular-nums">{{ (int) $row->issues }}</td>
                                <td class="px-4 py-3 text-right tabular-nums">{{ (int) $row->events }}</td>
                            </tr>
                        @empty
                            <tr>
                                <td colspan="3" class="px-4 py-8 text-sm text-grit-mist">No reports yet.</td>
                            </tr>
                        @endforelse
                    </tbody>
                </table>
            </div>
        </div>
    @else
        <div class="ref-panel mt-6 overflow-hidden">
            <form method="GET" class="flex flex-wrap items-center gap-4 border-b border-grit-line px-4 py-3">
                <input type="hidden" name="tab" value="issues">
                <div class="flex flex-wrap items-center gap-4">
                    @foreach ($statuses as $value => $label)
                        <a
                            href="{{ route('admin.reports.index', $query + ['tab' => 'issues', 'status' => $value]) }}"
                            class="px-3 py-1.5 text-xs uppercase tracking-[0.12em] {{ $filters['status'] === $value ? 'bg-signal text-white' : 'text-grit-mist hover:text-grit-text' }}"
                        >
                            {{ $label }}@if ($value !== 'all') {{ $statusCounts[$value] ?? 0 }}@endif
                        </a>
                    @endforeach
                </div>

                <input type="hidden" name="status" value="{{ $filters['status'] }}">
                <input
                    type="search"
                    name="q"
                    value="{{ $filters['q'] }}"
                    placeholder="Search"
                    class="min-w-40 flex-1 border border-grit-line bg-grit-surface px-3 py-1.5 text-sm text-grit-text placeholder:text-grit-mist focus:border-signal/60 focus:outline-none"
                >
                <select
                    name="game"
                    onchange="this.form.requestSubmit()"
                    class="border border-grit-line bg-grit-surface px-2 py-1.5 text-sm text-grit-text focus:border-signal/60 focus:outline-none"
                >
                    <option value="">Any game</option>
                    @foreach ($games as $slug)
                        <option value="{{ $slug }}" @selected($filters['game'] === $slug)>{{ \App\Support\GameReport::gameLabel($slug) }}</option>
                    @endforeach
                </select>
                <select
                    name="type"
                    onchange="this.form.requestSubmit()"
                    class="border border-grit-line bg-grit-surface px-2 py-1.5 text-sm text-grit-text focus:border-signal/60 focus:outline-none"
                >
                    <option value="">Any type</option>
                    @foreach ($types as $type)
                        <option value="{{ $type }}" @selected($filters['type'] === $type)>{{ $type }}</option>
                    @endforeach
                </select>
            </form>

            <div class="overflow-x-auto">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="border-b border-grit-line text-left text-xs uppercase tracking-[0.14em] text-grit-mist">
                            <th class="px-4 py-2 font-medium">#</th>
                            <th class="px-4 py-2 font-medium">Game</th>
                            <th class="px-4 py-2 font-medium">Type</th>
                            <th class="px-4 py-2 font-medium">Issue</th>
                            <th class="px-4 py-2 font-medium">Status</th>
                            <th class="px-4 py-2 text-right font-medium">Events</th>
                            <th class="px-4 py-2 text-right font-medium">Last seen</th>
                        </tr>
                    </thead>
                    <tbody
                        id="sentry-issues"
                        data-page="{{ $issues->currentPage() }}"
                        data-status="{{ $filters['status'] }}"
                        data-game="{{ $filters['game'] }}"
                        data-type="{{ $filters['type'] }}"
                        data-query="{{ $filters['q'] }}"
                    >
                        @forelse ($issues as $issue)
                            <tr class="border-b border-grit-line last:border-b-0 hover:bg-signal/10" data-issue-id="{{ $issue->id }}">
                                <td class="ref-issue-id px-4 py-3 font-mono text-xs text-grit-mist">
                                    @if ($unreadIds->has($issue->id))
                                        <button
                                            type="button"
                                            class="ref-unread"
                                            data-read-url="{{ route('admin.reports.read', $issue) }}"
                                            aria-label="Mark as read"
                                        ></button>
                                    @endif
                                    <a href="{{ route('admin.reports.show', $issue) }}">#{{ $issue->id }}</a>
                                </td>
                                <td class="px-4 py-3 text-xs text-grit-mist">{{ \App\Support\GameReport::gameLabel($issue->game ?: 'unknown') }}</td>
                                <td class="px-4 py-3 text-xs uppercase tracking-[0.08em] text-grit-mist">{{ $issue->type }}</td>
                                <td class="max-w-md px-4 py-3">
                                    <a href="{{ route('admin.reports.show', $issue) }}" class="block truncate font-medium text-grit-text hover:text-signal">{{ $issue->title }}</a>
                                    @if ($issue->culprit && $issue->culprit !== $issue->title)
                                        <p class="truncate font-mono text-xs text-grit-mist">{{ $issue->culprit }}</p>
                                    @endif
                                </td>
                                <td class="px-4 py-3 text-xs uppercase tracking-[0.08em] {{ $issue->status === 'unresolved' ? 'text-signal' : 'text-grit-mist' }}" data-issue-status>
                                    {{ $issue->status === 'unresolved' ? 'Open' : $issue->status }}
                                </td>
                                <td class="px-4 py-3 text-right tabular-nums">{{ $issue->events_count }}</td>
                                <td class="px-4 py-3 text-right text-xs text-grit-mist">{{ $issue->last_seen_at?->diffForHumans() ?? '—' }}</td>
                            </tr>
                        @empty
                            <tr data-empty>
                                <td colspan="7" class="px-4 py-8 text-sm text-grit-mist">No issues.</td>
                            </tr>
                        @endforelse
                    </tbody>
                </table>
            </div>
        </div>

        <div class="mt-6">
            {{ $issues->links() }}
        </div>
    @endif
@endsection
