<?php

namespace App\Services\Cdn;

use App\Models\Cdn\CdnPackage;
use Illuminate\Support\Carbon;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\DB;

class CdnMetrics
{
    public function record(
        string $kind,
        int $status,
        int $bytesOut = 0,
        ?string $titleId = null,
        ?string $channel = null,
    ): void {
        $kind = in_array($kind, ['manifest', 'file', 'other'], true) ? $kind : 'other';
        $bytesOut = max(0, $bytesOut);
        $bucket = Carbon::now()->startOfMinute();

        $class = $status >= 500 ? '5xx' : ($status >= 400 ? '4xx' : '2xx');

        Cache::increment('cdn:live:requests', 1);
        Cache::increment('cdn:live:bytes', $bytesOut);
        Cache::increment("cdn:live:status:{$class}", 1);

        $titleId = $titleId !== null && $titleId !== '' ? $titleId : '';
        $channel = $channel !== null && $channel !== '' ? $channel : '';

        DB::table('cdn_request_stats')->upsert(
            [[
                'bucket_at' => $bucket,
                'title_id' => $titleId,
                'channel' => $channel,
                'kind' => $kind,
                'requests' => 1,
                'bytes_out' => $bytesOut,
                'status_2xx' => $class === '2xx' ? 1 : 0,
                'status_4xx' => $class === '4xx' ? 1 : 0,
                'status_5xx' => $class === '5xx' ? 1 : 0,
            ]],
            ['bucket_at', 'title_id', 'channel', 'kind'],
            [
                'requests' => DB::raw('requests + 1'),
                'bytes_out' => DB::raw("bytes_out + {$bytesOut}"),
                'status_2xx' => DB::raw('status_2xx + '.($class === '2xx' ? 1 : 0)),
                'status_4xx' => DB::raw('status_4xx + '.($class === '4xx' ? 1 : 0)),
                'status_5xx' => DB::raw('status_5xx + '.($class === '5xx' ? 1 : 0)),
            ],
        );
    }

    /** Selectable report windows: key => [window seconds, bucket seconds, label]. */
    public const RANGES = [
        '5m' => [300, 60, '5 min'],
        '30m' => [1800, 60, '30 min'],
        '1h' => [3600, 60, '1 hr'],
        '24h' => [86400, 900, '24 hrs'],
        '30d' => [2592000, 21600, '30 days'],
        '1y' => [31536000, 86400, '1 yr'],
    ];

    public const DEFAULT_RANGE = '24h';

    public static function normalizeRange(?string $range): string
    {
        return isset(self::RANGES[$range]) ? $range : self::DEFAULT_RANGE;
    }

    /** First bucket start of the window; the window ends with the bucket containing now. */
    public static function rangeStart(string $range): Carbon
    {
        [$window, $bucket] = self::RANGES[self::normalizeRange($range)];
        $now = Carbon::now()->getTimestamp();

        return Carbon::createFromTimestamp(intdiv($now, $bucket) * $bucket - $window + $bucket);
    }

    public function recordDownload(CdnPackage $package, string $path, int $bytesOut = 0): void
    {
        $bytesOut = max(0, $bytesOut);

        DB::table('cdn_download_stats')->upsert(
            [[
                'bucket_at' => Carbon::now()->startOfMinute(),
                'package_id' => $package->id,
                'path' => $path,
                'downloads' => 1,
                'bytes_out' => $bytesOut,
            ]],
            ['bucket_at', 'package_id', 'path'],
            [
                'downloads' => DB::raw('downloads + 1'),
                'bytes_out' => DB::raw("bytes_out + {$bytesOut}"),
            ],
        );
    }

    /**
     * Downloads per package, optionally limited to a window start.
     *
     * @param  array<int>|null  $packageIds
     * @return array<int, int>
     */
    public function downloadsByPackage(?array $packageIds = null, ?Carbon $since = null): array
    {
        return DB::table('cdn_download_stats')
            ->when($packageIds !== null, fn ($q) => $q->whereIn('package_id', $packageIds))
            ->when($since !== null, fn ($q) => $q->where('bucket_at', '>=', $since))
            ->groupBy('package_id')
            ->selectRaw('package_id, SUM(downloads) as n')
            ->pluck('n', 'package_id')
            ->map(fn ($n) => (int) $n)
            ->all();
    }

    /**
     * Downloads per artifact path of one package.
     *
     * @return array<string, int>
     */
    public function downloadsByPath(int $packageId, ?Carbon $since = null): array
    {
        return DB::table('cdn_download_stats')
            ->where('package_id', $packageId)
            ->when($since !== null, fn ($q) => $q->where('bucket_at', '>=', $since))
            ->groupBy('path')
            ->selectRaw('path, SUM(downloads) as n')
            ->pluck('n', 'path')
            ->map(fn ($n) => (int) $n)
            ->all();
    }

