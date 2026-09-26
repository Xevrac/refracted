<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        // One row per distinct fault, the way staff think about it.
        Schema::create('report_issues', function (Blueprint $table) {
            $table->id();
            $table->string('fingerprint', 64)->unique();
            $table->string('type', 32)->index();
            $table->string('category_id')->nullable();
            $table->string('title');
            $table->string('culprit')->nullable();
            $table->string('status', 16)->default('unresolved');
            $table->unsignedInteger('events_count')->default(0);
            $table->unsignedInteger('sessions_count')->default(0);
            $table->timestamp('first_seen_at')->nullable();
            $table->timestamp('last_seen_at')->nullable();
            $table->timestamp('resolved_at')->nullable();
            $table->foreignId('resolved_by')->nullable()->constrained('users')->nullOnDelete();
            $table->text('notes')->nullable();
            $table->timestamps();

            // Drives the default list: open issues, most recent first.
            $table->index(['status', 'last_seen_at']);
        });

        // One row per report the game actually uploaded.
        Schema::create('report_events', function (Blueprint $table) {
            $table->id();
            $table->foreignId('report_issue_id')->constrained('report_issues')->cascadeOnDelete();

            $table->string('type', 32)->index();
            $table->string('category', 32)->nullable();
            $table->string('category_id')->nullable();

            $table->string('session_id')->nullable()->index();
            $table->string('sku')->nullable();
            $table->string('build_signature')->nullable()->index();
            $table->string('report_version', 32)->nullable();

            $table->string('server_name')->nullable();
            $table->string('server_type', 64)->nullable();
            $table->text('server_error')->nullable();
            $table->string('desync_id')->nullable();

            $table->longText('stack')->nullable();
            $table->longText('threads')->nullable();
            $table->longText('system_config')->nullable();
            $table->longText('context_data')->nullable();
            $table->longText('desync_data')->nullable();

            // Binary attachments live on disk; only the path is stored here.
            $table->string('screenshot_path')->nullable();
            $table->string('memdump_path')->nullable();
            $table->string('raw_path')->nullable();

            $table->unsignedInteger('payload_bytes')->default(0);
            $table->string('remote_ip', 45)->nullable();
            $table->timestamp('client_created_at')->nullable();
            $table->timestamp('received_at')->nullable()->index();
            $table->timestamps();
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('report_events');
        Schema::dropIfExists('report_issues');
    }
};
