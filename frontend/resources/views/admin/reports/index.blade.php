@extends('admin.layout')

@section('title', 'Reports')

@section('content')
    <div class="flex flex-wrap items-end justify-between gap-4">
        <div>
            <p class="font-display text-xs font-semibold uppercase tracking-[0.2em] text-signal">Reports</p>
            <h1 class="mt-2 font-display text-3xl font-semibold tracking-[-0.02em]">Crashes &amp; asserts</h1>
            <p class="mt-2 text-sm text-grit-mist">
                Uploaded automatically by game clients. Grouped by the fault the client reported.
            </p>
        </div>
        <dl class="flex gap-6 text-right">
            @foreach (['unresolved' => 'Open', 'resolved' => 'Resolved', 'ignored' => 'Ignored'] as $key => $label)
                <div>
                    <dt class="text-xs uppercase tracking-[0.14em] text-grit-mist">{{ $label }}</dt>
                    <dd class="font-display text-2xl font-semibold tabular-nums">{{ $statusCounts[$key] ?? 0 }}</dd>
                </div>
            @endforeach
        </dl>
    </div>

    <form method="GET" class="ref-panel mt-8 flex flex-wrap items-end gap-4 p-4">
        <label class="flex min-w-48 flex-1 flex-col gap-1.5">
            <span class="text-xs uppercase tracking-[0.14em] text-grit-mist">Search</span>
            <input
                type="search"
                name="q"
                value="{{ $filters['q'] }}"
                placeholder="File, line, address"
                class="border border-grit-line bg-grit-surface px-3 py-2 text-sm text-grit-text placeholder:text-grit-mist focus:border-signal/60 focus:outline-none"
            >
        </label>

        <label class="flex flex-col gap-1.5">
            <span class="text-xs uppercase tracking-[0.14em] text-grit-mist">Status</span>
            <select name="status" class="border border-grit-line bg-grit-surface px-3 py-2 text-sm text-grit-text focus:border-signal/60 focus:outline-none">
                @foreach (['unresolved' => 'Open', 'resolved' => 'Resolved', 'ignored' => 'Ignored', 'all' => 'All'] as $value => $label)
                    <option value="{{ $value }}" @selected($filters['status'] === $value)>{{ $label }}</option>
                @endforeach
            </select>
        </label>

        <label class="flex flex-col gap-1.5">
            <span class="text-xs uppercase tracking-[0.14em] text-grit-mist">Type</span>
            <select name="type" class="border border-grit-line bg-grit-surface px-3 py-2 text-sm text-grit-text focus:border-signal/60 focus:outline-none">
                <option value="">Any</option>
                @foreach ($types as $type)
                    <option value="{{ $type }}" @selected($filters['type'] === $type)>{{ $type }}</option>
                @endforeach
            </select>
        </label>

        <button type="submit" class="bg-signal px-4 py-2.5 font-display text-xs font-semibold uppercase tracking-[0.14em] text-white hover:bg-signal-soft">
            Filter
        </button>
    </form>

    <div class="mt-8 space-y-3">
        @forelse ($issues as $issue)
            <a href="{{ route('admin.reports.show', $issue) }}" class="ref-panel block p-4 transition hover:border-signal/50">
                <div class="flex flex-wrap items-start justify-between gap-4">
                    <div class="min-w-0">
                        <div class="flex items-center gap-2">
                            <span class="border border-grit-line px-2 py-0.5 text-[0.65rem] uppercase tracking-[0.14em] text-grit-mist">
                                {{ $issue->type }}
                            </span>
                            @if ($issue->status !== 'unresolved')
                                <span class="text-[0.65rem] uppercase tracking-[0.14em] text-signal">{{ $issue->status }}</span>
                            @endif
                        </div>
                        <p class="mt-2 truncate font-mono text-sm text-grit-text">{{ $issue->title }}</p>
                        @if ($issue->culprit && $issue->culprit !== $issue->title)
                            <p class="mt-1 truncate font-mono text-xs text-grit-mist">{{ $issue->culprit }}</p>
                        @endif
                    </div>
                    <div class="flex shrink-0 items-center gap-6 text-right">
                        <div>
                            <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">Events</p>
                            <p class="font-display text-lg font-semibold tabular-nums">{{ $issue->events_count }}</p>
                        </div>
                        <div>
                            <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">Last seen</p>
                            <p class="text-sm text-grit-text">{{ $issue->last_seen_at?->diffForHumans() ?? '—' }}</p>
                        </div>
                    </div>
                </div>
            </a>
        @empty
            <p class="text-sm text-grit-mist">
                No reports match.
            </p>
        @endforelse
    </div>

    <div class="mt-8">
        {{ $issues->links() }}
    </div>
@endsection