    /** @return array<string, mixed> */
    public function snapshot(string $range = self::DEFAULT_RANGE): array
    {
        $range = self::normalizeRange($range);
        [$window, $bucket, $label] = self::RANGES[$range];
        $since = self::rangeStart($range);

        // Pre-aggregate in SQL by minute/hour/day text prefix, then fold into $bucket in PHP.
        $prefix = $bucket < 3600 ? 16 : ($bucket < 86400 ? 13 : 10);
        $key = "substr(CAST(bucket_at AS CHAR(19)), 1, {$prefix})";
        $pad = [16 => ':00', 13 => ':00:00', 10 => ' 00:00:00'][$prefix];

        $rows = DB::table('cdn_request_stats')
            ->where('bucket_at', '>=', $since)
            ->groupByRaw($key)
            ->selectRaw("{$key} as k, SUM(requests) as requests, SUM(bytes_out) as bytes_out, "
                .'SUM(status_2xx) as status_2xx, SUM(status_4xx) as status_4xx, SUM(status_5xx) as status_5xx')
            ->get();

        $downloadRows = DB::table('cdn_download_stats')
            ->where('bucket_at', '>=', $since)
            ->groupByRaw($key)
            ->selectRaw("{$key} as k, SUM(downloads) as downloads")
            ->pluck('downloads', 'k');

        $series = [];
        for ($t = $since->getTimestamp(), $end = Carbon::now()->getTimestamp(); $t <= $end; $t += $bucket) {
            $series[$t] = [
                't' => Carbon::createFromTimestamp($t)->format('Y-m-d H:i'),
                'requests' => 0,
                'bytes_out' => 0,
                'downloads' => 0,
            ];
        }

        $slot = function (string $k) use ($pad, $bucket, $series): ?int {
            $ts = Carbon::parse($k.$pad)->getTimestamp();
            $ts = intdiv($ts, $bucket) * $bucket;

            return isset($series[$ts]) ? $ts : null;
        };

        $totals = [
            'requests' => 0,
            'bytes_out' => 0,
            'status_2xx' => 0,
            'status_4xx' => 0,
            'status_5xx' => 0,
            'downloads' => 0,
        ];

        foreach ($rows as $row) {
            foreach (['requests', 'bytes_out', 'status_2xx', 'status_4xx', 'status_5xx'] as $f) {
                $totals[$f] += (int) $row->{$f};
            }
            if (($ts = $slot($row->k)) !== null) {
                $series[$ts]['requests'] += (int) $row->requests;
                $series[$ts]['bytes_out'] += (int) $row->bytes_out;
            }
        }

        foreach ($downloadRows as $k => $n) {
            $totals['downloads'] += (int) $n;
            if (($ts = $slot($k)) !== null) {
                $series[$ts]['downloads'] += (int) $n;
            }
        }

        $elapsed = max(1, Carbon::now()->getTimestamp() - $since->getTimestamp());

        $byPackage = $this->downloadsByPackage(null, $since);
        arsort($byPackage);
        $top = array_slice($byPackage, 0, 10, true);
        $names = CdnPackage::query()->whereKey(array_keys($top))->get()->keyBy('id');
        $topDownloads = [];
        foreach ($top as $id => $n) {
            $pkg = $names->get($id);
            $topDownloads[] = [
                'package_id' => $id,
                'name' => $pkg?->displayName() ?? "deleted package #{$id}",
                'url' => $pkg ? route('cdn.admin.packages.show', $pkg) : null,
                'downloads' => $n,
            ];
        }

        return [
            'range' => $range,
            'range_label' => $label,
            'bucket_seconds' => $bucket,
            'since' => $since->toIso8601String(),
            'live' => [
                'requests_cache' => (int) Cache::get('cdn:live:requests', 0),
                'bytes_cache' => (int) Cache::get('cdn:live:bytes', 0),
                'approx_bytes_per_sec' => (int) round($totals['bytes_out'] / $elapsed),
            ],
            'totals' => $totals,
            'downloads' => [
                'range' => $totals['downloads'],
                'all_time' => (int) DB::table('cdn_download_stats')->sum('downloads'),
                'top' => $topDownloads,
            ],
            'series' => array_values($series),
            'packages' => [
                'published' => CdnPackage::query()->published()->count(),
                'draft' => CdnPackage::query()->where('status', 'draft')->count(),
            ],
            'generated_at' => now()->toIso8601String(),
        ];
    }
}
