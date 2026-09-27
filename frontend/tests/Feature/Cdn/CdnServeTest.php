<?php

namespace Tests\Feature\Cdn;

use App\Models\Cdn\CdnPackage;
use App\Models\User;
use App\Services\Cdn\CdnCatalog;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Storage;
use Tests\TestCase;

class CdnServeTest extends TestCase
{
    use RefreshDatabase;

    protected function setUp(): void
    {
        parent::setUp();
        Storage::fake('cdn');
        config([
            'cdn.require_launcher_headers' => true,
            'cdn.client_keys' => [],
            'cdn.disk' => 'cdn',
        ]);
    }

    public function test_manifest_requires_launcher_headers(): void
    {
        $this->getJson('/cdn/prism/cnc/manifest.json')
            ->assertForbidden()
            ->assertJsonPath('error', 'missing_or_invalid_client');
    }

    public function test_manifest_returns_published_package(): void
    {
        $catalog = app(CdnCatalog::class);
        $package = CdnPackage::query()->create([
            'title_id' => 'cnc',
            'channel' => 'release',
            'version' => '1.2.3',
            'revision' => 1,
            'status' => 'draft',
        ]);
        $catalog->storeArtifact($package, 'prism.dll', 'hello-prism');
        $catalog->publish($package);

        $response = $this->withHeaders($this->launcherHeaders('cnc', 'release'))
            ->getJson('/cdn/prism/cnc/manifest.json');

        $response->assertOk()
            ->assertJsonPath('titleId', 'cnc')
            ->assertJsonPath('version', '1.2.3')
            ->assertJsonPath('revision', 1)
            ->assertJsonPath('files.0.path', 'prism.dll');
    }

    public function test_file_download_streams_artifact(): void
    {
        $catalog = app(CdnCatalog::class);
        $package = CdnPackage::query()->create([
            'title_id' => 'cnc',
            'channel' => 'debug',
            'version' => '0.9.0',
            'revision' => 2,
            'status' => 'draft',
        ]);
        $catalog->storeArtifact($package, 'bin/a.dll', 'ABCDEF');
        $catalog->publish($package);

        $this->withHeaders($this->launcherHeaders('cnc', 'debug'))
            ->get('/cdn/prism/cnc/0.9.0/r2/files/bin/a.dll')
            ->assertOk()
            ->assertHeader('X-Content-SHA256', hash('sha256', 'ABCDEF'));
    }

    public function test_admin_dashboard_requires_admin(): void
    {
        $user = User::factory()->create(['is_admin' => false]);

        $this->actingAs($user)
            ->get('/cdn/admin')
            ->assertForbidden();
    }

    public function test_admin_can_open_dashboard(): void
    {
        $user = User::factory()->create(['is_admin' => true]);

        $this->actingAs($user)
            ->get('/cdn/admin')
            ->assertOk();
    }

    /** @return array<string, string> */
    private function launcherHeaders(string $title, string $channel): array
    {
        return [
            'X-Refracted-Client' => 'launcher',
            'X-Refracted-Launcher-Version' => '0.1.0',
            'X-Refracted-Title' => $title,
            'X-Refracted-Channel' => $channel,
            'X-Refracted-Realm' => 'preview-au',
        ];
    }
}
