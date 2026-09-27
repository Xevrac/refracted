<?php

use Illuminate\Foundation\Inspiring;
use Illuminate\Support\Facades\Artisan;

Artisan::command('inspire', function () {
    $this->comment(Inspiring::quote());
})->purpose('Display an inspiring quote');

Artisan::command('nexus:migrate {--force : Force the operation to run in production}', function () {
    $exit = $this->call('migrate', [
        '--database' => config('nexus.connection', 'nexus'),
        '--path' => 'database/migrations/nexus',
        '--force' => $this->option('force'),
    ]);

    return $exit;
})->purpose('Run Nexus migrations');
