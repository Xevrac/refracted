@extends('cdn.admin.layout')

@section('title', 'Dashboard')

@section('content')
    <div class="flex flex-wrap items-end justify-between gap-4">
        <div>
            <h1 class="font-display text-3xl font-semibold tracking-[-0.02em]">Throughput</h1>
            <p class="mt-2 text-sm text-grit-mist">CDN requests from launchers · last {{ $snapshot['range_label'] }} · auto-refresh 5s</p>
        </div>
        <a href="{{ route('cdn.admin.packages.create') }}" class="bg-signal px-4 py-2.5 font-display text-xs font-semibold uppercase tracking-[0.14em] text-white hover:bg-signal-soft">
            New package
        </a>
    </div>

    <div class="mt-6">
        @include('cdn.admin.partials.range', ['range' => $snapshot['range']])
    </div>

    <div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-3" id="cdn-live-cards">
        <div class="ref-panel p-4">
            <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">Requests ({{ $snapshot['range'] }})</p>
            <p class="mt-2 font-display text-3xl font-semibold" data-k="requests">{{ number_format($snapshot['totals']['requests']) }}</p>
        </div>
        <div class="ref-panel p-4">
            <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">Bytes ({{ $snapshot['range'] }})</p>
            <p class="mt-2 font-display text-3xl font-semibold" data-k="bytes_out">{{ number_format($snapshot['totals']['bytes_out']) }}</p>
        </div>
        <div class="ref-panel p-4">
            <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">≈ B/s ({{ $snapshot['range'] }})</p>
            <p class="mt-2 font-display text-3xl font-semibold" data-k="approx_bytes_per_sec">{{ number_format($snapshot['live']['approx_bytes_per_sec']) }}</p>
        </div>
        <div class="ref-panel p-4">
            <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">Downloads ({{ $snapshot['range'] }})</p>
            <p class="mt-2 font-display text-3xl font-semibold">
                <span data-k="downloads_range">{{ number_format($snapshot['downloads']['range']) }}</span>
                <span class="text-grit-mist">/</span>
                <span data-k="downloads_all_time" title="All time">{{ number_format($snapshot['downloads']['all_time']) }}</span>
            </p>
            <p class="mt-1 text-xs text-grit-mist">range / all time</p>
        </div>
        <div class="ref-panel p-4">
            <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">2xx / 4xx / 5xx ({{ $snapshot['range'] }})</p>
            <p class="mt-2 font-display text-3xl font-semibold">
                <span data-k="status_2xx">{{ number_format($snapshot['totals']['status_2xx']) }}</span>
                <span class="text-grit-mist">/</span>
                <span data-k="status_4xx">{{ number_format($snapshot['totals']['status_4xx']) }}</span>
                <span class="text-grit-mist">/</span>
                <span data-k="status_5xx">{{ number_format($snapshot['totals']['status_5xx']) }}</span>
            </p>
        </div>
        <div class="ref-panel p-4">
            <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">Published / draft</p>
            <p class="mt-2 font-display text-3xl font-semibold">
                <span data-k="published">{{ $snapshot['packages']['published'] }}</span>
                <span class="text-grit-mist">/</span>
                <span data-k="draft">{{ $snapshot['packages']['draft'] }}</span>
            </p>
        </div>
    </div>

    <div class="ref-panel mt-8 p-4">
        <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">Requests ({{ $snapshot['range'] }}) · downloads dashed · {{ $snapshot['bucket_seconds'] >= 3600 ? ($snapshot['bucket_seconds'] / 3600).' h' : ($snapshot['bucket_seconds'] / 60).' min' }} buckets</p>
        <canvas id="cdn-chart" class="mt-4 h-48 w-full" height="192"></canvas>
        <p class="mt-2 flex flex-wrap justify-between gap-2 text-xs text-grit-mist">
            <span id="cdn-series-start">{{ $snapshot['series'][0]['t'] ?? '' }}</span>
            <span>Total: <span id="cdn-total-req">{{ number_format($snapshot['totals']['requests']) }}</span> requests · <span id="cdn-total-bytes">{{ number_format($snapshot['totals']['bytes_out']) }}</span> bytes · UTC</span>
            <span id="cdn-series-end">{{ $snapshot['series'][count($snapshot['series']) - 1]['t'] ?? '' }}</span>
        </p>
    </div>

    <div class="mt-10">
        <h2 class="font-display text-xl font-semibold">Downloads by package ({{ $snapshot['range'] }})</h2>
        <ul class="mt-4 space-y-2">
            @forelse ($snapshot['downloads']['top'] as $row)
                <li class="ref-panel flex flex-wrap items-center justify-between gap-2 px-4 py-3 text-sm">
                    @if ($row['url'])
                        <a href="{{ $row['url'] }}?range={{ $snapshot['range'] }}" class="font-medium text-signal-soft underline decoration-signal/30">{{ $row['name'] }}</a>
                    @else
                        <span class="text-grit-mist">{{ $row['name'] }}</span>
                    @endif
                    <span class="text-xs uppercase tracking-[0.12em] text-grit-mist">{{ number_format($row['downloads']) }} downloads</span>
                </li>
            @empty
                <li class="text-sm text-grit-mist">No downloads in this range.</li>
            @endforelse
        </ul>
    </div>

    <div class="mt-10">
        <h2 class="font-display text-xl font-semibold">Recent packages</h2>
        <ul class="mt-4 space-y-2">
            @forelse ($recent as $pkg)
                <li class="ref-panel flex flex-wrap items-center justify-between gap-2 px-4 py-3 text-sm">
                    <a href="{{ route('cdn.admin.packages.show', $pkg) }}" class="font-medium text-signal-soft underline decoration-signal/30">
                        {{ $pkg->displayName() }}
                    </a>
                    <span class="text-xs uppercase tracking-[0.12em] text-grit-mist">{{ $pkg->status }}</span>
                </li>
            @empty
                <li class="text-sm text-grit-mist">No packages yet.</li>
            @endforelse
        </ul>
    </div>

    <script>
        (() => {
            const metricsUrl = @json(route('cdn.admin.metrics', ['range' => $snapshot['range']]));
            const series = @json($snapshot['series']);
            const canvas = document.getElementById('cdn-chart');
            const ctx = canvas.getContext('2d');

            function draw(points) {
                const w = canvas.width = canvas.clientWidth * devicePixelRatio;
                const h = canvas.height = 192 * devicePixelRatio;
                ctx.clearRect(0, 0, w, h);
                if (!points.length) return;
                const max = Math.max(1, ...points.map(p => Math.max(p.requests, p.downloads || 0)));
                const line = (key, dash) => {
                    ctx.setLineDash(dash.map(d => d * devicePixelRatio));
                    ctx.beginPath();
                    points.forEach((p, i) => {
                        const x = (i / Math.max(1, points.length - 1)) * (w - 8) + 4;
                        const y = h - 8 - ((p[key] || 0) / max) * (h - 16);
                        i ? ctx.lineTo(x, y) : ctx.moveTo(x, y);
                    });
                    ctx.stroke();
                };
                ctx.strokeStyle = 'rgba(214, 186, 120, 0.9)';
                ctx.lineWidth = 2 * devicePixelRatio;
                line('requests', []);
                ctx.strokeStyle = 'rgba(214, 186, 120, 0.5)';
                line('downloads', [6, 4]);
                ctx.setLineDash([]);
            }
            draw(series);

            async function tick() {
                try {
                    const res = await fetch(metricsUrl, { headers: { 'Accept': 'application/json' } });
                    if (!res.ok) return;
                    const data = await res.json();
                    const live = data.live || {};
                    const totals = data.totals || {};
                    const set = (k, v) => { document.querySelector(`[data-k="${k}"]`).textContent = Number(v || 0).toLocaleString(); };
                    ['requests', 'bytes_out', 'status_2xx', 'status_4xx', 'status_5xx'].forEach(k => set(k, totals[k]));
                    set('approx_bytes_per_sec', live.approx_bytes_per_sec);
                    set('downloads_range', data.downloads?.range);
                    set('downloads_all_time', data.downloads?.all_time);
                    if (data.packages) {
                        document.querySelector('[data-k="published"]').textContent = data.packages.published;
                        document.querySelector('[data-k="draft"]').textContent = data.packages.draft;
                    }
                    document.getElementById('cdn-total-req').textContent = Number(data.totals?.requests || 0).toLocaleString();
                    document.getElementById('cdn-total-bytes').textContent = Number(data.totals?.bytes_out || 0).toLocaleString();
                    const pts = data.series || [];
                    if (pts.length) {
                        document.getElementById('cdn-series-start').textContent = pts[0].t;
                        document.getElementById('cdn-series-end').textContent = pts[pts.length - 1].t;
                    }
                    draw(pts);
                } catch (e) {}
            }
            setInterval(tick, 5000);
        })();
    </script>
@endsection
