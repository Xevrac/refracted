<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * A distinct fault, grouped across every client that hit it.
 */
class ReportIssue extends Model
{
    public const STATUSES = ['unresolved', 'resolved', 'ignored'];

    protected $fillable = [
        'fingerprint',
        'type',
        'category_id',
        'title',
        'culprit',
        'status',
        'notes',
    ];

    protected function casts(): array
    {
        return [
            'events_count' => 'integer',
            'sessions_count' => 'integer',
            'first_seen_at' => 'datetime',
            'last_seen_at' => 'datetime',
            'resolved_at' => 'datetime',
        ];
    }

    public function events(): HasMany
    {
        return $this->hasMany(ReportEvent::class);
    }

    public function resolver(): BelongsTo
    {
        return $this->belongsTo(User::class, 'resolved_by');
    }

    public function scopeStatus(Builder $query, ?string $status): Builder
    {
        return filled($status) ? $query->where('status', $status) : $query;
    }

    public function scopeType(Builder $query, ?string $type): Builder
    {
        return filled($type) ? $query->where('type', $type) : $query;
    }

    public function scopeSearch(Builder $query, ?string $term): Builder
    {
        if (! filled($term)) {
            return $query;
        }

        $like = '%'.$term.'%';

        return $query->where(function (Builder $q) use ($like) {
            $q->where('title', 'like', $like)
                ->orWhere('culprit', 'like', $like)
                ->orWhere('category_id', 'like', $like);
        });
    }

    public function scopeRecent(Builder $query): Builder
    {
        return $query->orderByDesc('last_seen_at')->orderByDesc('id');
    }

    /** Recount from events so the list stays honest after a prune or delete. */
    public function refreshCounts(): void
    {
        $this->forceFill([
            'events_count' => $this->events()->count(),
            'sessions_count' => $this->events()->distinct()->count('session_id'),
            'first_seen_at' => $this->events()->min('received_at'),
            'last_seen_at' => $this->events()->max('received_at'),
        ])->save();
    }
}
