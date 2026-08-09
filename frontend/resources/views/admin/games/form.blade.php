@extends('admin.layout')

@section('title', $game->exists ? 'Edit game' : 'Add game')

@section('content')
    <p class="font-display text-xs font-semibold uppercase tracking-[0.2em] text-signal">Games</p>
    <h1 class="mt-2 font-display text-3xl font-semibold tracking-[-0.02em]">
        {{ $game->exists ? 'Edit game' : 'Add game' }}
    </h1>

    <form
        method="POST"
        action="{{ $game->exists ? route('admin.games.update', $game) : route('admin.games.store') }}"
        enctype="multipart/form-data"
        class="ref-panel mt-8 max-w-2xl space-y-5 p-6 sm:p-8"
    >
        @csrf
        @if ($game->exists)
            @method('PUT')
        @endif

        <div>
            <label class="mb-2 block text-xs uppercase tracking-[0.14em] text-grit-mist" for="name">Name</label>
            <input id="name" name="name" type="text" value="{{ old('name', $game->name) }}" required class="w-full border border-grit-line bg-grit-bg px-3 py-2.5 text-sm text-grit-text outline-none focus:border-signal">
        </div>

        <div>
            <label class="mb-2 block text-xs uppercase tracking-[0.14em] text-grit-mist" for="status">Status</label>
            <input id="status" name="status" type="text" value="{{ old('status', $game->status) }}" required class="w-full border border-grit-line bg-grit-bg px-3 py-2.5 text-sm text-grit-text outline-none focus:border-signal" placeholder="Available / Coming soon">
        </div>

        <div>
            <label class="mb-2 block text-xs uppercase tracking-[0.14em] text-grit-mist" for="blurb">Short description</label>
            <textarea id="blurb" name="blurb" rows="3" class="w-full border border-grit-line bg-grit-bg px-3 py-2.5 text-sm text-grit-text outline-none focus:border-signal">{{ old('blurb', $game->blurb) }}</textarea>
        </div>

        <div>
            <label class="mb-2 block text-xs uppercase tracking-[0.14em] text-grit-mist" for="url">Link (optional)</label>
            <input id="url" name="url" type="url" value="{{ old('url', $game->url) }}" class="w-full border border-grit-line bg-grit-bg px-3 py-2.5 text-sm text-grit-text outline-none focus:border-signal">
        </div>

        <div class="grid gap-5 sm:grid-cols-2">
            <div>
                <label class="mb-2 block text-xs uppercase tracking-[0.14em] text-grit-mist" for="sort_order">Sort order</label>
                <input id="sort_order" name="sort_order" type="number" min="0" value="{{ old('sort_order', $game->sort_order ?? 0) }}" class="w-full border border-grit-line bg-grit-bg px-3 py-2.5 text-sm text-grit-text outline-none focus:border-signal">
            </div>
            <div class="flex items-end pb-2">
                <label class="inline-flex items-center gap-2 text-sm text-grit-mist">
                    <input type="checkbox" name="is_published" value="1" @checked(old('is_published', $game->is_published ?? true)) class="border-grit-line bg-grit-bg text-signal focus:ring-signal">
                    Published on the site
                </label>
            </div>
        </div>

        <div>
            <label class="mb-2 block text-xs uppercase tracking-[0.14em] text-grit-mist" for="image">Hero image</label>
            @if ($game->imageUrl())
                <img src="{{ $game->imageUrl() }}" alt="" class="mb-3 h-28 w-full max-w-sm object-cover">
            @endif
            <input id="image" name="image" type="file" accept="image/*" class="block w-full text-sm text-grit-mist file:mr-3 file:border-0 file:bg-signal file:px-3 file:py-2 file:text-xs file:font-semibold file:uppercase file:tracking-[0.12em] file:text-white">
        </div>

        <div class="flex items-center gap-3 pt-2">
            <button type="submit" class="bg-signal px-5 py-2.5 font-display text-xs font-semibold uppercase tracking-[0.14em] text-white hover:bg-signal-soft">
                {{ $game->exists ? 'Save changes' : 'Add game' }}
            </button>
            <a href="{{ route('admin.games.index') }}" class="text-sm text-grit-mist underline decoration-grit-line underline-offset-2">Cancel</a>
        </div>
    </form>
@endsection
