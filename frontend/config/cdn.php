<?php

return [

    'domain' => env('CDN_DOMAIN', 'cdn.refracted.au'),

    /*
    | When true, public Prism GETs require launcher X-Refracted-* headers.
    | Local/dev can set false to curl without headers.
    */
    'require_launcher_headers' => filter_var(
        env('CDN_REQUIRE_LAUNCHER_HEADERS', true),
        FILTER_VALIDATE_BOOLEAN
    ),

    /** Allowed X-Refracted-Client values */
    'allowed_clients' => array_values(array_filter(array_map(
        'trim',
        explode(',', (string) env('CDN_ALLOWED_CLIENTS', 'launcher'))
    ))),

    /**
     * Optional shared secret. When non-empty, launcher must send
     * X-Refracted-Client-Key matching one of these (comma-separated).
     */
    'client_keys' => array_values(array_filter(array_map(
        'trim',
        explode(',', (string) env('CDN_CLIENT_KEYS', ''))
    ))),

    'channels' => ['release', 'debug'],

    'default_channel' => env('CDN_DEFAULT_CHANNEL', 'release'),

    'disk' => env('CDN_DISK', 'cdn'),

    'titles' => [
        'cnc' => 'Command & Conquer',
        'bf3' => 'Battlefield 3',
    ],

    'rate_limit_per_minute' => (int) env('CDN_RATE_LIMIT_PER_MINUTE', 120),

];
