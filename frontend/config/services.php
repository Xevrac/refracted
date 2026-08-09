<?php

return [

    /*
    |--------------------------------------------------------------------------
    | Third Party Services
    |--------------------------------------------------------------------------
    |
    | This file is for storing the credentials for third party services such
    | as Mailgun, Postmark, AWS and more. This file provides the de facto
    | location for this type of information, allowing packages to have
    | a conventional file to locate the various service credentials.
    |
    */

    'postmark' => [
        'key' => env('POSTMARK_API_KEY'),
    ],

    'resend' => [
        'key' => env('RESEND_API_KEY'),
    ],

    'ses' => [
        'key' => env('AWS_ACCESS_KEY_ID'),
        'secret' => env('AWS_SECRET_ACCESS_KEY'),
        'region' => env('AWS_DEFAULT_REGION', 'us-east-1'),
    ],

    'slack' => [
        'notifications' => [
            'bot_user_oauth_token' => env('SLACK_BOT_USER_OAUTH_TOKEN'),
            'channel' => env('SLACK_BOT_USER_DEFAULT_CHANNEL'),
        ],
    ],

    'discord' => [
        'invite' => env('DISCORD_INVITE_URL', 'https://discord.gg/'),
        'client_id' => env('DISCORD_CLIENT_ID'),
        'client_secret' => env('DISCORD_CLIENT_SECRET'),
        'redirect' => env('DISCORD_REDIRECT_URI', env('APP_URL').'/gateway/callback'),
        'admin_ids' => env('DISCORD_ADMIN_IDS', ''),
    ],

    'contact' => [
        'discord' => env('CONTACT_DISCORD', 'xevrac'),
        'telegram' => env('CONTACT_TELEGRAM', 'Xevrac'),
        'telegram_url' => env('CONTACT_TELEGRAM_URL', 'https://t.me/Xevrac'),
    ],

    'support' => [
        'url' => env('SUPPORT_URL', '#'),
    ],
];
