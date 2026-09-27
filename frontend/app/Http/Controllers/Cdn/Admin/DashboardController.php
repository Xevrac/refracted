<?php

namespace App\Http\Controllers\Cdn\Admin;

use App\Http\Controllers\Controller;
use App\Models\Cdn\CdnPackage;
use App\Services\Cdn\CdnMetrics;
use Illuminate\Http\JsonResponse;
use Illuminate\View\View;

class DashboardController extends Controller
{
    public function login(): View
    {
        return view('cdn.admin.login');
    }

    public function dashboard(CdnMetrics $metrics): View
    {
        return view('cdn.admin.dashboard', [
            'snapshot' => $metrics->snapshot(24),
            'recent' => CdnPackage::query()->latest()->limit(8)->get(),
        ]);
    }

    public function metrics(CdnMetrics $metrics): JsonResponse
    {
        return response()->json($metrics->snapshot(24));
    }
}
