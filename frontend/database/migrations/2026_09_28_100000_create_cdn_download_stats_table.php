<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        // Per-minute successful file serves by package. No FK so history survives package deletion.
        Schema::create('cdn_download_stats', function (Blueprint $table) {
            $table->id();
            $table->dateTime('bucket_at')->index();
            $table->unsignedBigInteger('package_id')->index();
            $table->string('path', 512);
            $table->unsignedInteger('downloads')->default(0);
            $table->unsignedBigInteger('bytes_out')->default(0);

            $table->unique(['bucket_at', 'package_id', 'path'], 'cdn_download_stats_bucket');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('cdn_download_stats');
    }
};
