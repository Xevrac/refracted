<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('report_events', function (Blueprint $table) {
            $table->longText('prism_log')->nullable()->after('stack');
        });
    }

    public function down(): void
    {
        Schema::table('report_events', function (Blueprint $table) {
            $table->dropColumn('prism_log');
        });
    }
};
