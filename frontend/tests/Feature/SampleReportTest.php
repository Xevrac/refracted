<?php

namespace Tests\Feature;

use App\Models\ReportIssue;
use App\Models\User;
use App\Support\SampleReport;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class SampleReportTest extends TestCase
{
    use RefreshDatabase;

    public function test_sample_issue_is_absent_while_testing(): void
    {
        $staff = User::factory()->staff()->create();

        $this->actingAs($staff)
            ->get('http://sentry.refracted.au/admin/reports')
            ->assertOk()
            ->assertDontSee('Sample crash');

        $this->assertDatabaseMissing('report_issues', [
            'fingerprint' => SampleReport::FINGERPRINT,
        ]);
    }

    public function test_local_mode_creates_a_sample_issue_that_opens(): void
    {
        $this->app->detectEnvironment(fn () => 'local');

        $staff = User::factory()->staff()->create();

        $this->actingAs($staff)
            ->get('http://sentry.refracted.au/admin/reports')
            ->assertOk()
            ->assertSee('Sample crash foo.cpp:12');

        $issue = ReportIssue::query()->where('fingerprint', SampleReport::FINGERPRINT)->first();
        $this->assertNotNull($issue);
        $this->assertSame(1, $issue->events_count);

        $this->actingAs($staff)
            ->get('http://sentry.refracted.au/admin/reports/'.$issue->id)
            ->assertOk()
            ->assertSee('Sample assert for the local dashboard.');
    }
}
