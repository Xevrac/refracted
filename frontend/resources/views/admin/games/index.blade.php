@extends('admin.layout')

@section('title', 'Games')

@section('content')
    <div class="flex flex-wrap items-end justify-between gap-4">
        <div>
            <p class="font-display text-xs font-semibold uppercase tracking-[0.2em] text-signal">Games</p>
            <h1 class="mt-2 font-display text-3xl font-semibold tracking-[-0.02em]">Manage titles</h1>
            <p class="mt-2 text-sm text-grit-mist">Drag the grid handle to change order.</p>
        </div>
        <a href="{{ route('admin.games.create') }}" class="bg-signal px-4 py-2.5 font-display text-xs font-semibold uppercase tracking-[0.14em] text-white hover:bg-signal-soft">
            Add game
        </a>
    </div>

    <div
        class="mt-8 space-y-3"
        data-ref-sortable
        data-reorder-url="{{ route('admin.games.reorder') }}"
    >
        @forelse ($games as $game)
            <article
                class="ref-panel flex items-stretch gap-3 p-3 sm:gap-4 sm:p-4"
                data-id="{{ $game->id }}"
            >
                <button
                    type="button"
                    class="ref-drag-handle flex w-8 shrink-0 cursor-grab items-center justify-center text-grit-mist transition hover:text-grit-text active:cursor-grabbing"
                    aria-label="Drag to reorder {{ $game->name }}"
                >
                    <svg class="h-3.5 w-2.5" viewBox="0 0 10 14" fill="currentColor" aria-hidden="true">
                        <rect x="1" y="1" width="2.2" height="2.2" />
                        <rect x="6.8" y="1" width="2.2" height="2.2" />
                        <rect x="1" y="5.9" width="2.2" height="2.2" />
                        <rect x="6.8" y="5.9" width="2.2" height="2.2" />
                        <rect x="1" y="10.8" width="2.2" height="2.2" />
                        <rect x="6.8" y="10.8" width="2.2" height="2.2" />
                    </svg>
                </button>

                <div class="flex min-w-0 flex-1 flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                    <div class="flex min-w-0 items-center gap-4">
                        @if ($game->imageUrl())
                            <img src="{{ $game->imageUrl() }}" alt="" class="h-16 w-24 shrink-0 object-cover">
                        @else
                            <div class="flex h-16 w-24 shrink-0 items-center justify-center bg-grit-surface text-xs text-grit-mist">No image</div>
                        @endif
                        <div class="min-w-0">
                            <p class="font-display text-lg font-semibold tracking-[-0.02em]">{{ $game->name }}</p>
                            <p class="mt-1 text-xs uppercase tracking-[0.14em] text-grit-mist">
                                {{ $game->status }}
                                · {{ $game->is_published ? 'Published' : 'Hidden' }}
                            </p>
                        </div>
                    </div>
                    <div class="flex items-center gap-3 ps-0 sm:ps-0">
                        <a href="{{ route('admin.games.edit', $game) }}" class="text-sm text-signal-soft underline decoration-signal/30 underline-offset-2">Edit</a>
                        <form method="POST" action="{{ route('admin.games.destroy', $game) }}" onsubmit="return confirm('Remove this game?')">
                            @csrf
                            @method('DELETE')
                            <button type="submit" class="text-sm text-red-300 underline decoration-red-400/30 underline-offset-2">Remove</button>
                        </form>
                    </div>
                </div>
            </article>
        @empty
            <p class="text-sm text-grit-mist">No games yet. Add your first title.</p>
        @endforelse
    </div>

    <div class="mt-16 flex flex-wrap items-end justify-between gap-4 border-t border-grit-line pt-12">
        <div>
            <p class="font-display text-xs font-semibold uppercase tracking-[0.2em] text-signal">Communities</p>
            <h2 class="mt-2 font-display text-2xl font-semibold tracking-[-0.02em]">Communities</h2>
            <p class="mt-2 text-sm text-grit-mist">Drag the grid handle to change order.</p>
        </div>
        <a href="{{ route('admin.communities.create') }}" class="border border-grit-line px-4 py-2.5 font-display text-xs font-semibold uppercase tracking-[0.14em] text-grit-text hover:border-signal/50 hover:bg-signal/10">
            Add community
        </a>
    </div>

    <div
        class="mt-8 space-y-3"
        data-ref-sortable
        data-reorder-url="{{ route('admin.communities.reorder') }}"
    >
        @forelse ($communities as $community)
            <article
                class="ref-panel flex items-stretch gap-3 p-3 sm:gap-4 sm:p-4"
                data-id="{{ $community->id }}"
            >
                <button
                    type="button"
                    class="ref-drag-handle flex w-8 shrink-0 cursor-grab items-center justify-center text-grit-mist transition hover:text-grit-text active:cursor-grabbing"
                    aria-label="Drag to reorder {{ $community->name }}"
                >
                    <svg class="h-3.5 w-2.5" viewBox="0 0 10 14" fill="currentColor" aria-hidden="true">
                        <rect x="1" y="1" width="2.2" height="2.2" />
                        <rect x="6.8" y="1" width="2.2" height="2.2" />
                        <rect x="1" y="5.9" width="2.2" height="2.2" />
                        <rect x="6.8" y="5.9" width="2.2" height="2.2" />
                        <rect x="1" y="10.8" width="2.2" height="2.2" />
                        <rect x="6.8" y="10.8" width="2.2" height="2.2" />
                    </svg>
                </button>

                <div class="flex min-w-0 flex-1 flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                    <div class="flex min-w-0 items-center gap-4">
                        @if ($community->imageUrl())
                            <img src="{{ $community->imageUrl() }}" alt="" class="h-16 w-24 shrink-0 object-cover">
                        @endif
                        <div class="min-w-0">
                            <p class="font-display text-lg font-semibold tracking-[-0.02em]">{{ $community->name }}</p>
                            <p class="mt-1 text-xs uppercase tracking-[0.14em] text-signal">{{ $community->tagline }}</p>
                        </div>
                    </div>
                    <div class="flex items-center gap-3">
                        <a href="{{ route('admin.communities.edit', $community) }}" class="text-sm text-signal-soft underline decoration-signal/30 underline-offset-2">Edit</a>
                        <form method="POST" action="{{ route('admin.communities.destroy', $community) }}" onsubmit="return confirm('Remove this community?')">
                            @csrf
                            @method('DELETE')
                            <button type="submit" class="text-sm text-red-300 underline decoration-red-400/30 underline-offset-2">Remove</button>
                        </form>
                    </div>
                </div>
            </article>
        @empty
            <p class="text-sm text-grit-mist">No communities yet.</p>
        @endforelse
    </div>
@endsection
