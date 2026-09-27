<?php

namespace App\Http\Controllers\Cdn\Admin;

use App\Http\Controllers\Controller;
use App\Models\User;
use App\Support\CdnHosts;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Str;

class DevLoginController extends Controller
{
    public function store(Request $request): RedirectResponse
    {
        abort_unless(CdnHosts::localDashboard() && ! app()->isProduction(), 404);

        $user = User::query()->where('is_admin', true)->orderBy('id')->first();

        if (! $user) {
            $user = User::query()->create([
                'name' => 'CDN Dev Admin',
                'email' => 'cdn-dev@refracted.local',
                'password' => Hash::make(Str::random(40)),
                'email_verified_at' => now(),
                'discord_id' => 'cdn-dev-admin',
                'discord_username' => 'cdn-dev',
                'is_admin' => true,
                'last_login_at' => now(),
            ]);
        }

        Auth::login($user, true);
        $request->session()->regenerate();

        return redirect()->intended(route('cdn.admin.dashboard'));
    }
}
