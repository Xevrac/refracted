<?php

namespace App\Support;

use Carbon\CarbonInterface;
use Illuminate\Support\Carbon;

/** Short relative timestamps for Nexus admin lists. */
class RelativeTime
{
    public static function ago(?CarbonInterface $at, ?CarbonInterface $now = null): string
    {
        if ($at === null) {
            return '—';
        }

        $now = Carbon::parse($now ?? now()->utc());
        $at = Carbon::parse($at)->utc();

        if ($at->greaterThan($now)) {
            return 'just now';
        }

        $seconds = (int) $at->diffInSeconds($now);

        if ($seconds < 60) {
            return 'just now';
        }

        $mins = (int) floor($seconds / 60);
        if ($mins < 60) {
            return $mins === 1 ? '1 min ago' : "{$mins} mins ago";
        }

        $hrs = (int) floor($mins / 60);
        if ($hrs < 24) {
            return $hrs === 1 ? '1 hr ago' : "{$hrs} hrs ago";
        }

        $days = (int) floor($hrs / 24);
        if ($days < 30) {
            return $days === 1 ? '1 day ago' : "{$days} days ago";
        }

        $months = (int) floor($days / 30);
        if ($months < 12) {
            return $months === 1 ? '1 month ago' : "{$months} months ago";
        }

        $years = (int) floor($days / 365);

        return $years <= 1 ? '1 year ago' : "{$years} years ago";
    }
}
