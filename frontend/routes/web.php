<?php

use App\Http\Controllers\Admin\GameController;
use App\Http\Controllers\Admin\ReportController;
use App\Http\Controllers\Auth\DevLoginController;
use App\Http\Controllers\Auth\DiscordAuthController;
use App\Http\Controllers\Cdn\Admin\DashboardController as CdnDashboardController;
use App\Http\Controllers\Cdn\Admin\DevLoginController as CdnDevLoginController;
use App\Http\Controllers\Cdn\Admin\PackageController as CdnPackageController;
use App\Http\Controllers\Cdn\ServeController as CdnServeController;
use App\Http\Controllers\Nexus\AdminController as NexusAdminController;
use App\Http\Controllers\Nexus\DeviceCodeController as NexusDeviceCodeController;
use App\Http\Controllers\Nexus\DevLoginController as NexusDevLoginController;
use App\Http\Controllers\Nexus\ProfileController as NexusProfileController;
use App\Http\Controllers\ReportIngestController;
use App\Models\Game;
use App\Support\AdminHosts;
use App\Support\CdnHosts;
use App\Support\GameReport;
use App\Support\NexusHosts;
use Illuminate\Support\Facades\Route;

Route::get('/', function () {
    return view('welcome', [
        'titles' => Game::query()->published()->ordered()->get(),
    ]);
});

Route::view('legal', 'legal')->name('legal');

Route::get('gateway/callback', [DiscordAuthController::class, 'callback'])
    ->name('discord.callback');

/*
 * CDN — cdn.refracted.au (local prefix /cdn)
 */
Route::group(CdnHosts::routeGroup(), function () {
    Route::get('/', [CdnServeController::class, 'home'])->name('cdn.home');

    Route::middleware(['cdn.launcher', 'throttle:cdn-public'])->group(function () {
        Route::get('prism/{title}/manifest.json', [CdnServeController::class, 'manifest'])
            ->where('title', '[A-Za-z0-9_-]+')
            ->name('cdn.prism.manifest');

        Route::get('prism/{title}/{version}/{revision}/files/{path}', [CdnServeController::class, 'file'])
            ->where([
                'title' => '[A-Za-z0-9_-]+',
                'version' => '[A-Za-z0-9._-]+',
                'revision' => 'r[0-9]+',
                'path' => '.*',
            ])
            ->name('cdn.prism.file');

        Route::get('refracted/manifest.json', [CdnServeController::class, 'refractedManifest'])
            ->name('cdn.refracted.manifest');

        Route::get('refracted/{version}/{revision}/files/{path}', [CdnServeController::class, 'refractedFile'])
            ->where([
                'version' => '[A-Za-z0-9._-]+',
                'revision' => 'r[0-9]+',
                'path' => '.*',
            ])
            ->name('cdn.refracted.file');
    });

    Route::middleware('guest')->group(function () {
        Route::get('admin/login', [CdnDashboardController::class, 'login'])->name('cdn.admin.login');
        Route::get('auth/discord', [DiscordAuthController::class, 'redirect'])->name('cdn.discord.login');

        if (CdnHosts::localDashboard()) {
            Route::post('admin/login/dev', [CdnDevLoginController::class, 'store'])->name('cdn.admin.login.dev');
        }
    });

    Route::middleware(['auth', 'admin'])->prefix('admin')->name('cdn.admin.')->group(function () {
        Route::get('/', [CdnDashboardController::class, 'dashboard'])->name('dashboard');
        Route::get('metrics', [CdnDashboardController::class, 'metrics'])->name('metrics');
        Route::post('logout', [DiscordAuthController::class, 'logout'])->name('logout');

        Route::get('packages', [CdnPackageController::class, 'index'])->name('packages.index');
        Route::get('packages/create', [CdnPackageController::class, 'create'])->name('packages.create');
        Route::post('packages', [CdnPackageController::class, 'store'])->name('packages.store');
        Route::get('packages/{package}', [CdnPackageController::class, 'show'])->name('packages.show');
        Route::get('packages/{package}/edit', [CdnPackageController::class, 'edit'])->name('packages.edit');
        Route::put('packages/{package}', [CdnPackageController::class, 'update'])->name('packages.update');
        Route::delete('packages/{package}', [CdnPackageController::class, 'destroy'])->name('packages.destroy');
        Route::post('packages/{package}/publish', [CdnPackageController::class, 'publish'])->name('packages.publish');
        Route::post('packages/{package}/unpublish', [CdnPackageController::class, 'unpublish'])->name('packages.unpublish');
        Route::post('packages/{package}/artifacts', [CdnPackageController::class, 'uploadArtifact'])->name('packages.artifacts.store');
        Route::delete('packages/{package}/artifacts/{artifact}', [CdnPackageController::class, 'destroyArtifact'])->name('packages.artifacts.destroy');
    });
});

