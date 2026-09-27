<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

/** Nexus users and personas. */
return new class extends Migration
{
    protected $connection;

    public function __construct()
    {
        $this->connection = config('nexus.connection', 'nexus');
    }

    public function up(): void
    {
        if (! Schema::connection($this->connection)->hasTable('users')) {
            Schema::connection($this->connection)->create('users', function (Blueprint $table) {
                $table->bigInteger('id')->primary();
                $table->string('username', 64);
                $table->string('email', 255)->default('');
                $table->dateTime('created_at');
                $table->unique('username', 'uq_users_username');
            });
        }

        if (! Schema::connection($this->connection)->hasTable('personas')) {
            Schema::connection($this->connection)->create('personas', function (Blueprint $table) {
                $table->bigInteger('id')->primary();
                $table->bigInteger('user_id');
                $table->string('display_name', 64);
                $table->dateTime('created_at');
                $table->index('user_id', 'idx_personas_user_id');
                $table->foreign('user_id', 'fk_personas_user')
                    ->references('id')
                    ->on('users')
                    ->cascadeOnDelete();
            });
        }
    }

    public function down(): void
    {
        Schema::connection($this->connection)->dropIfExists('personas');
        Schema::connection($this->connection)->dropIfExists('users');
    }
};
