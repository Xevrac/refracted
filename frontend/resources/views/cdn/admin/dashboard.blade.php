@extends('cdn.admin.layout')

@section('title', 'Dashboard')

@section('content')
    <div class="flex flex-wrap items-end justify-between gap-4">
        <div>
            <h1 class="font-display text-3xl font-semibold tracking-[-0.02em]">Throughput</h1>
            <p class="mt-2 text-sm text-grit-mist">Live CDN requests from launchers · auto-refresh 5s</p>
        </div>
        <a href="{{ route('cdn.admin.packages.create') }}" class="bg-signal px-4 py-2.5 font-display text-xs font-semibold uppercase tracking-[0.14em] text-white hover:bg-signal-soft">
            New package
        </a>
    </div>

    <div class="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-4" id="cdn-live-cards">
        <div class="ref-panel p-4">
            <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">Requests (5m)</p>
            <p class="mt-2 font-display text-3xl font-semibold" data-k="requests_last_5m">{{ $snapshot['live']['requests_last_5m'] }}</p>
        </div>
        <div class="ref-panel p-4">
            <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">Bytes (5m)</p>
            <p class="mt-2 font-display text-3xl font-semibold" data-k="bytes_last_5m">{{ number_format($snapshot['live']['bytes_last_5m']) }}</p>
        </div>
        <div class="ref-panel p-4">
            <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">≈ B/s (5m)</p>
            <p class="mt-2 font-display text-3xl font-semibold" data-k="approx_bytes_per_sec">{{ number_format($snapshot['live']['approx_bytes_per_sec']) }}</p>
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
        <p class="text-xs uppercase tracking-[0.14em] text-grit-mist">24h requests</p>
        <canvas id="cdn-chart" class="mt-4 h-48 w-full" height="192"></canvas>
        <p class="mt-2 text-xs text-grit-mist">Total 24h: <span id="cdn-total-req">{{ number_format($snapshot['totals']['requests']) }}</span> requests · <span id="cdn-total-bytes">{{ number_format($snapshot['totals']['bytes_out']) }}</span> bytes</p>
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
            const metricsUrl = @json(route('cdn.admin.metrics'));
            const series = @json($snapshot['series']);
            const canvas = document.getElementById('cdn-chart');
            const ctx = canvas.getContext('2d');

            function draw(points) {
                const w = canvas.width = canvas.clientWidth * devicePixelRatio;
                const h = canvas.height = 192 * devicePixelRatio;
                ctx.clearRect(0, 0, w, h);
                if (!points.length) return;
                const max = Math.max(1, ...points.map(p => p.requests));
                ctx.strokeStyle = 'rgba(214, 186, 120, 0.9)';
                ctx.lineWidth = 2 * devicePixelRatio;
                ctx.beginPath();
                points.forEach((p, i) => {
                    const x = (i / Math.max(1, points.length - 1)) * (w - 8) + 4;
                    const y = h - 8 - (p.requests / max) * (h - 16);
                    i ? ctx.lineTo(x, y) : ctx.moveTo(x, y);
                });
                ctx.stroke();
            }
            draw(series);

            async function tick() {
                try {
                    const res = await fetch(metricsUrl, { headers: { 'Accept': 'application/json' } });
                    if (!res.ok) return;
                    const data = await res.json();
                    const live = data.live || {};
                    document.querySelector('[data-k="requests_last_5m"]').textContent = live.requests_last_5m ?? 0;
                    document.querySelector('[data-k="bytes_last_5m"]').textContent = Number(live.bytes_last_5m || 0).toLocaleString();
                    document.querySelector('[data-k="approx_bytes_per_sec"]').textContent = Number(live.approx_bytes_per_sec || 0).toLocaleString();
                    if (data.packages) {
                        document.querySelector('[data-k="published"]').textContent = data.packages.published;
                        document.querySelector('[data-k="draft"]').textContent = data.packages.draft;
                    }
                    document.getElementById('cdn-total-req').textContent = Number(data.totals?.requests || 0).toLocaleString();
                    document.getElementById('cdn-total-bytes').textContent = Number(data.totals?.bytes_out || 0).toLocaleString();
                    draw(data.series || []);
                } catch (e) {}
            }
            setInterval(tick, 5000);
        })();
    </script>
@endsection
