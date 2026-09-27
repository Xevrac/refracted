<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

/** Why an auth session was revoked (signout, ban, admin), for the admin Sessions view. */
return new class extends Migration
{
    public function up(): void
    {
        $schema = Schema::connection(config('nexus.connection', 'nexus'));
        if (! $schema->hasTable('auth_sessions') || $schema->hasColumn('auth_sessions', 'revoked_reason')) {
            return;
        }

        $schema->table('auth_sessions', function (Blueprint $table) {
            $table->string('revoked_reason', 32)->nullable()->after('revoked_at');
        });
    }

    public function down(): void
    {
        $schema = Schema::connection(config('nexus.connection', 'nexus'));
        if ($schema->hasTable('auth_sessions') && $schema->hasColumn('auth_sessions', 'revoked_reason')) {
            $schema->table('auth_sessions', function (Blueprint $table) {
                $table->dropColumn('revoked_reason');
            });
        }
    }
};
