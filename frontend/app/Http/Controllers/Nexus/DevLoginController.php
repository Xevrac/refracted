<?php

namespace App\Http\Controllers\Nexus;

use App\Http\Controllers\Controller;
use App\Models\User;
use App\Services\Nexus\NexusProvisioner;
use App\Support\NexusHosts;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Str;

/** Local Nexus sign-in. */
class DevLoginController extends Controller
{
    public function store(Request $request, NexusProvisioner $provisioner): RedirectResponse
    {
        abort_unless(
            \App\Support\NexusHosts::localDashboard() && ! app()->isProduction(),
            404
        );

        $validated = $request->validate([
            'email' => ['required', 'email'],
            'name' => ['nullable', 'string', 'max:64'],
        ]);

        $email = $validated['email'];
        $name = $validated['name'] ?: Str::before($email, '@');
        $discordId = 'dev-'.substr(hash('sha256', $email), 0, 16);

        $access = app(\App\Services\Nexus\NexusAccessControl::class);
        try {
            $access->assertNotBanned($discordId);
            $already = \App\Models\Nexus\NexusUser::query()->where('discord_id', $discordId)->exists();
            $access->assertMayRegister($discordId, $already);
        } catch (\Illuminate\Auth\AuthenticationException $e) {
            return back()->withErrors(['email' => $e->getMessage()]);
        }

        $user = User::query()->updateOrCreate(
            ['email' => $email],
            [
                'name' => $name,
                'password' => Hash::make(Str::random(40)),
                'email_verified_at' => now(),
                'discord_id' => $discordId,
                'discord_username' => $name,
                'last_login_at' => now(),
            ],
        );

        $provisioner->ensureForWebsiteUser($user, $name);

        Auth::login($user, true);
        $request->session()->regenerate();

        return redirect()->intended(NexusHosts::profileUrl());
    }
}
