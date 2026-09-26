<?php

namespace Tests\Feature;

use App\Models\ReportIssue;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class ReportIngestTest extends TestCase
{
    use RefreshDatabase;

    private function sentry(string $method, string $path, ?string $body = null)
    {
        return $this->call(
            $method,
            'http://sentry.refracted.au'.$path,
            server: [
                'HTTP_HOST' => 'sentry.refracted.au',
                'CONTENT_TYPE' => 'application/xml',
            ],
            content: $body ?? '',
        );
    }

    public function test_sentry_host_is_the_staff_dashboard(): void
    {
        $this->sentry('GET', '/')
            ->assertRedirect(route('admin.login'));

        $staff = User::factory()->staff()->create();

        $this->actingAs($staff)
            ->get('http://sentry.refracted.au/')
            ->assertRedirect(route('admin.reports.index'));
    }

    public function test_main_site_does_not_serve_the_dashboard(): void
    {
        $this->call('GET', 'http://refracted.au/admin/login', server: [
            'HTTP_HOST' => 'refracted.au',
        ])->assertNotFound();

        $this->call('GET', 'http://refracted.au/admin', server: [
            'HTTP_HOST' => 'refracted.au',
        ])->assertNotFound();

        $this->call('GET', 'http://sentry.refracted.au/admin/login', server: [
            'HTTP_HOST' => 'sentry.refracted.au',
        ])->assertOk();
    }

    public function test_main_site_does_not_accept_report_posts(): void
    {
        $xml = '<?xml version="1.0"?><report><type>crash</type><categoryid>foo.cpp:1</categoryid></report>';

        $this->call('POST', 'http://refracted.au/crash/', server: [
            'HTTP_HOST' => 'refracted.au',
            'CONTENT_TYPE' => 'application/xml',
        ], content: $xml)->assertNotFound();
    }

    public function test_bugsentry_xml_is_stored(): void
    {
        $xml = <<<'XML'
<?xml version="1.0" encoding="UTF-8"?>
<report>
  <type>crash</type>
  <categoryid>foo.cpp:12</categoryid>
  <contextdata>boom</contextdata>
</report>
XML;

        $this->sentry('POST', '/crash/', $xml)->assertNotFound();
        $this->sentry('POST', '/testkey/crash/', $xml)->assertOk();

        $this->assertDatabaseHas('report_issues', [
            'type' => 'crash',
            'category_id' => 'foo.cpp:12',
            'title' => 'foo.cpp:12',
        ]);
    }

    public function test_devtrack_bug_submit_is_stored(): void
    {
        $xml = <<<'XML'
<?xml version="1.0" encoding="ISO-8859-1"?>
<DevTrackBug>
  <Title>Units path the wrong way</Title>
  <Description>Right click north, they walk south.</Description>
  <OwnerUserName>closed-testing</OwnerUserName>
  <CustomField><Key>severity</Key><Value>High</Value></CustomField>
</DevTrackBug>
XML;

        $this->sentry('POST', '/testkey/submit/', $xml)->assertOk();

        $issue = ReportIssue::query()->first();
        $this->assertNotNull($issue);
        $this->assertSame('submit', $issue->type);
        $this->assertSame('Units path the wrong way', $issue->title);
        $this->assertSame('Right click north, they walk south.', $issue->events()->first()?->context_data);
    }

    public function test_assert_submit_keeps_the_dialog_fields(): void
    {
        $xml = <<<'XML'
<?xml version="1.0" encoding="ISO-8859-1"?>
<DevTrackBug>
  <Title>lowDistance: 1.26729, nearPlane: 2.25, farPlane: 20000.1</Title>
  <Description></Description>
  <CustomField><Key>Severity</Key><Value>Major</Value></CustomField>
  <CustomField><Key>Game</Key><Value>CNC</Value></CustomField>
  <CustomField><Key>Juice Session Id</Key><Value>juice-1</Value></CustomField>
  <CustomField><Key>Data</Key><Value>Assert Info:
Z:\EALA\World\Render\WorldRenderer.cpp(393):
Invalid split distances for slice 0 and slice 1.
Function: fb::buildCascadingShadowViews
Module: Static
Expression: outSliceSplitDistances[sliceIt] &lt; outSliceSplitDistances[sliceIt+1]
Callstack:
0x00A221D4
0x00A239B6</Value></CustomField>
</DevTrackBug>
XML;

        $this->sentry('POST', '/testkey/submit/', $xml)->assertOk();

        $issue = ReportIssue::query()->first();
        $this->assertNotNull($issue);
        $this->assertSame('assert', $issue->type);
        $this->assertSame('cnc', $issue->game);
        $this->assertSame('WorldRenderer.cpp:393 fb::buildCascadingShadowViews', $issue->category_id);
        $this->assertSame('WorldRenderer.cpp:393', $issue->culprit);

        $event = $issue->events()->first();
        $this->assertNotNull($event);
        $this->assertStringContainsString('Severity: Major', (string) $event->context_data);
        $this->assertStringContainsString('Invalid split distances', (string) $event->context_data);
        $this->assertStringContainsString('outSliceSplitDistances', (string) $event->context_data);
        $this->assertStringContainsString("0x00A221D4\n0x00A239B6", (string) $event->stack);
        $this->assertSame('juice-1', $event->session_id);
    }

    public function test_dedicated_assert_wrapper_is_stored(): void
    {
        $xml = <<<'XML'
<?xml version="1.0" encoding="UTF-8"?>
<report>
  <type>assert</type>
  <categoryid>EXCEPTION level=3221225477</categoryid>
  <contextdata>[2026-09-26] ACCESS_VIOLATION</contextdata>
</report>
XML;

        $this->sentry('POST', '/testkey/assert/', $xml)->assertOk();

        $this->assertDatabaseHas('report_issues', [
            'type' => 'assert',
            'category_id' => 'EXCEPTION level=3221225477',
        ]);
    }
}
