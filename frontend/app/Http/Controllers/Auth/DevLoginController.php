<?php

namespace App\Http\Controllers\Auth;

use App\Http\Controllers\Controller;
use App\Models\User;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Validation\ValidationException;

class DevLoginController extends Controller
{
    public function store(Request $request): RedirectResponse
    {
        abort_unless(app()->environment('local'), 404);

        $credentials = $request->validate([
            'email' => ['required', 'email'],
            'password' => ['required', 'string'],
        ]);

        if (! Auth::attempt($credentials, true)) {
            throw ValidationException::withMessages([
                'email' => 'Those local credentials do not match.',
            ]);
        }

        /** @var User $user */
        $user = Auth::user();

        if (! $user->isAdmin()) {
            Auth::logout();

            throw ValidationException::withMessages([
                'email' => 'That account is not an admin.',
            ]);
        }

        $request->session()->regenerate();

        return redirect()->intended(route('admin.games.index'));
    }
}
