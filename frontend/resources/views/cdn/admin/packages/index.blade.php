@extends('cdn.admin.layout')

@section('title', 'Packages')

@section('content')
    <div class="flex flex-wrap items-end justify-between gap-4">
        <div>
            <h1 class="font-display text-3xl font-semibold tracking-[-0.02em]">Packages</h1>
            <p class="mt-2 text-sm text-grit-mist">Prism and Refracted launcher builds, by channel and revision.</p>
        </div>
        <a href="{{ route('cdn.admin.packages.create') }}" class="bg-signal px-4 py-2.5 font-display text-xs font-semibold uppercase tracking-[0.14em] text-white hover:bg-signal-soft">
            New package
        </a>
    </div>

    <div class="mt-8 space-y-3">
        @forelse ($packages as $package)
            <article class="ref-panel flex flex-wrap items-center justify-between gap-3 p-4">
                <div>
                    <a href="{{ route('cdn.admin.packages.show', $package) }}" class="font-display text-lg font-semibold text-signal-soft underline decoration-signal/30">
                        {{ $package->displayName() }}
                    </a>
                    <p class="mt-1 text-xs uppercase tracking-[0.12em] text-grit-mist">
                        {{ $package->status }}
                        @if ($package->is_latest) · latest @endif
                        · {{ $package->artifacts_count }} files
                    </p>
                </div>
                <a href="{{ route('cdn.admin.packages.edit', $package) }}" class="text-sm text-grit-mist underline">Edit</a>
            </article>
        @empty
            <p class="text-sm text-grit-mist">No packages yet.</p>
        @endforelse
    </div>

    <div class="mt-6">{{ $packages->links() }}</div>
@endsection
