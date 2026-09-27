<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

/** Nexus secrets and sessions. */
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

        if ($schema->hasTable('users') && ! $schema->hasColumn('users', 'secret_hash')) {
            $schema->table('users', function (Blueprint $table) {
                $table->char('secret_hash', 64)->default('');
                $table->char('secret_salt', 32)->default('');
            });
        }

        if (! $schema->hasTable('auth_sessions')) {
            $schema->create('auth_sessions', function (Blueprint $table) {
                $table->bigIncrements('id');
                $table->bigInteger('user_id');
                $table->bigInteger('persona_id');
                $table->char('token_hash', 64);
                $table->string('jwt_id', 64);
                $table->dateTime('expires_at');
                $table->dateTime('revoked_at')->nullable();
                $table->dateTime('created_at');
                $table->dateTime('last_seen_at');
                $table->unique('token_hash', 'uq_auth_sessions_token_hash');
                $table->unique('jwt_id', 'uq_auth_sessions_jwt_id');
                $table->index('user_id', 'idx_auth_sessions_user');
                $table->foreign('user_id', 'fk_auth_sessions_user')
                    ->references('id')
                    ->on('users')
                    ->cascadeOnDelete();
                $table->foreign('persona_id', 'fk_auth_sessions_persona')
                    ->references('id')
                    ->on('personas')
                    ->cascadeOnDelete();
            });
        }
    }

    public function down(): void
    {
        $schema = Schema::connection($this->connection);
        $schema->dropIfExists('auth_sessions');

        if ($schema->hasTable('users') && $schema->hasColumn('users', 'secret_hash')) {
            $schema->table('users', function (Blueprint $table) {
                $table->dropColumn(['secret_hash', 'secret_salt']);
            });
        }
    }
};
