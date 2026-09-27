<?php

namespace App\Http\Controllers\Nexus;

use App\Http\Controllers\Controller;
use App\Services\Nexus\NexusDeviceCodeService;
use App\Services\Nexus\NexusSessionIssuer;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Str;
use Illuminate\View\View;

class DeviceCodeController extends Controller
{
    public function start(Request $request, NexusDeviceCodeService $devices): JsonResponse
    {
        $game = (string) $request->input('game', $request->input('gameId', ''));

        return response()->json($devices->start($game !== '' ? $game : null));
    }

    public function poll(Request $request, NexusDeviceCodeService $devices): JsonResponse
    {
        $deviceCode = (string) $request->input('deviceCode', $request->input('device_code', ''));
        if ($deviceCode === '' || strlen($deviceCode) < 32 || strlen($deviceCode) > 128) {
            return response()->json(['status' => 'expired'], 400);
        }

        if (! preg_match('/^[a-f0-9]+$/i', $deviceCode)) {
            return response()->json(['status' => 'expired'], 400);
        }

        return response()->json($devices->poll($deviceCode));
    }

    public function show(Request $request, NexusDeviceCodeService $devices): View
    {
        $code = Str::upper(trim((string) $request->query('code', '')));
        $pending = $code !== '' ? $devices->publicByUserCode($code) : null;

        return view('nexus.device', [
            'userCode' => $code,
            'pending' => $pending,
            'expired' => $code !== '' && $pending === null,
            'confirmRequired' => $pending !== null && ($pending['status'] ?? '') === 'pending',
        ]);
    }

    public function approve(Request $request, NexusDeviceCodeService $devices): RedirectResponse
    {
        $validated = $request->validate([
            'user_code' => ['required', 'string', 'max:16'],
            'confirm_code' => ['required', 'string', 'max:16'],
        ]);

        $userCode = Str::upper(trim($validated['user_code']));
        $confirm = Str::upper(trim($validated['confirm_code']));

        if ($confirm !== $userCode) {
            return redirect()
                ->route('nexus.device', ['code' => $userCode])
                ->withErrors(['confirm_code' => 'Type the code exactly to approve this launcher.']);
        }

        try {
            $access = app(\App\Services\Nexus\NexusAccessControl::class);
            $access->assertNotBanned((string) ($request->user()->discord_id ?? ''));
            $devices->approve($userCode, $request->user());
        } catch (\Illuminate\Auth\AuthenticationException $e) {
            return redirect()
                ->route('nexus.device', ['code' => $userCode])
                ->withErrors(['user_code' => $e->getMessage()]);
        } catch (\Throwable $e) {
            return redirect()
                ->route('nexus.device', ['code' => $userCode])
                ->withErrors(['user_code' => $e->getMessage()]);
        }

        return redirect()
            ->route('nexus.device', ['code' => $userCode])
            ->with('status', 'Launcher approved. You can close this tab.');
    }

    public function revoke(Request $request, NexusSessionIssuer $sessions): JsonResponse
    {
        $token = (string) $request->input('token', '');
        if ($token === '' || strlen($token) > 128) {
            return response()->json(['ok' => false], 400);
        }

        $sessions->revokeByToken($token);

        return response()->json(['ok' => true]);
    }
}
