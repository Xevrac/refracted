<?php

namespace Tests;

use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Artisan;

trait RefreshNexusDatabase
{
    use RefreshDatabase {
        RefreshDatabase::refreshTestDatabase as refreshDefaultDatabase;
    }

    protected function refreshTestDatabase(): void
    {
        $this->refreshDefaultDatabase();

        Artisan::call('migrate', [
            '--database' => config('nexus.connection', 'nexus'),
            '--path' => 'database/migrations/nexus',
            '--force' => true,
        ]);
    }
}
