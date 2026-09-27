@props([
    'value' => null,
    'label' => null,
])

@php
    $raw = trim((string) ($value ?? ''));
    $empty = $raw === '' || $raw === '—';
@endphp

@if ($empty)
    <span {{ $attributes->merge(['class' => 'text-grit-mist']) }}>—</span>
@else
    <button
        type="button"
        data-ref-spoiler
        data-ref-spoiler-toggle
        data-tippy-content="Click to reveal"
        {{ $attributes->merge([
            'class' => 'ref-spoiler ref-spoiler--plate',
        ]) }}
        aria-expanded="false"
        aria-label="{{ $label ? 'Reveal '.$label : 'Reveal hidden value' }}"
        tabindex="0"
    >
        @if ($label)
            <span class="ref-spoiler__label">{{ $label }}</span>
        @endif
        <span class="ref-spoiler__value">{{ $raw }}</span>
    </button>
@endif
