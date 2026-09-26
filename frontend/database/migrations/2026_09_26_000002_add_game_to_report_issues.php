<?php

use App\Support\GameReport;
use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('report_issues', function (Blueprint $table) {
            $table->string('game', 64)->nullable()->index()->after('type');
        });

        $seen = [];

        foreach (DB::table('report_events')->orderBy('id')->get(['report_issue_id', 'sku']) as $event) {
            if (isset($seen[$event->report_issue_id]) || ! filled($event->sku)) {
                continue;
            }

            $seen[$event->report_issue_id] = true;

            DB::table('report_issues')
                ->where('id', $event->report_issue_id)
                ->whereNull('game')
                ->update(['game' => GameReport::gameSlug($event->sku)]);
        }
    }

    public function down(): void
    {
        Schema::table('report_issues', function (Blueprint $table) {
            $table->dropColumn('game');
        });
    }
};
