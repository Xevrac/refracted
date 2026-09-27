<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

/** Nexus Discord link. */
return new class extends Migration
{
    protected $connection;

    public function __construct()
    {
        $this->connection = config('nexus.connection', 'nexus');
    }

    public function up(): void
    {
        $schema = Schema::connection($this->connection);

        if (! $schema->hasTable('users')) {
            return;
        }

        if (! $schema->hasColumn('users', 'discord_id')) {
            $schema->table('users', function (Blueprint $table) {
                $table->string('discord_id', 32)->nullable();
                $table->unique('discord_id', 'uq_users_discord_id');
            });
        }

        if (! $schema->hasColumn('users', 'web_user_id')) {
            $schema->table('users', function (Blueprint $table) {
                $table->unsignedBigInteger('web_user_id')->nullable();
                $table->index('web_user_id', 'idx_users_web_user_id');
            });
        }
    }

    public function down(): void
    {
        $schema = Schema::connection($this->connection);

        if (! $schema->hasTable('users')) {
            return;
        }

        if ($schema->hasColumn('users', 'discord_id')) {
            $schema->table('users', function (Blueprint $table) {
                $table->dropUnique('uq_users_discord_id');
                $table->dropColumn('discord_id');
            });
        }

        if ($schema->hasColumn('users', 'web_user_id')) {
            $schema->table('users', function (Blueprint $table) {
                $table->dropIndex('idx_users_web_user_id');
                $table->dropColumn('web_user_id');
            });
        }
    }
};
