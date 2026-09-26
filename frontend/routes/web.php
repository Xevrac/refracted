<?php

use App\Http\Controllers\Admin\CommunityController;
use App\Http\Controllers\Admin\GameController;
use App\Http\Controllers\Admin\ReportController;
use App\Http\Controllers\Auth\DevLoginController;
use App\Http\Controllers\Auth\DiscordAuthController;
use App\Http\Controllers\ReportIngestController;
use App\Models\Community;
use App\Models\Game;
use App\Support\GameReport;
use App\Support\ReportIngest;
use Illuminate\Support\Facades\Route;

Route::get('/', function () {
    return view('welcome', [
        'titles' => Game::query()->published()->ordered()->get(),
        'communities' => Community::query()->published()->ordered()->get(),
    ]);
});

Route::view('legal', 'legal')->name('legal');

Route::get('gateway/callback', [DiscordAuthController::class, 'callback'])
    ->name('discord.callback');

/*
 * sentry.refracted.au
 */
Route::group(ReportIngest::routeGroup(), function () {
    if (ReportIngest::domain()) {
        Route::get('/', function () {
            if (auth()->check() && auth()->user()->isStaff()) {
                return redirect()->route('admin.reports.index');
            }

            return redirect()->route('admin.login');
        })->name('reports.home');
    }

    if (filled(config('reports.key'))) {
        $category = implode('|', GameReport::TYPES);

        foreach (ReportIngest::uris() as $uri) {
            Route::post($uri, [ReportIngestController::class, 'store'])
                ->where('category', $category)
                ->middleware('throttle:report-ingest')
                ->name('reports.ingest.'.(str_ends_with($uri, '/') ? 'slash' : 'bare'));
        }
    }

    Route::middleware(['auth', 'staff'])->prefix('admin')->name('admin.')->group(function () {
        Route::get('reports', [ReportController::class, 'index'])->name('reports.index');
        Route::get('reports/events/{event}/screenshot', [ReportController::class, 'screenshot'])
            ->name('reports.screenshot');
        Route::get('reports/events/{event}/raw', [ReportController::class, 'raw'])->name('reports.raw');
        Route::get('reports/{issue}', [ReportController::class, 'show'])->name('reports.show');
        Route::put('reports/{issue}', [ReportController::class, 'update'])->name('reports.update');
        Route::delete('reports/{issue}', [ReportController::class, 'destroy'])->name('reports.destroy');
    });
});

Route::middleware('guest')->group(function () {
    Route::view('admin/login', 'admin.login')->name('admin.login');
    Route::get('auth/discord', [DiscordAuthController::class, 'redirect'])->name('discord.login');

    if (app()->environment('local')) {
        Route::post('admin/login/dev', [DevLoginController::class, 'store'])->name('admin.login.dev');
    }
});

Route::middleware('auth')->prefix('admin')->name('admin.')->group(function () {
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

    Route::post('communities/reorder', [CommunityController::class, 'reorder'])->name('communities.reorder');
    Route::get('communities/create', [CommunityController::class, 'create'])->name('communities.create');
    Route::post('communities', [CommunityController::class, 'store'])->name('communities.store');
    Route::get('communities/{community}/edit', [CommunityController::class, 'edit'])->name('communities.edit');
    Route::put('communities/{community}', [CommunityController::class, 'update'])->name('communities.update');
    Route::delete('communities/{community}', [CommunityController::class, 'destroy'])->name('communities.destroy');
});
