<?php

namespace App\Http\Controllers\Auth;

use App\Http\Controllers\Controller;
use App\Services\DiscordAuthService;
use App\Support\AdminHosts;
use App\Support\CdnHosts;
use App\Support\NexusHosts;
use Illuminate\Auth\AuthenticationException;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Laravel\Socialite\Facades\Socialite;

class DiscordAuthController extends Controller
{
    public function redirect(Request $request): RedirectResponse
    {
        $intent = $request->routeIs('nexus.discord.login')
            ? DiscordAuthService::INTENT_NEXUS
            : DiscordAuthService::INTENT_ADMIN;

        // Ignore query-string intent overrides.
        $request->session()->put('auth.intent', $intent);
        $request->session()->put(
            'auth.return_cdn',
            $request->routeIs('cdn.discord.login') || CdnHosts::isCdnHost($request->getHost())
        );
        $redirectUri = $this->oauthRedirectUri($request);
        $request->session()->put('auth.discord_redirect_uri', $redirectUri);

        return Socialite::driver('discord')
            ->redirectUrl($redirectUri)
            ->scopes(['identify', 'email'])
            ->redirect();
    }

    public function callback(Request $request, DiscordAuthService $discordAuth): RedirectResponse
    {
        $intent = (string) $request->session()->pull('auth.intent', '');
        if ($intent !== DiscordAuthService::INTENT_ADMIN && $intent !== DiscordAuthService::INTENT_NEXUS) {
            return redirect()
                ->to(AdminHosts::loginUrl())
                ->withErrors(['discord' => 'Sign-in session expired. Start again from the login page.']);
        }

        $redirectUri = (string) $request->session()->pull(
            'auth.discord_redirect_uri',
            $this->oauthRedirectUri($request)
        );

        try {
            $discordUser = Socialite::driver('discord')
                ->redirectUrl($redirectUri)
                ->user();
            $user = $discordAuth->resolveForLogin($discordUser, $intent);

            Auth::login($user, true);
            $request->session()->regenerate();

            return redirect()->intended($this->homeFor($user, $intent));
        } catch (AuthenticationException $e) {
            return redirect()
                ->to($this->loginUrlFor($intent))
                ->withErrors(['discord' => $e->getMessage()]);
        } catch (\Throwable) {
            return redirect()
                ->to($this->loginUrlFor($intent))
                ->withErrors(['discord' => 'Discord sign-in could not be completed.']);
        }
    }

    public function logout(Request $request): RedirectResponse
    {
        $fromNexus = NexusHosts::isNexusHost($request->getHost())
            || $request->routeIs('nexus.*');
        $fromCdn = CdnHosts::isCdnHost($request->getHost())
            || $request->routeIs('cdn.*');

        Auth::logout();
        $request->session()->invalidate();
        $request->session()->regenerateToken();

        if ($fromNexus) {
            return redirect()->to(NexusHosts::loginUrl());
        }

        if ($fromCdn) {
            return redirect()->to(CdnHosts::loginUrl());
        }

        return redirect()->to(AdminHosts::loginUrl());
    }

    protected function homeFor($user, string $intent): string
    {
        if ($intent === DiscordAuthService::INTENT_NEXUS) {
            return NexusHosts::profileUrl();
        }

        if (session()->pull('auth.return_cdn')) {
            return CdnHosts::adminUrl();
        }

        return $user->isAdmin()
            ? route('admin.games.index')
            : AdminHosts::sentryUrl();
    }

    protected function loginUrlFor(string $intent): string
    {
        if ($intent === DiscordAuthService::INTENT_NEXUS) {
            return NexusHosts::loginUrl();
        }

        if (session('auth.return_cdn') || CdnHosts::isCdnHost()) {
            return CdnHosts::loginUrl();
        }

        return AdminHosts::loginUrl();
    }

    /**
     * Discord needs a full absolute URL. Config may be path-only (/gateway/callback)
     * so each FQDN (nexus / sentry / site) can share one Discord app.
     */
    protected function oauthRedirectUri(Request $request): string
    {
        $configured = trim((string) config('services.discord.redirect', '/gateway/callback'));

        if (preg_match('#^https?://#i', $configured) === 1) {
            return (string) preg_replace('#^http://#i', 'https://', $configured);
        }

        $path = '/'.ltrim($configured !== '' ? $configured : 'gateway/callback', '/');

        if (AdminHosts::localDashboard()) {
            return url($path);
        }

        return 'https://'.$request->getHost().$path;
    }
}
