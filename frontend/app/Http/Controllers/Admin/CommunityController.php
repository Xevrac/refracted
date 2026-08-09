<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use App\Models\Community;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\View\View;

class CommunityController extends Controller
{
    public function create(): View
    {
        return view('admin.communities.form', [
            'community' => new Community([
                'tagline' => 'Powered by Refracted',
                'sort_order' => (Community::query()->max('sort_order') ?? 0) + 1,
                'is_published' => true,
            ]),
        ]);
    }

    public function store(Request $request): RedirectResponse
    {
        $data = $this->validated($request);
        $data['image_path'] = $this->storeUpload($request, 'image', 'communities');
        $data['icon_path'] = $this->storeUpload($request, 'icon', 'communities/icons');

        Community::query()->create($data);

        return redirect()
            ->route('admin.games.index')
            ->with('status', 'Community added.');
    }

    public function edit(Community $community): View
    {
        return view('admin.communities.form', compact('community'));
    }

    public function update(Request $request, Community $community): RedirectResponse
    {
        $data = $this->validated($request);

        if ($request->hasFile('image')) {
            $this->deleteStored($community->image_path);
            $data['image_path'] = $this->storeUpload($request, 'image', 'communities');
        }

        if ($request->hasFile('icon')) {
            $this->deleteStored($community->icon_path);
            $data['icon_path'] = $this->storeUpload($request, 'icon', 'communities/icons');
        }

        $community->update($data);

        return redirect()
            ->route('admin.games.index')
            ->with('status', 'Community updated.');
    }

    public function destroy(Community $community): RedirectResponse
    {
        $community->deleteMedia();
        $community->delete();

        return redirect()
            ->route('admin.games.index')
            ->with('status', 'Community removed.');
    }

    public function reorder(Request $request): \Illuminate\Http\JsonResponse
    {
        $data = $request->validate([
            'order' => ['required', 'array', 'min:1'],
            'order.*' => ['integer', 'distinct', 'exists:communities,id'],
        ]);

        foreach ($data['order'] as $index => $id) {
            Community::query()->whereKey($id)->update(['sort_order' => $index + 1]);
        }

        return response()->json(['ok' => true]);
    }

    protected function validated(Request $request): array
    {
        $data = $request->validate([
            'name' => ['required', 'string', 'max:120'],
            'tagline' => ['required', 'string', 'max:120'],
            'blurb' => ['nullable', 'string', 'max:1000'],
            'url' => ['nullable', 'url', 'max:255'],
            'sort_order' => ['nullable', 'integer', 'min:0', 'max:9999'],
            'is_published' => ['sometimes', 'boolean'],
            'image' => ['nullable', 'image', 'max:5120'],
            'icon' => ['nullable', 'image', 'max:2048'],
        ]);

        $data['is_published'] = $request->boolean('is_published');
        $data['sort_order'] = (int) ($data['sort_order'] ?? 0);
        unset($data['image'], $data['icon']);

        return $data;
    }

    protected function storeUpload(Request $request, string $field, string $folder): ?string
    {
        if (! $request->hasFile($field)) {
            return null;
        }

        return $request->file($field)->store($folder, 'public');
    }

    protected function deleteStored(?string $path): void
    {
        if (filled($path) && ! str_starts_with($path, 'images/') && \Illuminate\Support\Facades\Storage::disk('public')->exists($path)) {
            \Illuminate\Support\Facades\Storage::disk('public')->delete($path);
        }
    }
}
