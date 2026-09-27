<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

/** Auth session client telemetry for admin lookup. */
return new class extends Migration
{
    public function up(): void
    {
        $connection = config('nexus.connection', 'nexus');
        $schema = Schema::connection($connection);

        if (! $schema->hasTable('auth_sessions')) {
            return;
        }

        $schema->table('auth_sessions', function (Blueprint $table) use ($schema) {
            if (! $schema->hasColumn('auth_sessions', 'client_ip')) {
                $table->string('client_ip', 45)->nullable()->after('last_seen_at');
            }
            if (! $schema->hasColumn('auth_sessions', 'country_code')) {
                $table->string('country_code', 2)->nullable()->after('client_ip');
            }
            if (! $schema->hasColumn('auth_sessions', 'game_id')) {
                $table->string('game_id', 64)->nullable()->index('idx_auth_sessions_game')->after('country_code');
            }
        });
    }

    public function down(): void
    {
        $connection = config('nexus.connection', 'nexus');
        $schema = Schema::connection($connection);

        if (! $schema->hasTable('auth_sessions')) {
            return;
        }

        $schema->table('auth_sessions', function (Blueprint $table) use ($schema) {
            if ($schema->hasColumn('auth_sessions', 'game_id')) {
                $table->dropIndex('idx_auth_sessions_game');
                $table->dropColumn('game_id');
            }
            if ($schema->hasColumn('auth_sessions', 'country_code')) {
                $table->dropColumn('country_code');
            }
            if ($schema->hasColumn('auth_sessions', 'client_ip')) {
                $table->dropColumn('client_ip');
            }
        });
    }
};
