<?php

namespace Tests\Feature\Nexus;

use App\Models\Nexus\NexusUser;
use App\Models\User;
use App\Services\DiscordAuthService;
use App\Services\Nexus\NexusProvisioner;
use Illuminate\Support\Facades\Hash;
use Laravel\Socialite\Contracts\User as SocialiteUser;
use Mockery;
use Tests\RefreshNexusDatabase;
use Tests\TestCase;

class NexusProvisionerTest extends TestCase
{
    use RefreshNexusDatabase;

    public function test_provisioner_creates_nexus_user_and_persona(): void
    {
        $websiteUser = User::factory()->create([
            'discord_id' => '123456789012345678',
            'discord_username' => 'ShadowLead',
            'email' => 'shadow@example.com',
            'is_admin' => false,
        ]);

        $nexusUser = app(NexusProvisioner::class)->ensureForWebsiteUser($websiteUser);

        $this->assertNotNull($nexusUser->id);
        $this->assertGreaterThanOrEqual(1_000_000_000_000, $nexusUser->id);
        $this->assertSame('123456789012345678', $nexusUser->discord_id);
        $this->assertSame($websiteUser->id, $nexusUser->web_user_id);
        $this->assertNotSame($websiteUser->id, $nexusUser->id);

        $persona = $nexusUser->defaultPersona();
        $this->assertNotNull($persona);
        $this->assertSame('ShadowLead', $persona->display_name);
        $this->assertGreaterThanOrEqual(1_000_000_000_000, $persona->id);
        $this->assertNotSame($nexusUser->id, $persona->id);
    }

    public function test_provisioner_is_idempotent_for_same_discord_id(): void
    {
        $websiteUser = User::factory()->create([
            'discord_id' => '999',
            'discord_username' => 'Twice',
        ]);

        $first = app(NexusProvisioner::class)->ensureForWebsiteUser($websiteUser);
        $second = app(NexusProvisioner::class)->ensureForWebsiteUser($websiteUser);

        $this->assertSame($first->id, $second->id);
        $this->assertSame(1, NexusUser::query()->where('discord_id', '999')->count());
        $this->assertSame(1, $second->personas()->count());
    }

    public function test_player_discord_login_provisions_without_admin_allowlist(): void
    {
        config(['services.discord.admin_ids' => 'only-admins']);

        $socialite = Mockery::mock(SocialiteUser::class);
        $socialite->shouldReceive('getId')->andReturn('player-discord-1');
        $socialite->shouldReceive('getNickname')->andReturn('CNCPlayer');
        $socialite->shouldReceive('getName')->andReturn('CNC Player');
        $socialite->shouldReceive('getEmail')->andReturn('player@example.com');
        $socialite->shouldReceive('getAvatar')->andReturn(null);

        $user = app(DiscordAuthService::class)->resolveForLogin(
            $socialite,
            DiscordAuthService::INTENT_NEXUS
        );

        $this->assertFalse($user->isAdmin());
        $this->assertSame('player-discord-1', $user->discord_id);

        $nexus = app(NexusProvisioner::class)->findForWebsiteUser($user);
        $this->assertNotNull($nexus);
        $this->assertSame('CNCPlayer', $nexus->defaultPersona()?->display_name);
    }

    public function test_admin_allowlist_does_not_promote_on_nexus_intent(): void
    {
        config(['services.discord.admin_ids' => 'admin-also-player']);

        $socialite = Mockery::mock(SocialiteUser::class);
        $socialite->shouldReceive('getId')->andReturn('admin-also-player');
        $socialite->shouldReceive('getNickname')->andReturn('Both');
        $socialite->shouldReceive('getName')->andReturn('Both');
        $socialite->shouldReceive('getEmail')->andReturn('both@example.com');
        $socialite->shouldReceive('getAvatar')->andReturn(null);

        $user = app(DiscordAuthService::class)->resolveForLogin(
            $socialite,
            DiscordAuthService::INTENT_NEXUS
        );

        $this->assertFalse($user->isAdmin());
    }

    public function test_admin_discord_login_still_requires_allowlist(): void
    {
        config(['services.discord.admin_ids' => 'allowed-admin']);

        $socialite = Mockery::mock(SocialiteUser::class);
        $socialite->shouldReceive('getId')->andReturn('not-allowed');
        $socialite->shouldReceive('getNickname')->andReturn('Nope');
        $socialite->shouldReceive('getName')->andReturn('Nope');
        $socialite->shouldReceive('getEmail')->andReturn('nope@example.com');
        $socialite->shouldReceive('getAvatar')->andReturn(null);

        $this->expectException(\Illuminate\Auth\AuthenticationException::class);

        app(DiscordAuthService::class)->resolveForLogin(
            $socialite,
            DiscordAuthService::INTENT_ADMIN
        );
    }

    public function test_profile_page_shows_nexus_identity(): void
    {
        $websiteUser = User::factory()->create([
            'discord_id' => '555',
            'discord_username' => 'ProfileUser',
            'password' => Hash::make('secret'),
        ]);

        app(NexusProvisioner::class)->ensureForWebsiteUser($websiteUser);

        $this->actingAs($websiteUser)
            ->get(route('nexus.profile'))
            ->assertOk()
            ->assertSee('ProfileUser')
            ->assertSee('Nexus')
            ->assertSee('User id');
    }

    public function test_guest_is_redirected_from_profile_to_nexus_login(): void
    {
        $this->get(route('nexus.profile'))
            ->assertRedirect(route('nexus.login'));
    }

