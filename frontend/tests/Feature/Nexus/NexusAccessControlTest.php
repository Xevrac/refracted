<?php

namespace Tests\Feature\Nexus;

use App\Models\Nexus\NexusBan;
use App\Models\Nexus\NexusSignupWhitelistEntry;
use App\Models\Nexus\NexusUser;
use App\Models\User;
use App\Services\DiscordAuthService;
use App\Services\Nexus\NexusAccessControl;
use App\Services\Nexus\NexusProvisioner;
use Illuminate\Support\Facades\Hash;
use Laravel\Socialite\Contracts\User as SocialiteUser;
use Mockery;
use Tests\RefreshNexusDatabase;
use Tests\TestCase;

class NexusAccessControlTest extends TestCase
{
    use RefreshNexusDatabase;

    public function test_closed_registrations_block_new_discord_signups(): void
    {
        app(NexusAccessControl::class)->setRegistrationsOpen(false);

        $socialite = $this->mockDiscord('new-player-1', 'Newbie');

        $this->expectException(\Illuminate\Auth\AuthenticationException::class);
        $this->expectExceptionMessage('closed');

        app(DiscordAuthService::class)->resolveForLogin($socialite, DiscordAuthService::INTENT_NEXUS);
    }

    public function test_existing_user_can_sign_in_when_registrations_closed(): void
    {
        $websiteUser = User::factory()->create([
            'discord_id' => 'existing-1',
            'discord_username' => 'Existing',
        ]);
        app(NexusProvisioner::class)->ensureForWebsiteUser($websiteUser);
        app(NexusAccessControl::class)->setRegistrationsOpen(false);

        $socialite = $this->mockDiscord('existing-1', 'Existing');
        $user = app(DiscordAuthService::class)->resolveForLogin(
            $socialite,
            DiscordAuthService::INTENT_NEXUS
        );

        $this->assertSame('existing-1', $user->discord_id);
    }

    public function test_whitelist_blocks_non_listed_discord(): void
    {
        $access = app(NexusAccessControl::class);
        $access->setRegistrationsOpen(true);
        $access->setWhitelistEnabled(true);
        $access->addWhitelist('111222333', 'tester', null);

        $socialite = $this->mockDiscord('999888777', 'Outsider');

        $this->expectException(\Illuminate\Auth\AuthenticationException::class);
        $this->expectExceptionMessage('whitelist');

        app(DiscordAuthService::class)->resolveForLogin($socialite, DiscordAuthService::INTENT_NEXUS);
    }

    public function test_whitelisted_discord_can_register(): void
    {
        $access = app(NexusAccessControl::class);
        $access->setRegistrationsOpen(true);
        $access->setWhitelistEnabled(true);
        $access->addWhitelist('444555666', 'ok', null);

        $socialite = $this->mockDiscord('444555666', 'Listed');
        $user = app(DiscordAuthService::class)->resolveForLogin(
            $socialite,
            DiscordAuthService::INTENT_NEXUS
        );

        $this->assertSame('444555666', $user->discord_id);
        $this->assertNotNull(NexusUser::query()->where('discord_id', '444555666')->first());
    }

    public function test_whitelist_overrides_closed_registrations(): void
    {
        $access = app(NexusAccessControl::class);
        $access->setRegistrationsOpen(false);
        $access->setWhitelistEnabled(false);
        $access->addWhitelist('777888999', 'invite', null);

        $socialite = $this->mockDiscord('777888999', 'Invitee');
        $user = app(DiscordAuthService::class)->resolveForLogin(
            $socialite,
            DiscordAuthService::INTENT_NEXUS
        );

        $this->assertSame('777888999', $user->discord_id);
        $this->assertNotNull(NexusUser::query()->where('discord_id', '777888999')->first());
    }

