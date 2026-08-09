<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use App\Models\Community;
use App\Models\Game;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\View\View;

class GameController extends Controller
{
    public function index(): View
    {
        return view('admin.games.index', [
            'games' => Game::query()->ordered()->get(),
            'communities' => Community::query()->ordered()->get(),
        ]);
    }

    public function create(): View
    {
        return view('admin.games.form', [
            'game' => new Game([
                'status' => 'Available',
                'sort_order' => (Game::query()->max('sort_order') ?? 0) + 1,
                'is_published' => true,
            ]),
        ]);
    }

    public function store(Request $request): RedirectResponse
    {
        $data = $this->validated($request);
        $data['image_path'] = $this->storeImage($request, 'games');

        Game::query()->create($data);

        return redirect()
            ->route('admin.games.index')
            ->with('status', 'Game added.');
    }

    public function edit(Game $game): View
    {
        return view('admin.games.form', compact('game'));
    }

    public function update(Request $request, Game $game): RedirectResponse
    {
        $data = $this->validated($request);

        if ($request->hasFile('image')) {
            $game->deleteImage();
            $data['image_path'] = $this->storeImage($request, 'games');
        }

        $game->update($data);

        return redirect()
            ->route('admin.games.index')
            ->with('status', 'Game updated.');
    }

    public function destroy(Game $game): RedirectResponse
    {
        $game->deleteImage();
        $game->delete();

        return redirect()
            ->route('admin.games.index')
            ->with('status', 'Game removed.');
    }

    public function reorder(Request $request): \Illuminate\Http\JsonResponse
    {
        $data = $request->validate([
            'order' => ['required', 'array', 'min:1'],
            'order.*' => ['integer', 'distinct', 'exists:games,id'],
        ]);

        foreach ($data['order'] as $index => $id) {
            Game::query()->whereKey($id)->update(['sort_order' => $index + 1]);
        }

        return response()->json(['ok' => true]);
    }

    protected function validated(Request $request): array
    {
        $data = $request->validate([
            'name' => ['required', 'string', 'max:120'],
            'status' => ['required', 'string', 'max:60'],
            'blurb' => ['nullable', 'string', 'max:1000'],
            'url' => ['nullable', 'url', 'max:255'],
            'sort_order' => ['nullable', 'integer', 'min:0', 'max:9999'],
            'is_published' => ['sometimes', 'boolean'],
            'image' => ['nullable', 'image', 'max:5120'],
        ]);

        $data['is_published'] = $request->boolean('is_published');
        $data['sort_order'] = (int) ($data['sort_order'] ?? 0);
        unset($data['image']);

        return $data;
    }

    protected function storeImage(Request $request, string $folder): ?string
    {
        if (! $request->hasFile('image')) {
            return null;
        }

        return $request->file('image')->store($folder, 'public');
    }
}
