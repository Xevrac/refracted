@extends('cdn.admin.layout')

@section('title', $package->displayName())

@section('content')
    <div class="flex flex-wrap items-start justify-between gap-4">
        <div>
            <h1 class="font-display text-3xl font-semibold tracking-[-0.02em]">{{ $package->displayName() }}</h1>
            <p class="mt-2 text-sm text-grit-mist">
                Status <strong>{{ $package->status }}</strong>
                @if ($package->is_latest) · latest for channel @endif
                @if ($package->published_at) · published {{ $package->published_at->diffForHumans() }} @endif
            </p>
        </div>
        <div class="flex flex-wrap gap-2">
            <a href="{{ route('cdn.admin.packages.edit', $package) }}" class="border border-grit-line px-3 py-2 text-sm hover:border-signal/40">Edit</a>
            @if ($package->status !== 'published')
                <form method="POST" action="{{ route('cdn.admin.packages.publish', $package) }}">
                    @csrf
                    <button class="bg-signal px-3 py-2 text-sm text-white">Publish</button>
                </form>
            @else
                <form method="POST" action="{{ route('cdn.admin.packages.unpublish', $package) }}">
                    @csrf
                    <button class="border border-grit-line px-3 py-2 text-sm">Unpublish</button>
                </form>
            @endif
            <form method="POST" action="{{ route('cdn.admin.packages.destroy', $package) }}" onsubmit="return confirm('Delete package?')">
                @csrf
                @method('DELETE')
                <button class="border border-red-500/40 px-3 py-2 text-sm text-red-200">Delete</button>
            </form>
        </div>
    </div>

    @if ($package->notes)
        <p class="ref-panel mt-6 p-4 text-sm text-grit-mist whitespace-pre-wrap">{{ $package->notes }}</p>
    @endif

    <h2 class="mt-10 font-display text-xl font-semibold">Artifacts</h2>
    <p class="mt-2 text-sm text-grit-mist">
        @if ($package->channel === 'release')
            Hint: The artifacts are the files you would upload to the release for users.
        @else
            Debug channel: DLLs preferred. PDBs need an explicit confirm. Linker leftovers (.exp/.lib/.map) are always refused.
        @endif
    </p>

    <form
        id="cdn-upload-form"
        method="POST"
        action="{{ route('cdn.admin.packages.artifacts.store', $package) }}"
        enctype="multipart/form-data"
        class="ref-panel mt-4 grid gap-3 p-4 sm:grid-cols-3"
        data-channel="{{ $package->channel }}"
        data-kind="{{ $package->kind }}"
    >
        @csrf
        <input type="hidden" name="confirm_risky" id="cdn-confirm-risky" value="0">
        <label class="text-sm sm:col-span-1">
            <span class="text-grit-mist">Folder (optional)</span>
            <input id="cdn-folder" name="folder" placeholder="game root" class="mt-1 w-full border border-grit-line bg-grit-surface px-3 py-2">
        </label>
        <label class="text-sm sm:col-span-1">
            <span class="text-grit-mist">Files</span>
            <input id="cdn-file" type="file" name="files[]" multiple class="mt-1 w-full text-sm" required>
        </label>
        <div class="flex items-end">
            <button type="submit" class="bg-signal px-4 py-2.5 text-xs font-semibold uppercase tracking-[0.12em] text-white">Upload</button>
        </div>
    </form>

    <dialog
        id="cdn-upload-guard"
        class="w-[min(100%,28rem)] border border-grit-line bg-grit-surface p-0 text-grit-text shadow-xl backdrop:bg-black/70"
    >
        <div class="px-5 py-5 sm:px-6">
            <h3 id="cdn-guard-title" class="font-display text-lg font-semibold tracking-[-0.02em]">Risky upload</h3>
            <p id="cdn-guard-body" class="mt-3 text-sm leading-relaxed text-grit-mist"></p>
            <ul id="cdn-guard-reasons" class="mt-3 list-disc space-y-1 ps-5 text-sm text-red-200"></ul>
            <div class="mt-6 flex flex-wrap justify-end gap-3">
                <button
                    type="button"
                    id="cdn-guard-cancel"
                    class="border border-grit-line px-4 py-2 text-sm text-grit-mist hover:border-signal hover:text-grit-text"
                >
                    Cancel
                </button>
                <button
                    type="button"
                    id="cdn-guard-continue"
                    class="bg-signal px-4 py-2 font-display text-sm font-semibold text-white hover:bg-signal-soft"
                    hidden
                >
                    Upload anyway
                </button>
            </div>
        </div>
    </dialog>

    <ul class="mt-4 space-y-2">
        @forelse ($package->artifacts as $artifact)
            <li class="ref-panel flex flex-wrap items-center justify-between gap-2 px-4 py-3 text-sm">
                <div>
                    <p class="font-medium">{{ $artifact->path }}</p>
                    <p class="mt-1 text-xs text-grit-mist">
                        {{ number_format($artifact->size_bytes) }} bytes · sha256 {{ \Illuminate\Support\Str::limit($artifact->sha256, 16) }}
                    </p>
                </div>
                <form method="POST" action="{{ route('cdn.admin.packages.artifacts.destroy', [$package, $artifact]) }}" onsubmit="return confirm('Remove artifact?')">
                    @csrf
                    @method('DELETE')
                    <button class="text-red-200 underline">Remove</button>
                </form>
            </li>
        @empty
            <li class="text-sm text-grit-mist">No files yet. Upload {{ $package->isRefracted() ? 'launcher build files' : 'Prism DLLs' }} here.</li>
        @endforelse
    </ul>

    <p class="mt-8 text-xs text-grit-mist">
        Launcher probe:
        <code class="text-grit-text">GET /{{ $package->urlPrefix() }}/manifest.json</code>
        with channel header <code class="text-grit-text">{{ $package->channel }}</code>
    </p>

    <script>
        (() => {
            const BLOCKED_RELEASE = new Set([
                'pdb','exp','lib','map','ilk','iobj','ipdb','idb','obj','o','a','bc','pch','ipch','tlog','lastbuildstate','log','recipe','sarif'
            ]);
            const ALWAYS_SUSPICIOUS = new Set(['exp','lib','map','ilk','iobj','ipdb','idb','obj','o','a','pch']);

            const form = document.getElementById('cdn-upload-form');
            const folderInput = document.getElementById('cdn-folder');
            const fileInput = document.getElementById('cdn-file');
            const confirmInput = document.getElementById('cdn-confirm-risky');
            const dialog = document.getElementById('cdn-upload-guard');
            const titleEl = document.getElementById('cdn-guard-title');
            const bodyEl = document.getElementById('cdn-guard-body');
            const reasonsEl = document.getElementById('cdn-guard-reasons');
            const cancelBtn = document.getElementById('cdn-guard-cancel');
            const continueBtn = document.getElementById('cdn-guard-continue');
            const channel = (form?.dataset.channel || 'release').toLowerCase();
            const dllOnly = channel === 'release' && form?.dataset.kind !== 'refracted';
            let forceSubmit = false;

            function basename(p) {
                const s = (p || '').replace(/\\/g, '/');
                const i = s.lastIndexOf('/');
                return i >= 0 ? s.slice(i + 1) : s;
            }

            function extOf(name) {
                const b = basename(name).toLowerCase();
                const i = b.lastIndexOf('.');
                return i >= 0 ? b.slice(i + 1) : '';
            }

            function looksDebug(name) {
                const b = basename(name).toLowerCase();
                const ext = extOf(b);
                const stem = ext ? b.slice(0, -(ext.length + 1)) : b;
                if (b.includes('.debug.') || b.endsWith('.debug.' + ext) || b.endsWith('_debug.' + ext) || b.endsWith('-debug.' + ext)) return true;
                if (b.includes('prism.cnc.debug')) return true;
                if (/(^|[.\-_])debug([.\-_]|$)/.test(stem)) return true;
                if (ext === 'dll' && /(?:^|[.\-_])(?:dbg|debug)(?:[.\-_]|$)/i.test(b)) return true;
                return false;
            }

            function inspect(path, fileName) {
                const names = [path, fileName].filter(Boolean);
                const reasons = [];
                let severity = 'ok';
                for (const name of names) {
                    const ext = extOf(name);
                    const base = basename(name);
                    const isDll = ext === 'dll';
                    const isPdb = ext === 'pdb';
                    const debug = looksDebug(name);

                    if (BLOCKED_RELEASE.has(ext)) reasons.push(`.${ext} is not a shippable runtime artifact (${base})`);
                    if (debug) reasons.push(`Name looks like a debug build (${base})`);
                    if (dllOnly && !isDll) reasons.push(`Release accepts DLL only — got .${ext || '(none)'} (${base})`);
                    if (channel === 'release' && isDll && debug) reasons.push(`Debug-named DLL blocked on release (${base})`);

                    if (channel === 'release' && ((dllOnly && !isDll) || debug || BLOCKED_RELEASE.has(ext))) severity = 'block';
                    else if (isPdb || debug || ALWAYS_SUSPICIOUS.has(ext) || (dllOnly && !isDll)) {
                        if (severity !== 'block') severity = 'warn';
                    }

                    if (channel === 'debug' && ALWAYS_SUSPICIOUS.has(ext) && !isPdb) severity = 'block';
                }
                return { severity, reasons: [...new Set(reasons)] };
            }

            cancelBtn?.addEventListener('click', () => dialog.close());
            continueBtn?.addEventListener('click', () => {
                forceSubmit = true;
                confirmInput.value = '1';
                dialog.close();
                form.requestSubmit();
            });

            form?.addEventListener('submit', (e) => {
                if (forceSubmit) {
                    forceSubmit = false;
                    return;
                }
                confirmInput.value = '0';
                const folder = folderInput.value.trim().replace(/\\/g, '/').replace(/^\/+|\/+$/g, '');
                const rank = { ok: 0, warn: 1, block: 2 };
                const result = { severity: 'ok', reasons: [] };
                for (const file of fileInput.files || []) {
                    const one = inspect(folder ? `${folder}/${file.name}` : file.name, '');
                    if (rank[one.severity] > rank[result.severity]) result.severity = one.severity;
                    result.reasons.push(...one.reasons);
                }
                result.reasons = [...new Set(result.reasons)];
                if (result.severity === 'ok') return;

                e.preventDefault();
                reasonsEl.innerHTML = '';
                result.reasons.forEach((r) => {
                    const li = document.createElement('li');
                    li.textContent = r;
                    reasonsEl.appendChild(li);
                });

                if (result.severity === 'block') {
                    titleEl.textContent = channel === 'release' ? 'Blocked on release' : 'Upload blocked';
                    bodyEl.textContent = channel === 'release'
                        ? 'Release packages block symbols, linker leftovers and debug-named binaries' + (dllOnly ? ', and accept DLLs only' : '') + '. Remove them, or create a debug-channel package instead.'
                        : 'This file type is never accepted on the CDN.';
                    continueBtn.hidden = true;
                } else {
                    titleEl.textContent = 'Confirm risky upload';
                    bodyEl.textContent = 'This looks like a debug symbol or non-runtime leftover. Upload only if you intend it for the debug channel.';
                    continueBtn.hidden = false;
                }
                dialog.showModal();
            });
        })();
    </script>
@endsection
