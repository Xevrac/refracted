<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('cdn_packages', function (Blueprint $table) {
            $table->string('kind', 16)->default('prism')->after('id')->index(); // prism | refracted
        });
    }

    public function down(): void
    {
        Schema::table('cdn_packages', function (Blueprint $table) {
            $table->dropIndex(['kind']);
            $table->dropColumn('kind');
        });
    }
};
