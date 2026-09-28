{{-- Report window selector; keeps other query params. Expects $range. --}}
<nav class="flex flex-wrap gap-2" aria-label="Report range">
    @foreach (\App\Services\Cdn\CdnMetrics::RANGES as $key => $def)
        <a
            href="{{ request()->fullUrlWithQuery(['range' => $key, 'page' => null]) }}"
            @class([
                'px-3 py-2 text-xs uppercase tracking-[0.12em]',
                'bg-signal text-white' => $key === $range,
                'border border-grit-line text-grit-mist hover:border-signal/40' => $key !== $range,
            ])
            @if ($key === $range) aria-current="true" @endif
        >{{ $def[2] }}</a>
    @endforeach
</nav>
