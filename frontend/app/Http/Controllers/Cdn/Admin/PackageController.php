<?php

namespace App\Http\Controllers\Cdn\Admin;

use App\Http\Controllers\Controller;
use App\Models\Cdn\CdnPackage;
use App\Services\Cdn\CdnArtifactGuard;
use App\Services\Cdn\CdnCatalog;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Storage;
use Illuminate\View\View;

class PackageController extends Controller
{
    public function __construct(private readonly CdnCatalog $catalog) {}

    public function index(): View
    {
        $packages = CdnPackage::query()
            ->withCount('artifacts')
            ->orderByDesc('updated_at')
            ->paginate(30);

        return view('cdn.admin.packages.index', compact('packages'));
    }

    public function create(): View
    {
        return view('cdn.admin.packages.form', [
            'package' => new CdnPackage([
                'kind' => CdnPackage::KIND_PRISM,
                'title_id' => 'cnc',
                'channel' => 'release',
                'version' => '0.1.0',
                'revision' => 1,
                'status' => 'draft',
            ]),
        ]);
    }

    public function store(Request $request): RedirectResponse
    {
        $data = $this->validated($request);
        $data['revision'] = $this->catalog->nextRevision(
            $data['title_id'],
            $data['channel'],
            $data['version']
        );
        $data['created_by'] = $request->user()?->id;
        $data['status'] = 'draft';
        $data['is_latest'] = false;

        $package = CdnPackage::query()->create($data);

        return redirect()
            ->route('cdn.admin.packages.show', $package)
            ->with('status', 'Package draft created (revision '.$package->revision.').');
    }

    public function show(CdnPackage $package): View
    {
        $package->load('artifacts');

        return view('cdn.admin.packages.show', compact('package'));
    }

    public function edit(CdnPackage $package): View
    {
        return view('cdn.admin.packages.form', compact('package'));
    }

    public function update(Request $request, CdnPackage $package): RedirectResponse
    {
        $data = $this->validated($request);
        $package->update($data);

        return redirect()
            ->route('cdn.admin.packages.show', $package)
            ->with('status', 'Package updated.');
    }

    public function publish(CdnPackage $package): RedirectResponse
    {
        if ($package->artifacts()->count() === 0) {
            return back()->withErrors(['package' => 'Add at least one artifact before publishing.']);
        }

        if ($package->channel === 'release') {
            $violations = app(CdnArtifactGuard::class)->releaseViolations(
                $package->artifacts()->pluck('path'),
                $package->kind
            );
            if (count($violations) > 0) {
                return back()->withErrors([
                    'package' => 'Release publish blocked — remove these artifacts: '
                        .implode(', ', $violations),
                ]);
            }
        }

        $this->catalog->publish($package);

        return back()->with('status', 'Published. Launchers on this channel will resolve this package.');
    }

    public function unpublish(CdnPackage $package): RedirectResponse
    {
        $this->catalog->unpublish($package);

        return back()->with('status', 'Unpublished (draft).');
    }

    public function destroy(CdnPackage $package): RedirectResponse
    {
        $package->delete();

        return redirect()
            ->route('cdn.admin.packages.index')
            ->with('status', 'Package removed.');
    }

    public function uploadArtifact(Request $request, CdnPackage $package): RedirectResponse
    {
        $data = $request->validate([
            'folder' => ['nullable', 'string', 'max:256', 'not_regex:/(^|[\\\/])\.\.([\\\/]|$)/'],
            'files' => ['required', 'array', 'min:1'],
            'files.*' => ['file', 'max:102400'],
            'confirm_risky' => ['sometimes', 'boolean'],
        ]);

        $folder = trim(str_replace('\\', '/', (string) ($data['folder'] ?? '')), '/');
        $guard = app(CdnArtifactGuard::class);

        // Guard every file before storing any, so a blocked file rejects the whole batch.
        $uploads = [];
        foreach ($request->file('files') as $uploaded) {
            $name = basename(str_replace('\\', '/', $uploaded->getClientOriginalName()));
            $path = $folder !== '' ? "{$folder}/{$name}" : $name;
            $guard->assertMayUpload($path, $package->channel, $request->boolean('confirm_risky'), $package->kind);
            $uploads[$path] = $uploaded;
        }

        foreach ($uploads as $path => $uploaded) {
            $contents = file_get_contents($uploaded->getRealPath());
            if ($contents === false) {
                return back()->withErrors(['files' => "Could not read {$path}."]);
            }

            $this->catalog->storeArtifact($package, $path, $contents, $uploaded->getMimeType());
        }

        $count = count($uploads);

        return back()->with('status', $count === 1
            ? 'Uploaded '.array_key_first($uploads).'.'
            : "Uploaded {$count} files.");
    }

    public function destroyArtifact(CdnPackage $package, int $artifact): RedirectResponse
    {
        $row = $package->artifacts()->whereKey($artifact)->firstOrFail();
        $disk = Storage::disk(config('cdn.disk', 'cdn'));
        if ($disk->exists($row->storage_path)) {
            $disk->delete($row->storage_path);
        }
        $row->delete();

        return back()->with('status', 'Artifact removed.');
    }

    protected function validated(Request $request): array
    {
        $data = $request->validate([
            'kind' => ['required', 'in:'.implode(',', CdnPackage::KINDS)],
            'title_id' => [
                'required_if:kind,'.CdnPackage::KIND_PRISM,
                'nullable',
                'string',
                'max:32',
                'in:'.implode(',', array_keys(config('cdn.titles', []))),
            ],
            'channel' => ['required', 'in:'.implode(',', config('cdn.channels', ['release', 'debug']))],
            'version' => ['required', 'string', 'max:64'],
            'label' => ['nullable', 'string', 'max:120'],
            'notes' => ['nullable', 'string', 'max:5000'],
        ]);

        $data['title_id'] = $data['kind'] === CdnPackage::KIND_REFRACTED
            ? CdnPackage::KIND_REFRACTED
            : strtolower($data['title_id']);
        $data['channel'] = strtolower($data['channel']);

        return $data;
    }
}
