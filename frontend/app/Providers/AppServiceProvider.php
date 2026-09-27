<?php

namespace App\Providers;

use Illuminate\Cache\RateLimiting\Limit;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\RateLimiter;
use Illuminate\Support\ServiceProvider;
use SocialiteProviders\Discord\Provider as DiscordProvider;
use SocialiteProviders\Manager\SocialiteWasCalled;

class AppServiceProvider extends ServiceProvider
{
    public function register(): void
    {
        //
    }

    public function boot(): void
    {
        $this->app['events']->listen(function (SocialiteWasCalled $event): void {
            $event->extendSocialite('discord', DiscordProvider::class);
        });

        // Report ingest is unauthenticated by design, so cap it per client. A
        // player crashing repeatedly is normal; hundreds a minute is not.
        RateLimiter::for('report-ingest', fn (Request $request) => Limit::perMinute(
            (int) config('reports.rate_limit_per_minute')
        )->by($request->ip()));

        RateLimiter::for('nexus-device-start', fn (Request $request) => Limit::perMinute(10)->by($request->ip()));
        RateLimiter::for('nexus-device-poll', fn (Request $request) => Limit::perMinute(60)->by($request->ip()));
        RateLimiter::for('nexus-device-revoke', fn (Request $request) => Limit::perMinute(20)->by($request->ip()));
        RateLimiter::for('nexus-device-session', fn (Request $request) => Limit::perMinute(30)->by($request->ip()));
        RateLimiter::for('nexus-device-approve', fn (Request $request) => Limit::perMinute(10)->by(
            optional($request->user())->id ?: $request->ip()
        ));
        RateLimiter::for('cdn-public', fn (Request $request) => Limit::perMinute(
            (int) config('cdn.rate_limit_per_minute', 120)
        )->by($request->ip()));
    }
}
