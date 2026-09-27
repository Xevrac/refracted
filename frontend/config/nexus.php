<?php

return [

    'domain' => env('NEXUS_DOMAIN', 'nexus.refracted.au'),

    'connection' => env('NEXUS_DB_CONNECTION', 'nexus'),

    // Shared with rfrcli when set. Empty = plain sha256 (legacy).
    'token_pepper' => env('NEXUS_TOKEN_PEPPER', ''),

    // HS256 key for issued JWTs. Empty → config('app.key') at issue time.
    'jwt_key' => env('NEXUS_JWT_KEY', ''),

    // Days between persona display-name changes.
    'display_name_cooldown_days' => (int) env('NEXUS_DISPLAY_NAME_COOLDOWN_DAYS', 30),

    // Known game ids shown in admin lookup / session telemetry.
    'games' => [
        'cnc' => 'Command & Conquer',
        'bf3' => 'Battlefield 3',
    ],

];
