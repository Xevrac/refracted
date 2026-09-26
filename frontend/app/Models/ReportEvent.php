<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Support\Facades\Storage;

/**
 * One report as uploaded by one client.
 */
class ReportEvent extends Model
{
    protected $fillable = [
        'type',
        'category',
        'category_id',
        'session_id',
        'sku',
        'build_signature',
        'report_version',
        'server_name',
        'server_type',
        'server_error',
        'desync_id',
        'stack',
        'threads',
        'system_config',
        'context_data',
        'desync_data',
        'screenshot_path',
        'memdump_path',
        'raw_path',
        'payload_bytes',
        'remote_ip',
        'client_created_at',
        'received_at',
    ];

    protected function casts(): array
    {
        return [
            'payload_bytes' => 'integer',
            'client_created_at' => 'datetime',
            'received_at' => 'datetime',
        ];
    }

    public function issue(): BelongsTo
    {
        return $this->belongsTo(ReportIssue::class, 'report_issue_id');
    }

    public function hasScreenshot(): bool
    {
        return filled($this->screenshot_path);
    }

    /**
     * Stack as individual frames. The client writes one hex address per frame,
     * space separated, with no symbolisation.
     */
    public function stackFrames(): array
    {
        if (! filled($this->stack)) {
            return [];
        }

        return preg_split('/\s+/', trim($this->stack)) ?: [];
    }

    /** Remove the on-disk attachments this event owns. */
    public function deleteAttachments(): void
    {
        $disk = Storage::disk(config('reports.disk'));

        foreach ([$this->screenshot_path, $this->memdump_path, $this->raw_path] as $path) {
            if (filled($path) && $disk->exists($path)) {
                $disk->delete($path);
            }
        }
    }

    protected static function booted(): void
    {
        static::deleted(function (self $event) {
            $event->deleteAttachments();
        });
    }
}
