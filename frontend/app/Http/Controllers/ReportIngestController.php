<?php

namespace App\Http\Controllers;

use App\Models\ReportEvent;
use App\Models\ReportIssue;
use App\Support\GameReport;
use Illuminate\Http\Request;
use Illuminate\Http\Response;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Storage;
use Illuminate\Support\Str;

/**
 * Receives crash / assert / desync / session reports straight from game clients.
 */
class ReportIngestController extends Controller
{
    public function store(Request $request, string $category): Response
    {
        $body = $request->getContent();
        $max = (int) config('reports.max_bytes');

        if ($body === '' || strlen($body) > $max) {
            return response('', 400);
        }

        $report = GameReport::parse($body, $category);

        if (! $report) {
            return response('', 400);
        }

        try {
            $this->record($request, $report, $body);
        } catch (\Throwable $e) {
            // Never let a storage problem look like a client problem
            Log::error('Report ingest failed', [
                'category' => $category,
                'type' => $report->type(),
                'error' => $e->getMessage(),
            ]);

            return response('', 500);
        }

        return response('', 200);
    }

    protected function record(Request $request, GameReport $report, string $body): void
    {
        $receivedAt = now();
        $paths = $this->storeAttachments($report, $receivedAt, $body);

        DB::transaction(function () use ($request, $report, $receivedAt, $paths) {
            $issue = ReportIssue::query()->firstOrCreate(
                ['fingerprint' => $report->fingerprint()],
                [
                    'type' => $report->type(),
                    'game' => $report->game(),
                    'category_id' => $report->categoryId(),
                    'title' => $report->title(),
                    'culprit' => $report->culprit(),
                    'status' => 'unresolved',
                    'first_seen_at' => $receivedAt,
                ],
            );

            $issue->events()->create([
                'type' => $report->type(),
                'category' => $report->category,
                'category_id' => $report->categoryId(),
                'session_id' => $report->field('sessionid'),
                'sku' => $report->field('sku') ?: ($report->game() !== 'unknown' ? $report->game() : null),
                'build_signature' => $report->field('buildsignature'),
                'report_version' => $report->field('version'),
                'server_name' => $report->field('servername'),
                'server_type' => $report->field('servertype'),
                'server_error' => $report->field('servererror'),
                'desync_id' => $report->field('desyncid'),
                'stack' => $report->field('stack'),
                'threads' => $report->field('threads'),
                'system_config' => $report->field('systemconfig'),
                'context_data' => $report->field('contextdata'),
                'desync_data' => $report->field('desyncdata'),
                'screenshot_path' => $paths['screenshot'],
                'memdump_path' => $paths['memdump'],
                'raw_path' => $paths['raw'],
                'payload_bytes' => $report->byteLength,
                'remote_ip' => $request->ip(),
                'client_created_at' => $report->createdAt(),
                'received_at' => $receivedAt,
            ]);

            // A resolved issue that fires again is a regression, not history.
            $reopened = $issue->status === 'resolved'
                ? ['status' => 'unresolved', 'resolved_at' => null, 'resolved_by' => null]
                : [];

            $game = $issue->game;
            if (($game === null || $game === 'unknown') && $report->game() !== 'unknown') {
                $game = $report->game();
            }

            $issue->forceFill($reopened + [
                'game' => $game,
                'last_seen_at' => $receivedAt,
                'first_seen_at' => $issue->first_seen_at ?? $receivedAt,
            ])->save();

            $issue->increment('events_count');
        });
    }

    /**
     * Screenshots, memory dumps and the original document
     *
     * @return array{screenshot: ?string, memdump: ?string, raw: ?string}
     */
    protected function storeAttachments(GameReport $report, \DateTimeInterface $receivedAt, string $body): array
    {
        $disk = Storage::disk(config('reports.disk'));
        $folder = 'reports/'.$receivedAt->format('Y/m/d').'/'.Str::uuid();

        $paths = ['screenshot' => null, 'memdump' => null, 'raw' => null];

        if ($screenshot = $report->screenshotBytes()) {
            $paths['screenshot'] = $folder.'/screenshot.'.(str_starts_with($screenshot, "\x89PNG") ? 'png' : 'jpg');
            $disk->put($paths['screenshot'], $screenshot);
        }

        if ($memdump = $report->memDumpBytes()) {
            $paths['memdump'] = $folder.'/memdump.bin';
            $disk->put($paths['memdump'], $memdump);
        }

        $paths['raw'] = $folder.'/report.xml';
        $disk->put($paths['raw'], $body);

        return $paths;
    }
}
