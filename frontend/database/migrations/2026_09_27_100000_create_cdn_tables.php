<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('cdn_packages', function (Blueprint $table) {
            $table->id();
            $table->string('title_id', 32)->index();
            $table->string('channel', 16)->index(); // release | debug
            $table->string('version', 64);
            $table->unsignedInteger('revision')->default(1);
            $table->string('label', 120)->nullable();
            $table->text('notes')->nullable();
            $table->string('status', 16)->default('draft')->index(); // draft|published|archived
            $table->boolean('is_latest')->default(false)->index();
            $table->timestamp('published_at')->nullable();
            $table->foreignId('created_by')->nullable()->constrained('users')->nullOnDelete();
            $table->timestamps();

            $table->unique(['title_id', 'channel', 'version', 'revision'], 'cdn_packages_identity');
        });

        Schema::create('cdn_artifacts', function (Blueprint $table) {
            $table->id();
            $table->foreignId('package_id')->constrained('cdn_packages')->cascadeOnDelete();
            $table->string('path', 512);
            $table->string('storage_path', 512);
            $table->string('sha256', 64);
            $table->unsignedBigInteger('size_bytes')->default(0);
            $table->string('content_type', 128)->nullable();
            $table->timestamps();

            $table->unique(['package_id', 'path']);
        });

        Schema::create('cdn_request_stats', function (Blueprint $table) {
            $table->id();
            $table->dateTime('bucket_at')->index();
            $table->string('title_id', 32)->default('')->index();
            $table->string('channel', 16)->default('')->index();
            $table->string('kind', 16)->default('other'); // manifest|file|other
            $table->unsignedInteger('requests')->default(0);
            $table->unsignedBigInteger('bytes_out')->default(0);
            $table->unsignedInteger('status_2xx')->default(0);
            $table->unsignedInteger('status_4xx')->default(0);
            $table->unsignedInteger('status_5xx')->default(0);

            $table->unique(
                ['bucket_at', 'title_id', 'channel', 'kind'],
                'cdn_request_stats_bucket'
            );
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('cdn_request_stats');
        Schema::dropIfExists('cdn_artifacts');
        Schema::dropIfExists('cdn_packages');
    }
};
