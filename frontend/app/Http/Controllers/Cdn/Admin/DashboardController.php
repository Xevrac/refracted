<?php

namespace App\Http\Controllers\Cdn\Admin;

use App\Http\Controllers\Controller;
use App\Models\Cdn\CdnPackage;
use App\Services\Cdn\CdnMetrics;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\View\View;

class DashboardController extends Controller
{
    public function login(): View
    {
        return view('cdn.admin.login');
    }

    public function dashboard(Request $request, CdnMetrics $metrics): View
    {
        return view('cdn.admin.dashboard', [
            'snapshot' => $metrics->snapshot(CdnMetrics::normalizeRange($request->query('range'))),
            'recent' => CdnPackage::query()->latest()->limit(8)->get(),
        ]);
    }

    public function metrics(Request $request, CdnMetrics $metrics): JsonResponse
    {
        return response()->json($metrics->snapshot(CdnMetrics::normalizeRange($request->query('range'))));
    }
}
