<?php

namespace Tests\Feature\Nexus;

use App\Models\Nexus\NexusAuthSession;
use App\Models\User;
use App\Services\Nexus\NexusDeviceCodeService;
use App\Services\Nexus\NexusProvisioner;
use Illuminate\Support\Facades\Hash;
use Tests\RefreshNexusDatabase;
use Tests\TestCase;

class NexusDeviceCodeTest extends TestCase
{
    use RefreshNexusDatabase;

    public function test_device_start_returns_codes(): void
    {
        $response = $this->postJson(route('nexus.device.start'));

        $response->assertOk()
            ->assertJsonStructure(['userCode', 'deviceCode', 'interval', 'expiresIn', 'verificationUri']);

        $this->assertNotEmpty($response->json('userCode'));
        $this->assertNotEmpty($response->json('deviceCode'));
    }

    public function test_poll_is_pending_until_approved(): void
    {
        $start = $this->postJson(route('nexus.device.start'))->json();

        $this->postJson(route('nexus.device.poll'), ['deviceCode' => $start['deviceCode']])
            ->assertOk()
            ->assertJson(['status' => 'pending']);
    }

    public function test_approve_issues_session_and_poll_returns_ticket_once(): void
    {
        $websiteUser = User::factory()->create([
            'discord_id' => 'device-player-1',
            'discord_username' => 'DevicePlayer',
            'password' => Hash::make('secret'),
        ]);
        app(NexusProvisioner::class)->ensureForWebsiteUser($websiteUser);

        $start = app(NexusDeviceCodeService::class)->start();

        $this->actingAs($websiteUser)
            ->post(route('nexus.device.approve'), [
                'user_code' => $start['userCode'],
                'confirm_code' => $start['userCode'],
            ])
            ->assertRedirect();

        $this->assertSame(1, NexusAuthSession::query()->count());

        $poll = $this->postJson(route('nexus.device.poll'), ['deviceCode' => $start['deviceCode']])
            ->assertOk()
            ->assertJsonPath('status', 'approved')
            ->json();

        $this->assertNotEmpty($poll['token']);
        $this->assertNotEmpty($poll['jwt']);
        $this->assertSame('DevicePlayer', $poll['displayName']);

        $this->postJson(route('nexus.device.poll'), ['deviceCode' => $start['deviceCode']])
            ->assertOk()
            ->assertJson(['status' => 'expired']);
    }

    public function test_device_start_accepts_game_and_stores_session_telemetry(): void
    {
        $websiteUser = User::factory()->create([
            'discord_id' => 'device-player-geo',
            'discord_username' => 'GeoPlayer',
            'password' => Hash::make('secret'),
        ]);
        app(NexusProvisioner::class)->ensureForWebsiteUser($websiteUser);

        $start = $this->postJson(route('nexus.device.start'), ['game' => 'cnc'])
            ->assertOk()
            ->json();

        $this->actingAs($websiteUser)
            ->withHeader('CF-IPCountry', 'AU')
            ->post(route('nexus.device.approve'), [
                'user_code' => $start['userCode'],
                'confirm_code' => $start['userCode'],
            ])
            ->assertRedirect();

        $session = NexusAuthSession::query()->first();
        $this->assertNotNull($session);
        $this->assertSame('cnc', $session->game_id);
        $this->assertSame('AU', $session->country_code);
    }
}
