<?php

namespace App\Services\Cdn;

use App\Models\Cdn\CdnPackage;
use App\Models\Cdn\CdnRequestStat;
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

    /** @return array<string, mixed> */
    public function snapshot(int $hours = 24): array
    {
        $since = Carbon::now()->subHours($hours)->startOfMinute();

        $rows = CdnRequestStat::query()
            ->where('bucket_at', '>=', $since)
            ->orderBy('bucket_at')
            ->get();

        $series = [];
        $totals = [
            'requests' => 0,
            'bytes_out' => 0,
            'status_2xx' => 0,
            'status_4xx' => 0,
            'status_5xx' => 0,
        ];

        foreach ($rows as $row) {
            $key = $row->bucket_at->format('Y-m-d H:i');
            if (! isset($series[$key])) {
                $series[$key] = [
                    't' => $key,
                    'requests' => 0,
                    'bytes_out' => 0,
                ];
            }
            $series[$key]['requests'] += $row->requests;
            $series[$key]['bytes_out'] += $row->bytes_out;
            $totals['requests'] += $row->requests;
            $totals['bytes_out'] += $row->bytes_out;
            $totals['status_2xx'] += $row->status_2xx;
            $totals['status_4xx'] += $row->status_4xx;
            $totals['status_5xx'] += $row->status_5xx;
        }

        $last5 = CdnRequestStat::query()
            ->where('bucket_at', '>=', Carbon::now()->subMinutes(5))
            ->get();

        $rpm = (int) $last5->sum('requests');
        $bps = (int) $last5->sum('bytes_out');

        return [
            'live' => [
                'requests_cache' => (int) Cache::get('cdn:live:requests', 0),
                'bytes_cache' => (int) Cache::get('cdn:live:bytes', 0),
                'requests_last_5m' => $rpm,
                'bytes_last_5m' => $bps,
                'approx_bytes_per_sec' => (int) round($bps / 300),
            ],
            'totals' => $totals,
            'series' => array_values($series),
            'packages' => [
                'published' => CdnPackage::query()->published()->count(),
                'draft' => CdnPackage::query()->where('status', 'draft')->count(),
            ],
            'generated_at' => now()->toIso8601String(),
        ];
    }
}
