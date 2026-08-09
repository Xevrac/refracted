<?php

namespace App\Http\Controllers\Auth;

use App\Http\Controllers\Controller;
use App\Services\DiscordAuthService;
use Illuminate\Auth\AuthenticationException;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Laravel\Socialite\Facades\Socialite;

class DiscordAuthController extends Controller
{
    public function redirect(): RedirectResponse
    {
        return Socialite::driver('discord')
            ->scopes(['identify', 'email'])
            ->redirect();
    }

    public function callback(Request $request, DiscordAuthService $discordAuth): RedirectResponse
    {
        try {
            $discordUser = Socialite::driver('discord')->user();
            $user = $discordAuth->resolveForLogin($discordUser);

            Auth::login($user, true);
            $request->session()->regenerate();

            return redirect()->intended(route('admin.games.index'));
        } catch (AuthenticationException $e) {
            return redirect()
                ->route('admin.login')
                ->withErrors(['discord' => $e->getMessage()]);
        } catch (\Throwable) {
            return redirect()
                ->route('admin.login')
                ->withErrors(['discord' => 'Discord sign-in could not be completed.']);
        }
    }

    public function logout(Request $request): RedirectResponse
    {
        Auth::logout();
        $request->session()->invalidate();
        $request->session()->regenerateToken();

        return redirect()->route('admin.login');
    }
}
