<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('users', function (Blueprint $table) {
            $table->string('discord_id')->nullable()->unique()->after('email');
            $table->string('discord_username')->nullable()->after('discord_id');
            $table->string('discord_avatar')->nullable()->after('discord_username');
            $table->timestamp('last_login_at')->nullable()->after('remember_token');
        });
    }

    public function down(): void
    {
        Schema::table('users', function (Blueprint $table) {
            $table->dropColumn([
                'discord_id',
                'discord_username',
                'discord_avatar',
                'last_login_at',
            ]);
        });
    }
};
