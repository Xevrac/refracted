<?php

namespace App\Http\Controllers\Cdn;

use App\Http\Controllers\Controller;
use App\Models\Cdn\CdnPackage;
use App\Services\Cdn\CdnCatalog;
use App\Services\Cdn\CdnMetrics;
use App\Support\CdnHosts;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\BinaryFileResponse;

class ServeController extends Controller
{
    public function __construct(
        private readonly CdnCatalog $catalog,
        private readonly CdnMetrics $metrics,
    ) {}

    public function home(): JsonResponse
    {
        return response()->json(['status' => 'OK'], 200, [], JSON_PRETTY_PRINT);
    }

    public function refractedManifest(Request $request): JsonResponse
    {
        return $this->manifest($request, CdnPackage::KIND_REFRACTED, CdnPackage::KIND_REFRACTED);
    }

    public function refractedFile(
        Request $request,
        string $version,
        string $revision,
        string $path,
    ): BinaryFileResponse|JsonResponse {
        return $this->file($request, CdnPackage::KIND_REFRACTED, $version, $revision, $path, CdnPackage::KIND_REFRACTED);
    }

    public function manifest(Request $request, string $title, string $kind = CdnPackage::KIND_PRISM): JsonResponse
    {
        $channel = $this->catalog->channelFromRequest($request);
        $headerTitle = strtolower(trim((string) $request->header('X-Refracted-Title', $title)));
        if ($headerTitle !== '' && strcasecmp($headerTitle, $title) !== 0) {
            $this->metrics->record('manifest', 403, 0, $title, $channel);

            return response()->json([
                'error' => 'title_mismatch',
                'message' => 'X-Refracted-Title must match the path title',
            ], 403);
        }

        $package = $this->catalog->resolvePublished($title, $channel, null, $kind);
        if (! $package) {
            $this->metrics->record('manifest', 404, 0, $title, $channel);

            return response()->json([
                'error' => 'not_found',
                'message' => "No published {$channel} package for {$title}",
            ], 404);
        }

        $payload = $this->catalog->manifestPayload($package, CdnHosts::homeUrl());
        $json = json_encode($payload, JSON_UNESCAPED_SLASHES);
        $bytes = strlen((string) $json);
        $this->metrics->record('manifest', 200, $bytes, $title, $channel);

        return response()->json($payload)->withHeaders([
            'Cache-Control' => 'public, max-age=60',
            'X-Refracted-Package-Version' => $package->version,
            'X-Refracted-Package-Revision' => (string) $package->revision,
            'X-Refracted-Package-Channel' => $package->channel,
        ]);
    }

    public function file(
        Request $request,
        string $title,
        string $version,
        string $revision,
        string $path,
        string $kind = CdnPackage::KIND_PRISM,
    ): BinaryFileResponse|JsonResponse {
        $channel = $this->catalog->channelFromRequest($request);
        $revisionNum = (int) ltrim($revision, 'rR');

        $package = $this->catalog->resolvePublished($title, $channel, $version, $kind);
        if (! $package || (int) $package->revision !== $revisionNum) {
            // Allow exact revision match even if not "latest"
            $package = CdnPackage::query()
                ->published()
                ->ofKind($kind)
                ->forTitleChannel($title, $channel)
                ->where('version', $version)
                ->where('revision', $revisionNum)
                ->with('artifacts')
                ->first();
        }

        if (! $package) {
            $this->metrics->record('file', 404, 0, $title, $channel);

            return response()->json(['error' => 'package_not_found'], 404);
        }

        $path = ltrim(str_replace('\\', '/', $path), '/');
        $artifact = $package->artifacts->firstWhere('path', $path);
        if (! $artifact || ! $artifact->existsOnDisk()) {
            $this->metrics->record('file', 404, 0, $title, $channel);

            return response()->json(['error' => 'file_not_found'], 404);
        }

        $this->metrics->record('file', 200, (int) $artifact->size_bytes, $title, $channel);
        if ($request->isMethod('GET')) {
            $this->metrics->recordDownload($package, $artifact->path, (int) $artifact->size_bytes);
        }

        return response()->file($artifact->absolutePath(), [
            'Content-Type' => $artifact->content_type ?: 'application/octet-stream',
            'Content-Length' => (string) $artifact->size_bytes,
            'ETag' => '"'.$artifact->sha256.'"',
            'Cache-Control' => 'public, max-age=3600',
            'X-Content-SHA256' => $artifact->sha256,
            'X-Refracted-Package-Version' => $package->version,
            'X-Refracted-Package-Revision' => (string) $package->revision,
        ]);
    }
}
