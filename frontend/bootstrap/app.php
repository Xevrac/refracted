<?php

use App\Support\AdminHosts;
use App\Support\CdnHosts;
use App\Support\NexusHosts;
use Illuminate\Foundation\Application;
use Illuminate\Foundation\Configuration\Exceptions;
use Illuminate\Foundation\Configuration\Middleware;
use Illuminate\Http\Request;

return Application::configure(basePath: dirname(__DIR__))
    ->withRouting(
        web: __DIR__.'/../routes/web.php',
        commands: __DIR__.'/../routes/console.php',
        health: '/up',
    )
    ->withMiddleware(function (Middleware $middleware): void {
        $trusted = env('TRUSTED_PROXIES');
        // Avoid app()->isProduction() here — env is not bound yet during middleware config.
        $isProd = env('APP_ENV', 'production') === 'production';
        if ($trusted === '*' || ($trusted === null && ! $isProd)) {
            $middleware->trustProxies(at: '*');
        } elseif (is_string($trusted) && $trusted !== '') {
            $middleware->trustProxies(at: array_values(array_filter(array_map(
                'trim',
                explode(',', $trusted)
            ))));
        }


        $middleware->trustHosts(
            at: array_values(array_filter([
                AdminHosts::domain(),
                NexusHosts::domain(),
                CdnHosts::domain(),
            ])),
            subdomains: true,
        );

        $middleware->alias([
            'admin' => \App\Http\Middleware\EnsureUserIsAdmin::class,
            'staff' => \App\Http\Middleware\EnsureUserIsStaff::class,
            'nexus.admin' => \App\Http\Middleware\EnsureNexusAdmin::class,
            'cdn.launcher' => \App\Http\Middleware\VerifyCdnLauncherHeaders::class,
        ]);

        $middleware->validateCsrfTokens(except: array_merge(
            AdminHosts::csrfExcept(),
            [
                'device/start',
                'device/poll',
                'device/revoke',
                'device/session',
                'nexus/device/start',
                'nexus/device/poll',
                'nexus/device/revoke',
                'nexus/device/session',
            ],
        ));

        $middleware->redirectGuestsTo(function (Request $request) {
            if (NexusHosts::isNexusHost($request->getHost())
                || str_starts_with($request->path(), 'nexus')) {
                return route('nexus.login');
            }

            if (CdnHosts::isCdnHost($request->getHost())
                || str_starts_with($request->path(), 'cdn')) {
                return route('cdn.admin.login');
            }

            return '/admin/login';
        });

        $middleware->redirectUsersTo(function (Request $request) {
            if (NexusHosts::isNexusHost($request->getHost())
                || str_starts_with($request->path(), 'nexus')) {
                return route('nexus.profile');
            }

            if (CdnHosts::isCdnHost($request->getHost())
                || str_starts_with($request->path(), 'cdn')) {
                return route('cdn.admin.dashboard');
            }

            return '/admin';
        });
    })->withExceptions(function (Exceptions $exceptions): void {
        $exceptions->shouldRenderJsonWhen(
            fn (Request $request) => $request->is('api/*'),
        );
    })->create();
