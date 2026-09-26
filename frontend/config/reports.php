<?php

return [

    'domain' => env('REPORTS_INGEST_DOMAIN', 'sentry.refracted.au'),

    'key' => env('REPORTS_INGEST_KEY'),

    'max_bytes' => 4 * 1024 * 1024,

    'rate_limit_per_minute' => (int) env('REPORTS_INGEST_RATE_LIMIT', 30),

    'disk' => env('REPORTS_INGEST_DISK', 'local'),

    'retain_event_days' => env('REPORTS_RETAIN_EVENT_DAYS', 90),

];
