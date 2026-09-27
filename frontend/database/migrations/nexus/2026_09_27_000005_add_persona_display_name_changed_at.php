<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        $connection = config('nexus.connection', 'nexus');
        $schema = Schema::connection($connection);

        if (! $schema->hasTable('personas')) {
            return;
        }

        if (! $schema->hasColumn('personas', 'display_name_changed_at')) {
            $schema->table('personas', function (Blueprint $table) {
                $table->timestamp('display_name_changed_at')->nullable()->after('display_name');
            });
        }
    }

    public function down(): void
    {
        $connection = config('nexus.connection', 'nexus');
        $schema = Schema::connection($connection);

        if ($schema->hasTable('personas') && $schema->hasColumn('personas', 'display_name_changed_at')) {
            $schema->table('personas', function (Blueprint $table) {
                $table->dropColumn('display_name_changed_at');
            });
        }
    }
};
