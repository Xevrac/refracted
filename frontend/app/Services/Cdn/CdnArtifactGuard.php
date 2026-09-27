<?php

namespace App\Services\Cdn;

/**
 * Prevents shipping debug symbols / non-runtime junk on CDN packages.
 *
 * Prism release: DLL only (and not *debug* DLLs).
 * Refracted (launcher) release: any runtime file (exe, dll, json, web assets); symbols/leftovers blocked.
 * Debug: DLL + optional PDB with an explicit confirm; still blocks linker leftovers.
 */
class CdnArtifactGuard
{
    /** Extensions never appropriate for a release package. */
    public const BLOCKED_RELEASE_EXTENSIONS = [
        'pdb', 'exp', 'lib', 'map', 'ilk', 'iobj', 'ipdb', 'idb',
        'obj', 'o', 'a', 'bc', 'pch', 'ipch', 'tlog', 'lastbuildstate',
        'log', 'recipe', 'sarif',
    ];

    /** Linker / IDE leftovers — blocked on every channel unless confirmed (debug only). */
    public const ALWAYS_SUSPICIOUS_EXTENSIONS = [
        'exp', 'lib', 'map', 'ilk', 'iobj', 'ipdb', 'idb', 'obj', 'o', 'a', 'pch',
    ];

    /**
     * @return array{
     *   basename: string,
     *   extension: string,
     *   is_dll: bool,
     *   is_pdb: bool,
     *   looks_debug: bool,
     *   blocked_on_release: bool,
     *   requires_confirm: bool,
     *   severity: 'ok'|'warn'|'block',
     *   reasons: list<string>
     * }
     */
    public function inspect(string $relativePath, string $channel, string $kind = 'prism'): array
    {
        $path = ltrim(str_replace('\\', '/', $relativePath), '/');
        $basename = basename($path);
        $extension = strtolower(pathinfo($basename, PATHINFO_EXTENSION));
        $lowerBase = strtolower($basename);
        $channel = strtolower(trim($channel));

        $reasons = [];
        $isDll = $extension === 'dll';
        $isPdb = $extension === 'pdb';
        $looksDebug = $this->looksLikeDebugBinary($lowerBase, $extension);
        $dllOnly = $channel === 'release' && $kind !== 'refracted';

        if (in_array($extension, self::BLOCKED_RELEASE_EXTENSIONS, true)) {
            $reasons[] = ".{$extension} files are not shippable runtime artifacts";
        }

        if ($looksDebug) {
            $reasons[] = "Name looks like a debug build ({$basename})";
        }

        if ($dllOnly && ! $isDll) {
            $reasons[] = 'Release packages accept DLL files only';
        }

        if ($channel === 'release' && $isDll && $looksDebug) {
            $reasons[] = 'Debug-named DLLs must not ship on the release channel';
        }

        $blockedOnRelease = $channel === 'release' && (
            ($dllOnly && ! $isDll)
            || $looksDebug
            || in_array($extension, self::BLOCKED_RELEASE_EXTENSIONS, true)
        );

        $requiresConfirm = $looksDebug
            || $isPdb
            || in_array($extension, self::ALWAYS_SUSPICIOUS_EXTENSIONS, true)
            || ($dllOnly && ! $isDll);

        $severity = 'ok';
        if ($blockedOnRelease || ($channel === 'release' && $requiresConfirm)) {
            $severity = 'block';
        } elseif ($requiresConfirm) {
            $severity = 'warn';
        }

        return [
            'basename' => $basename,
            'extension' => $extension,
            'is_dll' => $isDll,
            'is_pdb' => $isPdb,
            'looks_debug' => $looksDebug,
            'blocked_on_release' => $blockedOnRelease,
            'requires_confirm' => $requiresConfirm,
            'severity' => $severity,
            'reasons' => array_values(array_unique($reasons)),
        ];
    }

    /**
     * @throws \Illuminate\Validation\ValidationException
     */
    public function assertMayUpload(string $relativePath, string $channel, bool $confirmed = false, string $kind = 'prism'): void
    {
        $info = $this->inspect($relativePath, $channel, $kind);

        if ($info['blocked_on_release'] || ($channel === 'release' && $info['severity'] === 'block')) {
            throw \Illuminate\Validation\ValidationException::withMessages([
                'file' => 'Refusing '.$info['basename'].' on release'
                    .(count($info['reasons']) ? ' — '.implode('; ', $info['reasons']) : '')
                    .'. Switch the package channel to debug, or upload a release build.',
            ]);
        }

        if ($info['requires_confirm'] && ! $confirmed) {
            throw \Illuminate\Validation\ValidationException::withMessages([
                'file' => 'Suspicious artifact '.$info['basename']
                    .' needs an explicit confirm (debug/PDB/linker leftover).',
            ]);
        }

        // Debug channel: still refuse pure linker leftovers even with confirm.
        if ($channel === 'debug'
            && in_array($info['extension'], self::ALWAYS_SUSPICIOUS_EXTENSIONS, true)
            && ! $info['is_pdb']) {
            throw \Illuminate\Validation\ValidationException::withMessages([
                'file' => '.'.$info['extension'].' files are never uploaded to the CDN (linker leftovers).',
            ]);
        }
    }

    /** @return list<string> */
    public function releaseViolations(iterable $paths, string $kind = 'prism'): array
    {
        $violations = [];
        foreach ($paths as $path) {
            $info = $this->inspect((string) $path, 'release', $kind);
            if ($info['blocked_on_release']) {
                $violations[] = $info['basename'].(count($info['reasons'])
                    ? ' ('.implode('; ', $info['reasons']).')'
                    : '');
            }
        }

        return $violations;
    }

    protected function looksLikeDebugBinary(string $lowerBasename, string $extension): bool
    {
        if (str_contains($lowerBasename, '.debug.')
            || str_ends_with($lowerBasename, '.debug.'.$extension)
            || str_ends_with($lowerBasename, '_debug.'.$extension)
            || str_ends_with($lowerBasename, '-debug.'.$extension)
            || str_contains($lowerBasename, 'prism.cnc.debug')
            || preg_match('/(^|[.\-_])debug([.\-_]|$)/', pathinfo($lowerBasename, PATHINFO_FILENAME)) === 1
        ) {
            return true;
        }

        // Common MSVC patterns: foo.pdb alongside foo.dll already caught by extension;
        // also catch Explicit *d.dll only when clearly tagged (avoid HelloWorld.dll).
        if ($extension === 'dll' && preg_match('/(?:^|[.\-_])(?:dbg|debug)(?:[.\-_]|$)/i', $lowerBasename) === 1) {
            return true;
        }

        return false;
    }
}
