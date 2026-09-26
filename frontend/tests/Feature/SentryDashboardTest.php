<?php

namespace Tests\Feature;

use App\Models\ReportIssue;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class SentryDashboardTest extends TestCase
{
    use RefreshDatabase;

    public function test_reporting_summarises_games_and_issues_filter_by_game(): void
    {
        $staff = User::factory()->staff()->create();

        ReportIssue::query()->create([
            'fingerprint' => hash('sha256', 'cnc'),
            'type' => 'crash',
            'game' => 'cnc',
            'title' => 'Shadow split assert',
            'status' => 'unresolved',
        ])->forceFill(['events_count' => 4, 'last_seen_at' => now()])->save();

        ReportIssue::query()->create([
            'fingerprint' => hash('sha256', 'labs'),
            'type' => 'assert',
            'game' => 'labs',
            'title' => 'Labs material missing',
            'status' => 'unresolved',
        ])->forceFill(['events_count' => 1, 'last_seen_at' => now()])->save();

        $this->actingAs($staff)
            ->get('http://sentry.refracted.au/admin/reports?tab=reporting')
            ->assertOk()
            ->assertSee('Reporting')
            ->assertSee('Command & Conquer')
            ->assertSee('Battlefield Labs')
            ->assertSee('By game')
            ->assertSee('By type');

        $this->actingAs($staff)
            ->get('http://sentry.refracted.au/admin/reports?tab=issues&game=cnc')
            ->assertOk()
            ->assertSee('Shadow split assert')
            ->assertDontSee('Labs material missing');
    }
}