    public function test_my_games_lists_titles_from_sessions(): void
    {
        $websiteUser = User::factory()->create([
            'discord_id' => 'games-user-1',
            'discord_username' => 'Gamer',
        ]);
        $nexus = app(NexusProvisioner::class)->ensureForWebsiteUser($websiteUser);
        $persona = $nexus->defaultPersona();

        \App\Models\Nexus\NexusAuthSession::query()->create([
            'user_id' => $nexus->id,
            'persona_id' => $persona->id,
            'token_hash' => hash('sha256', 'games-token-1'),
            'jwt_id' => 'jwt-games-1',
            'expires_at' => now()->utc()->addDay()->format('Y-m-d H:i:s'),
            'created_at' => now()->utc()->subDays(3)->format('Y-m-d H:i:s'),
            'last_seen_at' => now()->utc()->subHour()->format('Y-m-d H:i:s'),
            'game_id' => 'cnc',
        ]);

        $this->actingAs($websiteUser)
            ->get(route('nexus.profile.games'))
            ->assertOk()
            ->assertSee('My Games')
            ->assertSee('Command & Conquer');
    }

    public function test_display_name_can_be_updated(): void
    {
        $websiteUser = User::factory()->create([
            'discord_id' => '777',
            'discord_username' => 'OldName',
        ]);

        $nexus = app(NexusProvisioner::class)->ensureForWebsiteUser($websiteUser);

        $this->actingAs($websiteUser)
            ->put(route('nexus.profile.update'), ['display_name' => 'NewPersona'])
            ->assertRedirect(route('nexus.profile'));

        $persona = $nexus->fresh()->defaultPersona();
        $this->assertSame('NewPersona', $persona?->display_name);
        $this->assertNotNull($persona?->display_name_changed_at);
    }

    public function test_display_name_change_is_limited_to_once_per_cooldown(): void
    {
        config(['nexus.display_name_cooldown_days' => 30]);

        $websiteUser = User::factory()->create([
            'discord_id' => '778',
            'discord_username' => 'FirstName',
        ]);

        $nexus = app(NexusProvisioner::class)->ensureForWebsiteUser($websiteUser);

        $this->actingAs($websiteUser)
            ->put(route('nexus.profile.update'), ['display_name' => 'SecondName'])
            ->assertRedirect(route('nexus.profile'));

        $this->actingAs($websiteUser)
            ->from(route('nexus.profile'))
            ->put(route('nexus.profile.update'), ['display_name' => 'ThirdName'])
            ->assertRedirect(route('nexus.profile'))
            ->assertSessionHasErrors('display_name');

        $this->assertSame('SecondName', $nexus->fresh()->defaultPersona()?->display_name);
    }

    public function test_display_name_can_be_changed_again_after_cooldown(): void
    {
        config(['nexus.display_name_cooldown_days' => 30]);

        $websiteUser = User::factory()->create([
            'discord_id' => '779',
            'discord_username' => 'Alpha',
        ]);

        $nexus = app(NexusProvisioner::class)->ensureForWebsiteUser($websiteUser);
        $persona = $nexus->defaultPersona();
        $persona->forceFill([
            'display_name' => 'Alpha',
            'display_name_changed_at' => now()->utc()->subDays(31)->format('Y-m-d H:i:s'),
        ])->save();

        $this->actingAs($websiteUser)
            ->put(route('nexus.profile.update'), ['display_name' => 'Bravo'])
            ->assertRedirect(route('nexus.profile'))
            ->assertSessionDoesntHaveErrors();

        $this->assertSame('Bravo', $nexus->fresh()->defaultPersona()?->display_name);
    }

    public function test_display_name_cannot_be_taken_by_another_persona(): void
    {
        config(['nexus.display_name_cooldown_days' => 30]);

        $holder = User::factory()->create([
            'discord_id' => '880',
            'discord_username' => 'TakenName',
        ]);
        $claimer = User::factory()->create([
            'discord_id' => '881',
            'discord_username' => 'OtherPlayer',
        ]);

        app(NexusProvisioner::class)->ensureForWebsiteUser($holder);
        $claimerNexus = app(NexusProvisioner::class)->ensureForWebsiteUser($claimer);
        $claimerNexus->defaultPersona()?->forceFill([
            'display_name_changed_at' => now()->utc()->subDays(31)->format('Y-m-d H:i:s'),
        ])->save();

        $this->actingAs($claimer)
            ->from(route('nexus.profile'))
            ->put(route('nexus.profile.update'), ['display_name' => 'takenname'])
            ->assertRedirect(route('nexus.profile'))
            ->assertSessionHasErrors('display_name');

        $this->assertSame('OtherPlayer', $claimerNexus->fresh()->defaultPersona()?->display_name);
    }

    public function test_admin_can_bypass_display_name_cooldown(): void
    {
        config([
            'nexus.display_name_cooldown_days' => 30,
            'services.discord.admin_ids' => 'admin-bypass-1',
        ]);

        $admin = User::factory()->create([
            'discord_id' => 'admin-bypass-1',
            'discord_username' => 'AdminOld',
            'is_admin' => true,
        ]);

        $nexus = app(NexusProvisioner::class)->ensureForWebsiteUser($admin);
        $nexus->defaultPersona()?->forceFill([
            'display_name' => 'AdminOld',
            'display_name_changed_at' => now()->utc()->format('Y-m-d H:i:s'),
        ])->save();

        $this->actingAs($admin)
            ->put(route('nexus.profile.update'), ['display_name' => 'AdminNew'])
            ->assertRedirect(route('nexus.profile'))
            ->assertSessionHas('status', 'Persona display name updated.');

        $this->assertSame('AdminNew', $nexus->fresh()->defaultPersona()?->display_name);
    }
}