/*
 * Nexus
 */
Route::group(NexusHosts::routeGroup(), function () {
    Route::get('/', [NexusProfileController::class, 'home'])->name('nexus.home');

    Route::post('device/start', [NexusDeviceCodeController::class, 'start'])
        ->middleware('throttle:nexus-device-start')
        ->name('nexus.device.start');
    Route::post('device/poll', [NexusDeviceCodeController::class, 'poll'])
        ->middleware('throttle:nexus-device-poll')
        ->name('nexus.device.poll');
    Route::post('device/revoke', [NexusDeviceCodeController::class, 'revoke'])
        ->middleware('throttle:nexus-device-revoke')
        ->name('nexus.device.revoke');
    Route::post('device/session', [NexusDeviceCodeController::class, 'session'])
        ->middleware('throttle:nexus-device-session')
        ->name('nexus.device.session');

    Route::middleware('guest')->group(function () {
        Route::get('login', [NexusProfileController::class, 'login'])->name('nexus.login');
        Route::get('auth/discord', [DiscordAuthController::class, 'redirect'])->name('nexus.discord.login');

        if (NexusHosts::localDashboard()) {
            Route::post('login/dev', [NexusDevLoginController::class, 'store'])->name('nexus.login.dev');
        }
    });

    Route::middleware('auth')->group(function () {
        Route::get('profile', [NexusProfileController::class, 'show'])->name('nexus.profile');
        Route::get('profile/games', [NexusProfileController::class, 'games'])->name('nexus.profile.games');
        Route::put('profile', [NexusProfileController::class, 'update'])->name('nexus.profile.update');
        Route::post('logout', [NexusProfileController::class, 'logout'])->name('nexus.logout');

        Route::get('device', [NexusDeviceCodeController::class, 'show'])->name('nexus.device');
        Route::post('device/approve', [NexusDeviceCodeController::class, 'approve'])
            ->middleware('throttle:nexus-device-approve')
            ->name('nexus.device.approve');

        Route::middleware('nexus.admin')->prefix('admin')->group(function () {
            Route::get('/', [NexusAdminController::class, 'index'])->name('nexus.admin');
            Route::get('lookup', [NexusAdminController::class, 'lookup'])->name('nexus.admin.lookup');
            Route::get('gatekeeper', [NexusAdminController::class, 'gatekeeper'])->name('nexus.admin.gatekeeper');
            Route::get('sessions', [NexusAdminController::class, 'sessions'])->name('nexus.admin.sessions');
            Route::post('sessions/{session}/revoke', [NexusAdminController::class, 'revokeSession'])
                ->name('nexus.admin.sessions.revoke');
            Route::post('registrations', [NexusAdminController::class, 'updateRegistrations'])
                ->name('nexus.admin.registrations');
            Route::post('whitelist/mode', [NexusAdminController::class, 'updateWhitelistMode'])
                ->name('nexus.admin.whitelist.mode');
            Route::post('whitelist', [NexusAdminController::class, 'storeWhitelist'])
                ->name('nexus.admin.whitelist.store');
            Route::delete('whitelist', [NexusAdminController::class, 'destroyWhitelist'])
                ->name('nexus.admin.whitelist.destroy');
            Route::post('bans', [NexusAdminController::class, 'storeBan'])
                ->name('nexus.admin.bans.store');
            Route::post('bans/{ban}/lift', [NexusAdminController::class, 'liftBan'])
                ->name('nexus.admin.bans.lift');
        });
    });
});

/*
 * sentry.refracted.au
 */
