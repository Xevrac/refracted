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
    }
}
