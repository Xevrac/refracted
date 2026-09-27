@extends('cdn.admin.layout')

@section('title', $package->exists ? 'Edit package' : 'New package')

@section('content')
    <h1 class="font-display text-3xl font-semibold tracking-[-0.02em]">
        {{ $package->exists ? 'Edit package' : 'New package' }}
    </h1>

    <form
        method="POST"
        action="{{ $package->exists ? route('cdn.admin.packages.update', $package) : route('cdn.admin.packages.store') }}"
        class="ref-panel mt-8 max-w-xl space-y-4 p-5"
    >
        @csrf
        @if ($package->exists)
            @method('PUT')
        @endif

        @php($kind = old('kind', $package->kind ?? 'prism'))
        <label class="block text-sm">
            <span class="text-grit-mist">Type</span>
            <select id="cdn-kind" name="kind" class="mt-1 w-full border border-grit-line bg-grit-surface px-3 py-2">
                <option value="prism" @selected($kind === 'prism')>Prism (per title)</option>
                <option value="refracted" @selected($kind === 'refracted')>Refracted (launcher)</option>
            </select>
        </label>

        <label id="cdn-title-field" class="block text-sm" @if ($kind === 'refracted') hidden @endif>
            <span class="text-grit-mist">Title</span>
            @php($titleId = old('title_id', $package->isRefracted() ? '' : $package->title_id))
            <select name="title_id" class="mt-1 w-full border border-grit-line bg-grit-surface px-3 py-2">
                @foreach (config('cdn.titles') as $id => $name)
                    <option value="{{ $id }}" @selected($titleId === $id)>{{ $name }} ({{ $id }})</option>
                @endforeach
                @if ($titleId && ! array_key_exists($titleId, config('cdn.titles')))
                    <option value="{{ $titleId }}" selected disabled>{{ $titleId }} (unknown — pick a title)</option>
                @endif
            </select>
        </label>

        <label class="block text-sm">
            <span class="text-grit-mist">Channel</span>
            <select name="channel" class="mt-1 w-full border border-grit-line bg-grit-surface px-3 py-2">
                @foreach (config('cdn.channels') as $ch)
                    <option value="{{ $ch }}" @selected(old('channel', $package->channel) === $ch)>{{ $ch }}</option>
                @endforeach
            </select>
        </label>

        <label class="block text-sm">
            <span class="text-grit-mist">Version</span>
            <input name="version" value="{{ old('version', $package->version) }}" class="mt-1 w-full border border-grit-line bg-grit-surface px-3 py-2" required>
            @if ($package->exists)
                <span class="mt-1 block text-xs text-grit-mist">Revision {{ $package->revision }} (auto on create)</span>
            @endif
        </label>

        <label class="block text-sm">
            <span class="text-grit-mist">Label</span>
            <input name="label" value="{{ old('label', $package->label) }}" class="mt-1 w-full border border-grit-line bg-grit-surface px-3 py-2" placeholder="optional">
        </label>

        <label class="block text-sm">
            <span class="text-grit-mist">Notes</span>
            <textarea name="notes" rows="4" class="mt-1 w-full border border-grit-line bg-grit-surface px-3 py-2">{{ old('notes', $package->notes) }}</textarea>
        </label>

        <button type="submit" class="bg-signal px-4 py-2.5 font-display text-xs font-semibold uppercase tracking-[0.14em] text-white hover:bg-signal-soft">
            Save
        </button>
    </form>

    <script>
        document.getElementById('cdn-kind')?.addEventListener('change', (e) => {
            document.getElementById('cdn-title-field').hidden = e.target.value === 'refracted';
        });
    </script>
@endsection