    public function test_ban_blocks_login_and_can_be_lifted(): void
    {
        $admin = User::factory()->create(['is_admin' => true, 'discord_id' => 'admin-1']);
        $player = User::factory()->create([
            'discord_id' => 'banned-1',
            'discord_username' => 'Banned',
            'password' => Hash::make('x'),
        ]);
        $nexus = app(NexusProvisioner::class)->ensureForWebsiteUser($player);

        $access = app(NexusAccessControl::class);
        $ban = $access->ban($nexus, 'banned-1', 'griefing', null, $admin);
        $this->assertTrue($ban->isActive());

        $socialite = $this->mockDiscord('banned-1', 'Banned');
        try {
            app(DiscordAuthService::class)->resolveForLogin($socialite, DiscordAuthService::INTENT_NEXUS);
            $this->fail('Expected ban exception');
        } catch (\Illuminate\Auth\AuthenticationException $e) {
            $this->assertStringContainsString('banned', strtolower($e->getMessage()));
        }

        $access->lift($ban, $admin);
        $user = app(DiscordAuthService::class)->resolveForLogin($socialite, DiscordAuthService::INTENT_NEXUS);
        $this->assertSame('banned-1', $user->discord_id);
    }

    public function test_admin_panel_requires_admin(): void
    {
        $player = User::factory()->create([
            'discord_id' => 'player-admin-gate',
            'is_admin' => false,
            'password' => Hash::make('x'),
        ]);

        $this->actingAs($player)
            ->get(route('nexus.admin'))
            ->assertForbidden();
    }

    public function test_admin_can_open_panel_and_toggle_registrations(): void
    {
        config(['services.discord.admin_ids' => 'panel-admin']);
        $admin = User::factory()->create([
            'discord_id' => 'panel-admin',
            'is_admin' => true,
            'password' => Hash::make('x'),
        ]);

        $this->actingAs($admin)
            ->get(route('nexus.admin.gatekeeper'))
            ->assertOk()
            ->assertSee('Registrations')
            ->assertSee('Gatekeeper');

        $this->actingAs($admin)
            ->post(route('nexus.admin.registrations'), [
                'registrations_open' => '0',
                'confirm' => '1',
            ])
            ->assertRedirect(route('nexus.admin.gatekeeper'));

        $this->assertFalse(app(NexusAccessControl::class)->registrationsOpen());
    }

    public function test_admin_player_lookup_finds_by_username_and_discord(): void
    {
        config(['services.discord.admin_ids' => 'lookup-admin']);
        $admin = User::factory()->create([
            'discord_id' => 'lookup-admin',
            'is_admin' => true,
            'password' => Hash::make('x'),
        ]);
        $player = User::factory()->create([
            'discord_id' => '998877665544',
            'discord_username' => 'ShadowLead',
        ]);
        app(NexusProvisioner::class)->ensureForWebsiteUser($player);

        $this->actingAs($admin)
            ->get(route('nexus.admin.lookup', ['q' => 'Shadow']))
            ->assertOk()
            ->assertSee('Player lookup')
            ->assertSee('ShadowLead')
            ->assertSee('last game')
            ->assertSee('country')
            ->assertSee('Active sessions');

        $this->actingAs($admin)
            ->get(route('nexus.admin.lookup', ['q' => '998877665544']))
            ->assertOk()
            ->assertSee('998877665544');

        $this->actingAs($admin)
            ->get(route('nexus.admin'))
            ->assertOk()
            ->assertSee('User id')
            ->assertSee('Last seen');
    }

    protected function mockDiscord(string $id, string $nick): SocialiteUser
    {
        $socialite = Mockery::mock(SocialiteUser::class);
        $socialite->shouldReceive('getId')->andReturn($id);
        $socialite->shouldReceive('getNickname')->andReturn($nick);
        $socialite->shouldReceive('getName')->andReturn($nick);
        $socialite->shouldReceive('getEmail')->andReturn($id.'@example.com');
        $socialite->shouldReceive('getAvatar')->andReturn(null);

        return $socialite;
    }
}
