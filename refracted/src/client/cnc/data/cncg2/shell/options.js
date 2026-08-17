CCApp.controller('OptionsController', function($scope, $timeout) {
    $scope.optionsTab = 'GRAPHICS';
    var systemWidth = (window.screen && (window.screen.availWidth || window.screen.width)) || 1920;
    var systemHeight = (window.screen && (window.screen.availHeight || window.screen.height)) || 1080;
    var STANDARD_RESOLUTIONS = [
        '3840 X 2160',
        '2560 X 1440',
        '1920 X 1080',
        '1680 X 1050',
        '1600 X 900',
        '1440 X 900',
        '1366 X 768',
        '1360 X 768',
        '1280 X 1024',
        '1280 X 800',
        '1280 X 720',
        '1152 X 864',
        '1024 X 768'
    ];

    function asResolution(width, height) {
        return String(width) + ' X ' + String(height);
    }

    function parseResolution(value) {
        var match = String(value || '').match(/(\d+)\s*[xX]\s*(\d+)/);
        if (!match) {
            return null;
        }
        return {width: parseInt(match[1], 10), height: parseInt(match[2], 10)};
    }

    function fitsOnScreen(resolution) {
        var parsed = parseResolution(resolution);
        return !!parsed && parsed.width <= systemWidth && parsed.height <= systemHeight;
    }

    function dedupeResolutionList(values) {
        var deduped = [];
        var seen = {};
        for (var i = 0; i < values.length; i++) {
            var key = String(values[i] || '').trim();
            if (!key || seen[key]) {
                continue;
            }
            seen[key] = true;
            deduped.push(key);
        }
        return deduped;
    }

    function buildStandardResolutions(includeAboveNative) {
        var filtered = [];
        for (var i = 0; i < STANDARD_RESOLUTIONS.length; i++) {
            if (includeAboveNative || fitsOnScreen(STANDARD_RESOLUTIONS[i])) {
                filtered.push(STANDARD_RESOLUTIONS[i]);
            }
        }
        if (filtered.length === 0) {
            filtered = ['1920 X 1080', '1600 X 900', '1280 X 720', '1024 X 768'];
        }
        return dedupeResolutionList(filtered);
    }

    function nearestStandardResolution(width, height) {
        var candidates = buildStandardResolutions(false);
        var best = candidates[0];
        var bestScore = Number.MAX_SAFE_INTEGER;
        for (var i = 0; i < candidates.length; i++) {
            var parsed = parseResolution(candidates[i]);
            if (!parsed) {
                continue;
            }
            var score = Math.abs(parsed.width - width) + Math.abs(parsed.height - height);
            if (score < bestScore) {
                bestScore = score;
                best = candidates[i];
            }
        }
        return best || '1600 X 900';
    }

    var systemWindowedResolution = nearestStandardResolution(systemWidth, systemHeight);
    var systemWindowedParsed = parseResolution(systemWindowedResolution) || { width: 1600, height: 900 };
    var systemWindowedWidth = systemWindowedParsed.width;
    var systemWindowedHeight = systemWindowedParsed.height;

    function normalizeResolutionList(values) {
        if (!angular.isArray(values)) {
            return [];
        }
        var normalized = [];
        for (var i = 0; i < values.length; i++) {
            var entry = values[i];
            if (typeof entry === 'string') {
                normalized.push(entry);
            } else if (entry && typeof entry === 'object') {
                var w = parseInt(entry.width, 10);
                var h = parseInt(entry.height, 10);
                if (!isNaN(w) && !isNaN(h)) {
                    normalized.push(asResolution(w, h));
                }
            }
        }
        return dedupeResolutionList(normalized);
    }

    $scope.fullscreenResolutionOptions = dedupeResolutionList([asResolution(systemWidth, systemHeight)].concat(buildStandardResolutions(true)));
    $scope.windowedResolutionOptions = dedupeResolutionList(buildStandardResolutions(false).concat(['2560 X 1440', '3840 X 2160']));

    // Child knobs: DefaultValues.lua Quality (Low=0..Ultra=3). Alpha only has console
    // maps through High — Ultra is clamped in CncUserSettingsGraphicsBridge and omitted here.
    // OverallGraphicsQuality: OA indices 0=Autodetect..5=Custom. Ultra (4) cascades
    // UndergrowthQuality=Ultra → Lua assert; not offered in UI.
    var QUALITY = { Low: 0, Medium: 1, High: 2, Ultra: 3 };
    var OVERALL = { Autodetect: 0, Low: 1, Medium: 2, High: 3, Ultra: 4, Custom: 5 };
    var QUALITY_MAX = QUALITY.High;
    var GRAPHICS_PRESETS = {};
    GRAPHICS_PRESETS[OVERALL.Low] = {
        texturequality: QUALITY.Low, shadowquality: QUALITY.Low, effectsquality: QUALITY.Low,
        meshquality: QUALITY.Low, terrainquality: QUALITY.Low,
        antialiasingpost: 0, anisotropicfilter: 1, ambientocclusion: 0,
        motionblurenabled: false
    };
    GRAPHICS_PRESETS[OVERALL.Medium] = {
        texturequality: QUALITY.Medium, shadowquality: QUALITY.Medium, effectsquality: QUALITY.Medium,
        meshquality: QUALITY.Medium, terrainquality: QUALITY.Medium,
        antialiasingpost: 2, anisotropicfilter: 2, ambientocclusion: 1,
        motionblurenabled: false
    };
    GRAPHICS_PRESETS[OVERALL.High] = {
        texturequality: QUALITY.High, shadowquality: QUALITY.High, effectsquality: QUALITY.High,
        meshquality: QUALITY.High, terrainquality: QUALITY.High,
        antialiasingpost: 3, anisotropicfilter: 2, ambientocclusion: 2,
        motionblurenabled: true
    };

    var applyingOverallPreset = false;

    $scope.overallQualityOptions = [
        { value: OVERALL.Autodetect, label: 'Autodetect' },
        { value: OVERALL.Low, label: 'Low' },
        { value: OVERALL.Medium, label: 'Medium' },
        { value: OVERALL.High, label: 'High' },
        { value: OVERALL.Custom, label: 'Custom' }
    ];
    $scope.qualityOptions = [
        { value: QUALITY.Low, label: 'Low' },
        { value: QUALITY.Medium, label: 'Medium' },
        { value: QUALITY.High, label: 'High' }
    ];
    $scope.aaPostOptions = [
        { value: 0, label: 'Off' },
        { value: 1, label: 'Low (FXAA)' },
        { value: 2, label: 'Medium (FXAA)' },
        { value: 3, label: 'High (FXAA)' }
    ];
    $scope.aoOptions = [
        { value: 0, label: 'Off' },
        { value: 1, label: 'SSAO' },
        { value: 2, label: 'HBAO' },
        { value: 3, label: 'HBAO (Full)' }
    ];
    // DefaultValues.lua AnisotropicFilter enum: Off/X1/X2/X4 (OA indices 0..2).
    $scope.anisoOptions = [
        { value: 0, label: '1x' },
        { value: 1, label: '2x' },
        { value: 2, label: '4x' }
    ];

    $scope.settings = {
        shellfullscreen: false,
        gamefullscreen: false,
        fullscreenwidth: systemWidth,
        fullscreenheight: systemHeight,
        windowedwidth: systemWindowedWidth,
        windowedheight: systemWindowedHeight,
        mastervolume: 30,
        edgepan: true,
        edgescrollspeed: 40,
        middlemousecameradrag: false,
        allowcamerarotation: false,
        movemodeattack: false,
        allowdeselect: true,
        fullscreenResolution: asResolution(systemWidth, systemHeight),
        windowedResolution: systemWindowedResolution,
        overallgraphicsquality: OVERALL.Autodetect,
        texturequality: QUALITY.Medium,
        shadowquality: QUALITY.Medium,
        effectsquality: QUALITY.Medium,
        meshquality: QUALITY.Medium,
        terrainquality: QUALITY.Medium,
        antialiasingpost: 2,
        ambientocclusion: 1,
        anisotropicfilter: 2,
        vsyncenabled: false,
        motionblurenabled: false,
        // Native PostProcess.UIBrightnessNorm default is full (1.0). Int bridge used to
        // truncate mid values to 0 — factory UI default matches retail full brightness.
        brightness: 1.0,
        brightnessPercent: 100,
        shellUiTheme: (window.CncShellTheme && CncShellTheme.get) ? CncShellTheme.get() : 'aurora',
        shellUiThemeDefault: (window.CncShellTheme && CncShellTheme.getDefault)
            ? CncShellTheme.getDefault()
            : ((window.CncShellTheme && CncShellTheme.get) ? CncShellTheme.get() : 'aurora')
    };

    $scope.defaultSettings = angular.copy($scope.settings);
    $scope.cameraRotationLocked = false;
    $scope.shellThemeOptions = (window.CncShellTheme && CncShellTheme.list) ? CncShellTheme.list() : [
        { id: 'classic', label: 'Classic' },
        { id: 'aurora', label: 'Aurora' }
    ];
    $scope.allowShellThemeSelect = !!$scope.$root.allowShellThemeSelect;
    // Placeholder defaults must never be applied/saved until /usersettings hydrates.
    $scope.settingsReady = false;
    $scope.settingsLoadError = false;
    $scope.settingsSaving = false;

    var themeSnapshot = null;
    var themeSaveCommitted = false;
    var settingsLoadSeq = 0;
    var controlsResynced = false;

    function refreshShellThemeHint() {
        var id = $scope.settings.shellUiThemeDefault || $scope.settings.shellUiTheme;
        if (window.CncShellTheme && CncShellTheme.hint) {
            $scope.shellThemeHint = CncShellTheme.hint(id);
        } else {
            $scope.shellThemeHint = '';
        }
    }

    function readCommittedTheme() {
        return {
            theme: (window.CncShellTheme && CncShellTheme.get) ? CncShellTheme.get() : 'aurora',
            defaultTheme: (window.CncShellTheme && CncShellTheme.getDefault)
                ? CncShellTheme.getDefault()
                : ((window.CncShellTheme && CncShellTheme.get) ? CncShellTheme.get() : 'aurora')
        };
    }

    function captureThemeSnapshot() {
        themeSnapshot = readCommittedTheme();
        $scope.settings.shellUiTheme = themeSnapshot.theme;
        $scope.settings.shellUiThemeDefault = themeSnapshot.defaultTheme;
        themeSaveCommitted = false;
        refreshShellThemeHint();
    }

    function revertThemeDraft() {
        if (!themeSnapshot) {
            themeSnapshot = readCommittedTheme();
        }
        $scope.settings.shellUiTheme = themeSnapshot.theme;
        $scope.settings.shellUiThemeDefault = themeSnapshot.defaultTheme;
        if (window.CncShellTheme && CncShellTheme.get &&
                CncShellTheme.get() !== themeSnapshot.theme) {
            if (CncShellTheme.restore) {
                CncShellTheme.restore(themeSnapshot.theme, themeSnapshot.defaultTheme);
            } else if (CncShellTheme.apply) {
                CncShellTheme.apply(themeSnapshot.theme);
            }
        }
        refreshShellThemeHint();
    }

    refreshShellThemeHint();

    // Shell access layer returns JSON strings, not parsed objects — normalize once here.
    function coerceShellResponse(res) {
        if (typeof res === 'string') {
            var trimmed = res.trim();
            if (!trimmed) {
                return {};
            }
            try {
                res = JSON.parse(trimmed);
            } catch (e) {
                return {};
            }
        }
        if (!res || typeof res !== 'object') {
            return {};
        }
        return res;
    }

    function asNumber(value) {
        if (typeof value === 'number' && isFinite(value)) {
            return value;
        }
        if (typeof value === 'string' && value !== '') {
            var n = Number(value);
            if (isFinite(n)) {
                return n;
            }
        }
        return null;
    }

    function sortResolutionsDescending(list) {
        return list.slice().sort(function(a, b) {
            var pa = parseResolution(a);
            var pb = parseResolution(b);
            if (!pa || !pb) {
                return 0;
            }
            return (pb.width * pb.height) - (pa.width * pa.height);
        });
    }

    function applyResolutionOptionsFromEngine(res) {
        if (!res || typeof res !== 'object') {
            return;
        }
        var available = normalizeResolutionList(res.availableResolutions);
        var fullscreen = normalizeResolutionList(res.fullscreenResolutions);
        var windowed = normalizeResolutionList(res.windowedResolutions);

        if (fullscreen.length === 0 && available.length > 0) {
            fullscreen = sortResolutionsDescending(
                dedupeResolutionList([asResolution(systemWidth, systemHeight)].concat(available))
            );
        }
        if (windowed.length === 0 && available.length > 0) {
            windowed = sortResolutionsDescending(available.filter(fitsOnScreen));
            if (windowed.length === 0) {
                windowed = sortResolutionsDescending(available.slice());
            }
        }
        if (windowed.length > 0) {
            windowed = windowed.filter(function(r) {
                return STANDARD_RESOLUTIONS.indexOf(r) !== -1 || available.indexOf(r) !== -1;
            });
        }

        if (fullscreen.length > 0) {
            $scope.fullscreenResolutionOptions = fullscreen;
        }
        if (windowed.length > 0) {
            $scope.windowedResolutionOptions = windowed;
        } else if (fullscreen.length === 0) {
            $scope.windowedResolutionOptions = dedupeResolutionList(
                buildStandardResolutions(false).concat(['2560 X 1440', '3840 X 2160'])
            );
        }
        syncResolutionModels();
        if ($scope.fullscreenResolutionOptions.indexOf($scope.settings.fullscreenResolution) === -1) {
            $scope.fullscreenResolutionOptions.unshift($scope.settings.fullscreenResolution);
        }
        if ($scope.windowedResolutionOptions.indexOf($scope.settings.windowedResolution) === -1) {
            $scope.windowedResolutionOptions.unshift($scope.settings.windowedResolution);
        }
        $scope.$applyAsync();
    }

    function executeShell(resource, extra, onResponse) {
        if (!window.shellaccesslayer || typeof window.shellaccesslayer.execute !== 'function') {
            if (typeof onResponse === 'function') {
                onResponse(null);
            }
            return;
        }
        var req = {_resource: resource};
        if (extra) {
            for (var key in extra) {
                if (Object.prototype.hasOwnProperty.call(extra, key)) {
                    req[key] = extra[key];
                }
            }
        }
        if (typeof onResponse === 'function') {
            req._response = function(res) {
                var coerced = coerceShellResponse(res);
                $timeout(function() {
                    onResponse(coerced);
                }, 0);
            };
        }
        window.shellaccesslayer.execute(req);
    }

    function syncResolutionModels() {
        $scope.settings.fullscreenResolution = asResolution($scope.settings.fullscreenwidth, $scope.settings.fullscreenheight);
        $scope.settings.windowedResolution = asResolution($scope.settings.windowedwidth, $scope.settings.windowedheight);
    }

    function applyPartial(partial) {
        // Never push placeholder defaults into the engine before hydration.
        if (!$scope.settingsReady || !partial) {
            return;
        }
        executeShell('/usersettings/apply', partial);
    }

    function clampInt(value, min, max, fallback) {
        var n = parseInt(value, 10);
        if (isNaN(n)) {
            return fallback;
        }
        return Math.max(min, Math.min(max, n));
    }

    // Map legacy / engine Ultra onto the highest level the alpha UI exposes.
    function sanitizeOverallForUi(value) {
        var o = clampInt(value, 0, 5, OVERALL.Custom);
        return o === OVERALL.Ultra ? OVERALL.High : o;
    }

    function sanitizeChildQuality(value) {
        return clampInt(value, 0, QUALITY_MAX, QUALITY.Medium);
    }

    function sanitizeAnisoForUi(value) {
        return clampInt(value, 0, 2, 2);
    }

    function syncBrightnessPercent() {
        var b = Number($scope.settings.brightness);
        if (!isFinite(b)) {
            b = 1.0;
        }
        if (b > 1) {
            b = b / 100;
        }
        $scope.settings.brightness = Math.max(0.01, Math.min(1, b));
        $scope.settings.brightnessPercent = Math.max(1, Math.round($scope.settings.brightness * 100));
    }

    // In-game CEF often won't flush $applyAsync until input; force a digest.
    function digestSettingsUi() {
        $timeout(function() {}, 0);
    }

    function applyPresetToSettings(presetId) {
        var preset = GRAPHICS_PRESETS[presetId];
        if (!preset) {
            return;
        }
        applyingOverallPreset = true;
        angular.extend($scope.settings, preset);
        applyingOverallPreset = false;
    }

    function buildGraphicsPayload() {
        return {
            overallgraphicsquality: sanitizeOverallForUi($scope.settings.overallgraphicsquality),
            texturequality: sanitizeChildQuality($scope.settings.texturequality),
            shadowquality: sanitizeChildQuality($scope.settings.shadowquality),
            effectsquality: sanitizeChildQuality($scope.settings.effectsquality),
            meshquality: sanitizeChildQuality($scope.settings.meshquality),
            terrainquality: sanitizeChildQuality($scope.settings.terrainquality),
            antialiasingpost: clampInt($scope.settings.antialiasingpost, 0, 3, 0),
            ambientocclusion: clampInt($scope.settings.ambientocclusion, 0, 3, 0),
            anisotropicfilter: sanitizeAnisoForUi($scope.settings.anisotropicfilter),
            // Engine Settings store these as 0/1 (DefaultValues / Graphics.lua).
            vsyncenabled: $scope.settings.vsyncenabled ? 1 : 0,
            motionblurenabled: $scope.settings.motionblurenabled ? 1 : 0,
            brightness: Math.max(0.01, Math.min(1, Number($scope.settings.brightness) || 1.0))
        };
    }

    function syncCameraRotationLock() {
        $scope.cameraRotationLocked = !!$scope.settings.middlemousecameradrag;
        if ($scope.cameraRotationLocked) {
            $scope.settings.allowcamerarotation = false;
        }
    }

    function buildControlsApplyPayload() {
        syncCameraRotationLock();
        return {
            edgepan: $scope.settings.edgepan ? 1 : 0,
            edgescrollspeed: Math.max(0, Math.min(100, Math.round($scope.settings.edgescrollspeed))),
            // 0/1 — some Shell JSON paths drop boolean false; RMB camera is false.
            middlemousecameradrag: $scope.settings.middlemousecameradrag ? 1 : 0,
            allowcamerarotation: $scope.settings.allowcamerarotation ? 1 : 0
        };
    }

    function buildApplyPayloadFromSettings() {
        var fullParsed = parseResolution($scope.settings.fullscreenResolution);
        var windowParsed = parseResolution($scope.settings.windowedResolution);
        if (fullParsed) {
            $scope.settings.fullscreenwidth = fullParsed.width;
            $scope.settings.fullscreenheight = fullParsed.height;
        }
        if (windowParsed) {
            $scope.settings.windowedwidth = windowParsed.width;
            $scope.settings.windowedheight = windowParsed.height;
        }
        syncCameraRotationLock();
        var payload = {
            shellfullscreen: $scope.settings.shellfullscreen ? 1 : 0,
            gamefullscreen: $scope.settings.gamefullscreen ? 1 : 0,
            fullscreenwidth: $scope.settings.fullscreenwidth,
            fullscreenheight: $scope.settings.fullscreenheight,
            windowedwidth: $scope.settings.windowedwidth,
            windowedheight: $scope.settings.windowedheight,
            mastervolume: Math.max(0, Math.min(100, Math.round($scope.settings.mastervolume))) / 10,
            edgepan: $scope.settings.edgepan ? 1 : 0,
            edgescrollspeed: Math.max(0, Math.min(100, Math.round($scope.settings.edgescrollspeed))),
            middlemousecameradrag: $scope.settings.middlemousecameradrag ? 1 : 0,
            allowcamerarotation: $scope.settings.allowcamerarotation ? 1 : 0,
            movemodeattack: $scope.settings.movemodeattack ? 1 : 0,
            allowdeselect: $scope.settings.allowdeselect ? 1 : 0
        };
        angular.extend(payload, buildGraphicsPayload());
        return payload;
    }


    function readGraphicsFromResponse(res) {
        if (!res || typeof res !== 'object') {
            return;
        }
        var keys = [
            'overallgraphicsquality', 'texturequality', 'shadowquality', 'effectsquality',
            'meshquality', 'terrainquality', 'antialiasingpost', 'ambientocclusion',
            'anisotropicfilter'
        ];
        for (var i = 0; i < keys.length; i++) {
            var k = keys[i];
            if (typeof res[k] === 'number') {
                if (k === 'overallgraphicsquality') {
                    // Legacy Quality.Autodetect (-1) → OA index 0; Ultra → High (not offered).
                    $scope.settings[k] = res[k] < 0 ? OVERALL.Autodetect : sanitizeOverallForUi(res[k]);
                } else if (k === 'texturequality' || k === 'shadowquality' || k === 'effectsquality'
                        || k === 'meshquality' || k === 'terrainquality') {
                    $scope.settings[k] = sanitizeChildQuality(res[k]);
                } else if (k === 'anisotropicfilter') {
                    $scope.settings[k] = sanitizeAnisoForUi(res[k]);
                } else {
                    $scope.settings[k] = res[k];
                }
            }
        }
        if (typeof res.vsyncenabled === 'boolean') {
            $scope.settings.vsyncenabled = res.vsyncenabled;
        } else if (typeof res.vsyncenabled === 'number') {
            $scope.settings.vsyncenabled = res.vsyncenabled !== 0;
        }
        if (typeof res.motionblurenabled === 'boolean') {
            $scope.settings.motionblurenabled = res.motionblurenabled;
        } else if (typeof res.motionblurenabled === 'number') {
            $scope.settings.motionblurenabled = res.motionblurenabled !== 0;
        }
        if (typeof res.brightness === 'number') {
            $scope.settings.brightness = res.brightness;
        }
        syncBrightnessPercent();
    }

    function asBool(value, fallback) {
        if (typeof value === 'boolean') {
            return value;
        }
        if (typeof value === 'number') {
            return value !== 0;
        }
        if (typeof value === 'string') {
            var s = value.toLowerCase();
            if (s === 'true' || s === '1') {
                return true;
            }
            if (s === 'false' || s === '0') {
                return false;
            }
        }
        return fallback;
    }

    function responseLooksHydrated(res) {
        if (!res || typeof res !== 'object') {
            return false;
        }
        // Retail may emit fullscreen flags as 0/1/strings and omit nils from JSON.
        if (res.shellfullscreen != null || res.gamefullscreen != null
                || asNumber(res.mastervolume) != null
                || asNumber(res.fullscreenwidth) != null
                || asNumber(res.windowedwidth) != null
                || asNumber(res.overallgraphicsquality) != null
                || asNumber(res.texturequality) != null
                || asNumber(res.shadowquality) != null) {
            return true;
        }
        // status:0 alone is not enough (empty patch / failed read), but status + keys is.
        var statusNum = asNumber(res.status);
        if (statusNum === 0) {
            for (var key in res) {
                if (Object.prototype.hasOwnProperty.call(res, key) && key !== 'status' && res[key] != null) {
                    return true;
                }
            }
        }
        return false;
    }

    function applyEngineSettingsResponse(res) {
        if (!res || typeof res !== 'object') {
            return false;
        }
        if (res.shellfullscreen != null) {
            $scope.settings.shellfullscreen = asBool(res.shellfullscreen, $scope.settings.shellfullscreen);
        }
        if (res.gamefullscreen != null) {
            $scope.settings.gamefullscreen = asBool(res.gamefullscreen, $scope.settings.gamefullscreen);
        }
        var fw = asNumber(res.fullscreenwidth);
        var fh = asNumber(res.fullscreenheight);
        var ww = asNumber(res.windowedwidth);
        var wh = asNumber(res.windowedheight);
        if (fw != null) { $scope.settings.fullscreenwidth = fw; }
        if (fh != null) { $scope.settings.fullscreenheight = fh; }
        if (ww != null) { $scope.settings.windowedwidth = ww; }
        if (wh != null) { $scope.settings.windowedheight = wh; }
        var mv = asNumber(res.mastervolume);
        if (mv != null) {
            var asPercent = mv <= 10 ? (mv * 10) : mv;
            $scope.settings.mastervolume = Math.max(0, Math.min(100, Math.round(asPercent)));
        }
        if (res.edgepan != null) { $scope.settings.edgepan = asBool(res.edgepan, $scope.settings.edgepan); }
        var ess = asNumber(res.edgescrollspeed);
        if (ess != null) { $scope.settings.edgescrollspeed = Math.max(0, Math.min(100, Math.round(ess))); }
        if (res.middlemousecameradrag != null) {
            $scope.settings.middlemousecameradrag = asBool(res.middlemousecameradrag, $scope.settings.middlemousecameradrag);
        }
        if (res.allowcamerarotation != null) {
            $scope.settings.allowcamerarotation = asBool(res.allowcamerarotation, $scope.settings.allowcamerarotation);
        }
        syncCameraRotationLock();
        if (res.movemodeattack != null) {
            $scope.settings.movemodeattack = asBool(res.movemodeattack, $scope.settings.movemodeattack);
        }
        if (res.allowdeselect != null) {
            $scope.settings.allowdeselect = asBool(res.allowdeselect, $scope.settings.allowdeselect);
        }
        // Coerce stringy graphics ints before readGraphicsFromResponse.
        var gKeys = [
            'overallgraphicsquality', 'texturequality', 'shadowquality', 'effectsquality',
            'meshquality', 'terrainquality', 'antialiasingpost', 'ambientocclusion',
            'anisotropicfilter', 'vsyncenabled', 'motionblurenabled', 'brightness'
        ];
        for (var gi = 0; gi < gKeys.length; gi++) {
            var gk = gKeys[gi];
            if (typeof res[gk] === 'string') {
                var gn = asNumber(res[gk]);
                if (gn != null) {
                    res[gk] = gn;
                }
            }
        }
        readGraphicsFromResponse(res);
        syncResolutionModels();
        return responseLooksHydrated(res);
    }

    function loadUserSettings(done) {
        var seq = ++settingsLoadSeq;
        var attempts = 0;
        var maxAttempts = 8;

        function attempt() {
            attempts += 1;
            executeShell('/usersettings', null, function(res) {
                if (seq !== settingsLoadSeq) {
                    return;
                }
                var ok = applyEngineSettingsResponse(res);
                if (!ok && attempts < maxAttempts) {
                    // Shell Lua / profile can lag first open; retry before hard-fail UI.
                    setTimeout(attempt, 300 * attempts);
                    return;
                }
                $scope.settingsReady = ok;
                $scope.settingsLoadError = !ok;
                // Last attempt: accept partial hydrate so in-game is never stuck forever.
                if (!ok && attempts >= maxAttempts) {
                    $scope.settingsReady = true;
                    $scope.settingsLoadError = false;
                }
                digestSettingsUi();
                // One resync after first hydrate — avoids applyProfileSettings spam on retries.
                if ($scope.settingsReady && !controlsResynced) {
                    controlsResynced = true;
                    executeShell('/usersettings/apply', buildControlsApplyPayload());
                }
                if (typeof done === 'function') {
                    done($scope.settingsReady);
                }
            });
        }

        attempt();
    }

    function loadDisplayConfig() {
        executeShell('/config/display', null, applyResolutionOptionsFromEngine);
    }

    function loadGraphicsOptions() {
        executeShell('/options/graphics/get', null, applyResolutionOptionsFromEngine);
    }

    $scope.setOptionsTab = function(tabName) {
        $scope.optionsTab = tabName;
    };

    $scope.applyGraphicsMode = function() {
        if (!$scope.settingsReady) {
            return;
        }
        applyPartial({
            shellfullscreen: $scope.settings.shellfullscreen ? 1 : 0,
            gamefullscreen: $scope.settings.gamefullscreen ? 1 : 0
        });
    };

    $scope.applyFullscreenResolution = function() {
        if (!$scope.settingsReady) {
            return;
        }
        var parsed = parseResolution($scope.settings.fullscreenResolution);
        if (!parsed) {
            return;
        }
        $scope.settings.fullscreenwidth = parsed.width;
        $scope.settings.fullscreenheight = parsed.height;
        applyPartial({
            fullscreenwidth: parsed.width,
            fullscreenheight: parsed.height
        });
    };

    $scope.applyWindowedResolution = function() {
        if (!$scope.settingsReady) {
            return;
        }
        var parsed = parseResolution($scope.settings.windowedResolution);
        if (!parsed) {
            return;
        }
        $scope.settings.windowedwidth = parsed.width;
        $scope.settings.windowedheight = parsed.height;
        applyPartial({
            windowedwidth: parsed.width,
            windowedheight: parsed.height
        });
    };

    $scope.onOverallGraphicsChange = function() {
        if (!$scope.settingsReady) {
            return;
        }
        var overall = sanitizeOverallForUi($scope.settings.overallgraphicsquality);
        $scope.settings.overallgraphicsquality = overall;
        if (overall === OVERALL.Custom || overall === OVERALL.Autodetect) {
            applyPartial(buildGraphicsPayload());
            return;
        }
        applyPresetToSettings(overall);
        applyPartial(buildGraphicsPayload());
    };

    $scope.onGraphicsDetailChange = function() {
        if (!$scope.settingsReady) {
            return;
        }
        if (!applyingOverallPreset && $scope.settings.overallgraphicsquality !== OVERALL.Custom) {
            $scope.settings.overallgraphicsquality = OVERALL.Custom;
        }
        applyPartial(buildGraphicsPayload());
    };

    var brightnessApplyTimer = null;

    $scope.onBrightnessChange = function() {
        if (!$scope.settingsReady) {
            return;
        }
        var pct = clampInt($scope.settings.brightnessPercent, 1, 100, 100);
        $scope.settings.brightnessPercent = pct;
        $scope.settings.brightness = pct / 100;
        if (!applyingOverallPreset && $scope.settings.overallgraphicsquality !== OVERALL.Custom) {
            $scope.settings.overallgraphicsquality = OVERALL.Custom;
        }
        // Debounce + brightness-only payload. Full graphics apply every tick was the lag.
        if (brightnessApplyTimer) {
            $timeout.cancel(brightnessApplyTimer);
        }
        brightnessApplyTimer = $timeout(function () {
            brightnessApplyTimer = null;
            applyPartial({
                brightness: $scope.settings.brightness,
                overallgraphicsquality: $scope.settings.overallgraphicsquality
            });
        }, 180);
    };

    $scope.applyVolume = function() {
        if (!$scope.settingsReady) {
            return;
        }
        var volume = Math.max(0, Math.min(100, Math.round($scope.settings.mastervolume)));
        $scope.settings.mastervolume = volume;
        executeShell('/usersettings/apply', {mastervolume: volume / 10}, function () {
            executeShell('/usersettings/save');
        });
    };

    $scope.applyControls = function() {
        if (!$scope.settingsReady) {
            return;
        }
        executeShell('/usersettings/apply', buildControlsApplyPayload(), function () {
            // Persist to ProgramData\CNCO_DL\...\RtsProfileSettings.cfg
            executeShell('/usersettings/save');
        });
    };

    $scope.applyGameplay = function() {
        if (!$scope.settingsReady) {
            return;
        }
        executeShell('/usersettings/apply', {
            movemodeattack: $scope.settings.movemodeattack ? 1 : 0,
            allowdeselect: $scope.settings.allowdeselect ? 1 : 0
        }, function () {
            executeShell('/usersettings/save');
        });
    };

    $scope.onDefaultThemeChange = function() {
        if (!$scope.allowShellThemeSelect) {
            return;
        }
        var id = $scope.settings.shellUiThemeDefault || 'aurora';
        $scope.settings.shellUiTheme = id;
        // Preview only until OK — persist both active + default on save.
        if (window.CncShellTheme && CncShellTheme.apply) {
            CncShellTheme.apply(id);
        } else if (window.CncShellTheme && CncShellTheme.set) {
            CncShellTheme.set(id);
        }
        refreshShellThemeHint();
    };

    $scope.restoreDefaults = function() {
        if (!$scope.settingsReady) {
            return;
        }
        $scope.settings = angular.copy($scope.defaultSettings);
        if ($scope.allowShellThemeSelect && $scope.settings.shellUiThemeDefault) {
            $scope.settings.shellUiTheme = $scope.settings.shellUiThemeDefault;
        }
        syncBrightnessPercent();
        syncResolutionModels();
        applyPartial(buildApplyPayloadFromSettings());
        executeShell('/usersettings/applyAudio');
        if ($scope.allowShellThemeSelect) {
            refreshShellThemeHint();
        }
    };

    $scope.actionSave = function() {
        if (!$scope.settingsReady || $scope.settingsSaving) {
            return;
        }
        $scope.settingsSaving = true;
        if (brightnessApplyTimer) {
            $timeout.cancel(brightnessApplyTimer);
            brightnessApplyTimer = null;
        }
        var payload = buildApplyPayloadFromSettings();
        var pendingTheme = null;
        var pendingDefaultTheme = null;
        if ($scope.allowShellThemeSelect) {
            pendingDefaultTheme = $scope.settings.shellUiThemeDefault || 'aurora';
            pendingTheme = pendingDefaultTheme;
            themeSaveCommitted = true;
            themeSnapshot = {
                theme: pendingTheme,
                defaultTheme: pendingDefaultTheme
            };
            $scope.settings.shellUiTheme = pendingTheme;
            $scope.settings.shellUiThemeDefault = pendingDefaultTheme;
        }
        var saveWatchdog = $timeout(function() {
            if ($scope.settingsSaving) {
                finishSave();
            }
        }, 3000);
        function finishSave() {
            if (saveWatchdog) {
                $timeout.cancel(saveWatchdog);
                saveWatchdog = null;
            }
            $scope.settingsSaving = false;
            $scope.closeOptions();
        }
        // Apply + save persist. Do not wait on /usersettings/discard — retail discard
        // often omits _response in-game, which left OK stuck disabled.
        executeShell('/usersettings/apply', payload, function () {
            executeShell('/usersettings/save', null, function () {
                executeShell('/usersettings/discard');
                finishSave();
            });
        });
        executeShell('/usersettings/applyAudio');
        if ($scope.allowShellThemeSelect && window.CncShellTheme) {
            if (pendingTheme && CncShellTheme.set) {
                CncShellTheme.set(pendingTheme);
            }
            if (pendingDefaultTheme && CncShellTheme.setDefault) {
                CncShellTheme.setDefault(pendingDefaultTheme);
            }
            refreshShellThemeHint();
        }
    };

    $scope.actionCancel = function() {
        executeShell('/usersettings/discard');
        $scope.settingsReady = false;
        $scope.settingsLoadError = false;
        loadUserSettings();
        if ($scope.allowShellThemeSelect) {
            revertThemeDraft();
            themeSaveCommitted = false;
        }
        $scope.closeOptions();
    };

    $scope.$watch('optionsOpen', function(isOpen) {
        if (isOpen) {
            $scope.settingsReady = false;
            $scope.settingsLoadError = false;
            $scope.settingsSaving = false;
            controlsResynced = false;
            loadUserSettings();
            loadDisplayConfig();
            loadGraphicsOptions();
            if ($scope.allowShellThemeSelect) {
                captureThemeSnapshot();
            }
        } else if ($scope.allowShellThemeSelect && !themeSaveCommitted) {
            revertThemeDraft();
        }
    });

    syncResolutionModels();
    loadUserSettings();
    loadDisplayConfig();
    loadGraphicsOptions();
});
