<?php

namespace App\Support;

use App\Models\ReportIssue;
use Illuminate\Support\Facades\Storage;

class SampleReport
{
    public const FINGERPRINT = 'local-sample';

    public static function ensure(): void
    {
        if (! self::enabled()) {
            return;
        }

        $issue = ReportIssue::query()->firstOrCreate(
            ['fingerprint' => self::FINGERPRINT],
            [
                'type' => 'crash',
                'game' => 'cnc',
                'category_id' => 'foo.cpp:12',
                'title' => 'Sample crash foo.cpp:12',
                'culprit' => 'foo.cpp:12',
                'status' => 'unresolved',
            ],
        );

        if ($issue->game === null) {
            $issue->forceFill(['game' => 'cnc'])->save();
        }

        if ($issue->events()->exists()) {
            return;
        }

        $folder = 'reports/sample';
        $xml = <<<'XML'
<?xml version="1.0" encoding="UTF-8"?>
<report>
  <type>crash</type>
  <categoryid>foo.cpp:12</categoryid>
  <contextdata>Sample assert for the local dashboard.</contextdata>
</report>
XML;

        $disk = Storage::disk(config('reports.disk'));
        $disk->put($folder.'/report.xml', $xml);
        $disk->put($folder.'/screenshot.jpg', self::jpeg());

        $receivedAt = now();

        $issue->events()->create([
            'type' => 'crash',
            'category' => 'crash',
            'category_id' => 'foo.cpp:12',
            'session_id' => 'sample-session',
            'sku' => 'CNC',
            'build_signature' => 'local',
            'stack' => '0x00401234 0x00405678',
            'context_data' => 'Sample assert for the local dashboard.',
            'screenshot_path' => $folder.'/screenshot.jpg',
            'raw_path' => $folder.'/report.xml',
            'payload_bytes' => strlen($xml),
            'remote_ip' => '127.0.0.1',
            'client_created_at' => $receivedAt,
            'received_at' => $receivedAt,
        ]);

        $issue->forceFill([
            'events_count' => 1,
            'first_seen_at' => $receivedAt,
            'last_seen_at' => $receivedAt,
        ])->save();
    }

    private static function enabled(): bool
    {
        if (app()->runningUnitTests()) {
            return app()->environment('local');
        }

        return app()->environment('local') || (bool) config('app.debug');
    }

    private static function jpeg(): string
    {
        return base64_decode(
            '/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAP//////////////////////////////////////////////////////////////////////////////////////wgALCAABAAEBAREA/8QAFBABAAAAAAAAAAAAAAAAAAAAAP/aAAgBAQABPxA='
        );
    }
}
