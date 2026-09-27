<?php

namespace App\Services\Cdn;

use App\Models\Cdn\CdnArtifact;
use App\Models\Cdn\CdnPackage;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Storage;
use Illuminate\Support\Str;

class CdnCatalog
{
    public function resolvePublished(
        string $titleId,
        string $channel,
        ?string $version = null,
        string $kind = CdnPackage::KIND_PRISM,
    ): ?CdnPackage {
        $titleId = Str::lower(trim($titleId));
        $channel = Str::lower(trim($channel));

        $query = CdnPackage::query()
            ->published()
            ->ofKind($kind)
            ->forTitleChannel($titleId, $channel)
            ->with('artifacts');

        if (filled($version)) {
            return $query
                ->where('version', $version)
                ->orderByDesc('revision')
                ->first();
        }

        return $query
            ->orderByDesc('is_latest')
            ->orderByDesc('published_at')
            ->orderByDesc('revision')
            ->first();
    }

    public function channelFromRequest(Request $request): string
    {
        $header = strtolower(trim((string) $request->header('X-Refracted-Channel', '')));
        if (in_array($header, config('cdn.channels', ['release', 'debug']), true)) {
            return $header;
        }

        $query = strtolower(trim((string) $request->query('channel', '')));
        if (in_array($query, config('cdn.channels', ['release', 'debug']), true)) {
            return $query;
        }

        return (string) config('cdn.default_channel', 'release');
    }

    public function publish(CdnPackage $package): CdnPackage
    {
        return DB::transaction(function () use ($package) {
            CdnPackage::query()
                ->forTitleChannel($package->title_id, $package->channel)
                ->whereKeyNot($package->id)
                ->update(['is_latest' => false]);

            $package->update([
                'status' => 'published',
                'is_latest' => true,
                'published_at' => now(),
            ]);

            return $package->fresh('artifacts');
        });
    }

    public function unpublish(CdnPackage $package): CdnPackage
    {
        $package->update([
            'status' => 'draft',
            'is_latest' => false,
        ]);

        $next = CdnPackage::query()
            ->published()
            ->forTitleChannel($package->title_id, $package->channel)
            ->orderByDesc('published_at')
            ->orderByDesc('revision')
            ->first();

        if ($next) {
            $next->update(['is_latest' => true]);
        }

        return $package->fresh();
    }

    public function storeArtifact(CdnPackage $package, string $relativePath, string $contents, ?string $contentType = null): CdnArtifact
    {
        $relativePath = ltrim(str_replace('\\', '/', $relativePath), '/');
        $sha = hash('sha256', $contents);
        $storagePath = sprintf(
            '%s/%s/%s_r%s/%s',
            $package->title_id,
            $package->channel,
            $package->version,
            $package->revision,
            $relativePath
        );

        Storage::disk(config('cdn.disk', 'cdn'))->put($storagePath, $contents);

        return CdnArtifact::query()->updateOrCreate(
            [
                'package_id' => $package->id,
                'path' => $relativePath,
            ],
            [
                'storage_path' => $storagePath,
                'sha256' => $sha,
                'size_bytes' => strlen($contents),
                'content_type' => $contentType,
            ],
        );
    }

    /** @return array<string, mixed> */
    public function manifestPayload(CdnPackage $package, string $cdnBaseUrl): array
    {
        $base = rtrim($cdnBaseUrl, '/');

        return [
            'kind' => $package->kind,
            'titleId' => $package->title_id,
            'channel' => $package->channel,
            'version' => $package->version,
            'revision' => $package->revision,
            'label' => $package->label,
            'publishedAt' => optional($package->published_at)?->toIso8601String(),
            'files' => $package->artifacts->map(fn (CdnArtifact $a) => [
                'path' => $a->path,
                'sha256' => $a->sha256,
                'size' => $a->size_bytes,
                'url' => "{$base}/{$package->urlPrefix()}/{$package->version}/r{$package->revision}/files/{$a->path}",
            ])->values()->all(),
        ];
    }

    public function nextRevision(string $titleId, string $channel, string $version): int
    {
        $max = CdnPackage::query()
            ->forTitleChannel($titleId, $channel)
            ->where('version', $version)
            ->max('revision');

        return ((int) $max) + 1;
    }
}
