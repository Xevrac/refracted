<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

/** Nexus access control: settings, Discord whitelist, bans. */
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

        if (! $schema->hasTable('settings')) {
            $schema->create('settings', function (Blueprint $table) {
                $table->string('key', 64)->primary();
                $table->text('value');
                $table->dateTime('updated_at');
            });
        }

        $now = now()->utc()->format('Y-m-d H:i:s');
        foreach ([
            'registrations_open' => '1',
            'whitelist_enabled' => '0',
        ] as $key => $value) {
            DB::connection($this->connection)->table('settings')->updateOrInsert(
                ['key' => $key],
                ['value' => $value, 'updated_at' => $now]
            );
        }

        if (! $schema->hasTable('signup_whitelist')) {
            $schema->create('signup_whitelist', function (Blueprint $table) {
                $table->id();
                $table->string('discord_id', 32);
                $table->string('note', 255)->default('');
                $table->unsignedBigInteger('created_by_web_user_id')->nullable();
                $table->dateTime('created_at');
                $table->unique('discord_id', 'uq_signup_whitelist_discord');
            });
        }

        if (! $schema->hasTable('bans')) {
            $schema->create('bans', function (Blueprint $table) {
                $table->id();
                $table->bigInteger('user_id')->nullable()->index('idx_bans_user_id');
                $table->string('discord_id', 32)->nullable()->index('idx_bans_discord_id');
                $table->string('reason', 512)->default('');
                $table->dateTime('banned_until')->nullable(); // null = permanent
                $table->unsignedBigInteger('created_by_web_user_id')->nullable();
                $table->dateTime('created_at');
                $table->unsignedBigInteger('lifted_by_web_user_id')->nullable();
                $table->dateTime('lifted_at')->nullable();
            });
        }
    }

    public function down(): void
    {
        Schema::connection($this->connection)->dropIfExists('bans');
        Schema::connection($this->connection)->dropIfExists('signup_whitelist');
        Schema::connection($this->connection)->dropIfExists('settings');
    }
};