Route::group(AdminHosts::routeGroup(), function () {
    if (AdminHosts::domain()) {
        Route::get('/', function () {
            if (auth()->check() && auth()->user()->isStaff()) {
                return redirect()->route('admin.reports.index');
            }

            return redirect()->route('admin.login');
        })->name('reports.home');
    }

    if (filled(config('reports.key'))) {
        $category = implode('|', GameReport::TYPES);

        foreach (AdminHosts::uris() as $uri) {
            Route::post($uri, [ReportIngestController::class, 'store'])
                ->where('category', $category)
                ->middleware('throttle:report-ingest')
                ->name('reports.ingest.'.(str_ends_with($uri, '/') ? 'slash' : 'bare'));
        }
    }
});

$sentryUi = AdminHosts::dashboardHost()
    ? ['domain' => AdminHosts::dashboardHost()]
    : [];

Route::group($sentryUi, function () {
    Route::middleware('guest')->group(function () {
        Route::view('admin/login', 'admin.login')->name('admin.login');
        Route::get('auth/discord', [DiscordAuthController::class, 'redirect'])->name('discord.login');

        if (AdminHosts::localDashboard()) {
            Route::post('admin/login/dev', [DevLoginController::class, 'store'])->name('admin.login.dev');
        }
    });

    Route::middleware('auth')->prefix('admin')->name('admin.')->group(function () {
        Route::post('logout', [DiscordAuthController::class, 'logout'])->name('logout');
    });

    Route::middleware(['auth', 'staff'])->prefix('admin')->name('admin.')->group(function () {
        Route::get('/', fn () => redirect()->route('admin.reports.index'));

        Route::get('reports/inbox', [ReportController::class, 'inbox'])->name('reports.inbox');
        Route::post('reports/read-all', [ReportController::class, 'readAll'])->name('reports.read-all');
        Route::post('reports/{issue}/read', [ReportController::class, 'read'])->name('reports.read');
        Route::get('reports', [ReportController::class, 'index'])->name('reports.index');
        Route::get('reports/events/{event}/screenshot', [ReportController::class, 'screenshot'])
            ->name('reports.screenshot');
        Route::get('reports/events/{event}/raw', [ReportController::class, 'raw'])->name('reports.raw');
        Route::get('reports/{issue}', [ReportController::class, 'show'])->name('reports.show');
        Route::put('reports/{issue}', [ReportController::class, 'update'])->name('reports.update');
        Route::delete('reports/{issue}', [ReportController::class, 'destroy'])->name('reports.destroy');
    });

    if (AdminHosts::localDashboard()) {
        Route::middleware(['auth', 'admin'])->prefix('admin')->name('admin.')->group(function () {
            Route::post('games/reorder', [GameController::class, 'reorder'])->name('games.reorder');
            Route::get('games', [GameController::class, 'index'])->name('games.index');
            Route::get('games/create', [GameController::class, 'create'])->name('games.create');
            Route::post('games', [GameController::class, 'store'])->name('games.store');
            Route::get('games/{game}/edit', [GameController::class, 'edit'])->name('games.edit');
            Route::put('games/{game}', [GameController::class, 'update'])->name('games.update');
            Route::delete('games/{game}', [GameController::class, 'destroy'])->name('games.destroy');
        });
    }
});

/*
 * refracted.au — site Games admin only (not on sentry host).
 */
if ($site = AdminHosts::siteHost()) {
    Route::group(['domain' => $site], function () {
        Route::middleware('guest')->group(function () {
            Route::view('admin/login', 'admin.login')->name('site.admin.login');
            Route::get('auth/discord', [DiscordAuthController::class, 'redirect'])->name('site.discord.login');
        });

        Route::middleware('auth')->prefix('admin')->name('site.admin.')->group(function () {
            Route::post('logout', [DiscordAuthController::class, 'logout'])->name('logout');
        });

        Route::middleware(['auth', 'admin'])->prefix('admin')->name('admin.')->group(function () {
            Route::get('/', fn () => redirect()->route('admin.games.index'));

            Route::post('games/reorder', [GameController::class, 'reorder'])->name('games.reorder');
            Route::get('games', [GameController::class, 'index'])->name('games.index');
            Route::get('games/create', [GameController::class, 'create'])->name('games.create');
            Route::post('games', [GameController::class, 'store'])->name('games.store');
            Route::get('games/{game}/edit', [GameController::class, 'edit'])->name('games.edit');
            Route::put('games/{game}', [GameController::class, 'update'])->name('games.update');
            Route::delete('games/{game}', [GameController::class, 'destroy'])->name('games.destroy');
        });
    });
}
