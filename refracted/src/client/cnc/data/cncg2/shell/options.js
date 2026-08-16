CCApp.controller('OptionsController', function($scope) {
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

    // Child knobs use DefaultValues.lua Quality enum (Low=0..Ultra=3).
    // OverallGraphicsQuality uses OA cascade indices (0=Autodetect..5=Custom) — see LuaOptionSetManager.
    var QUALITY = { Low: 0, Medium: 1, High: 2, Ultra: 3 };
    var OVERALL = { Autodetect: 0, Low: 1, Medium: 2, High: 3, Ultra: 4, Custom: 5 };
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
        antialiasingpost: 3, anisotropicfilter: 4, ambientocclusion: 2,
        motionblurenabled: true
    };
    GRAPHICS_PRESETS[OVERALL.Ultra] = {
        texturequality: QUALITY.Ultra, shadowquality: QUALITY.Ultra, effectsquality: QUALITY.Ultra,
        meshquality: QUALITY.Ultra, terrainquality: QUALITY.Ultra,
        antialiasingpost: 3, anisotropicfilter: 4, ambientocclusion: 2,
        motionblurenabled: true
    };

    var applyingOverallPreset = false;

    $scope.overallQualityOptions = [
        { value: OVERALL.Autodetect, label: 'Autodetect' },
        { value: OVERALL.Low, label: 'Low' },
        { value: OVERALL.Medium, label: 'Medium' },
        { value: OVERALL.High, label: 'High' },
        { value: OVERALL.Ultra, label: 'Ultra' },
        { value: OVERALL.Custom, label: 'Custom' }
    ];
    $scope.qualityOptions = [
        { value: QUALITY.Low, label: 'Low' },
        { value: QUALITY.Medium, label: 'Medium' },
        { value: QUALITY.High, label: 'High' },
        { value: QUALITY.Ultra, label: 'Ultra' }
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
        { value: 3, label: 'HBAO Full' }
    ];
    $scope.anisoOptions = [
        { value: 0, label: '1x' },
        { value: 1, label: '2x' },
        { value: 2, label: '4x' },
        { value: 3, label: '8x' },
        { value: 4, label: '16x' }
    ];

    $scope.settings = {
        shellfullscreen: true,
        gamefullscreen: false,
        fullscreenwidth: systemWidth,
        fullscreenheight: systemHeight,
        windowedwidth: systemWindowedWidth,
        windowedheight: systemWindowedHeight,
        mastervolume: 30,
        edgepan: true,
        edgescrollspeed: 40,
        middlemousecameradrag: false,
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
        brightness: 0.5,
        brightnessPercent: 50,
        shellUiTheme: (window.CncShellTheme && CncShellTheme.get) ? CncShellTheme.get() : 'aurora',
        shellUiThemeDefault: (window.CncShellTheme && CncShellTheme.getDefault)
            ? CncShellTheme.getDefault()
            : ((window.CncShellTheme && CncShellTheme.get) ? CncShellTheme.get() : 'aurora')
    };

    $scope.defaultSettings = angular.copy($scope.settings);
    $scope.shellThemeOptions = (window.CncShellTheme && CncShellTheme.list) ? CncShellTheme.list() : [
        { id: 'classic', label: 'Classic' },
        { id: 'aurora', label: 'Aurora' }
    ];
    $scope.allowShellThemeSelect = !!$scope.$root.allowShellThemeSelect;

    var themeSnapshot = null;
    var themeSaveCommitted = false;

    function refreshShellThemeHint() {
        if (window.CncShellTheme && CncShellTheme.hint) {
            $scope.shellThemeHint = CncShellTheme.hint($scope.settings.shellUiTheme);
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

    function executeShell(resource, extra, onResponse) {
        if (!window.shellaccesslayer || typeof window.shellaccesslayer.execute !== 'function') {
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
                onResponse(res || {});
            };
        }
        window.shellaccesslayer.execute(req);
    }

    function syncResolutionModels() {
        $scope.settings.fullscreenResolution = asResolution($scope.settings.fullscreenwidth, $scope.settings.fullscreenheight);
        $scope.settings.windowedResolution = asResolution($scope.settings.windowedwidth, $scope.settings.windowedheight);
    }

    function applyPartial(partial) {
        executeShell('/usersettings/apply', partial);
    }

    function clampInt(value, min, max, fallback) {
        var n = parseInt(value, 10);
        if (isNaN(n)) {
            return fallback;
        }
        return Math.max(min, Math.min(max, n));
    }

    function syncBrightnessPercent() {
        var b = Number($scope.settings.brightness);
        if (!isFinite(b)) {
            b = 0.5;
        }
        if (b > 1) {
            b = b / 100;
        }
        $scope.settings.brightness = Math.max(0, Math.min(1, b));
        $scope.settings.brightnessPercent = Math.round($scope.settings.brightness * 100);
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
            overallgraphicsquality: clampInt($scope.settings.overallgraphicsquality, 0, 5, OVERALL.Custom),
            texturequality: clampInt($scope.settings.texturequality, 0, 3, QUALITY.Medium),
            shadowquality: clampInt($scope.settings.shadowquality, 0, 3, QUALITY.Medium),
            effectsquality: clampInt($scope.settings.effectsquality, 0, 3, QUALITY.Medium),
            meshquality: clampInt($scope.settings.meshquality, 0, 3, QUALITY.Medium),
            terrainquality: clampInt($scope.settings.terrainquality, 0, 3, QUALITY.Medium),
            antialiasingpost: clampInt($scope.settings.antialiasingpost, 0, 3, 0),
            ambientocclusion: clampInt($scope.settings.ambientocclusion, 0, 3, 0),
            anisotropicfilter: clampInt($scope.settings.anisotropicfilter, 0, 4, 1),
            // Engine Settings store these as 0/1 (DefaultValues / Graphics.lua).
            vsyncenabled: $scope.settings.vsyncenabled ? 1 : 0,
            motionblurenabled: $scope.settings.motionblurenabled ? 1 : 0,
            brightness: Math.max(0, Math.min(1, Number($scope.settings.brightness) || 0.5))
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
        var payload = {
            shellfullscreen: !!$scope.settings.shellfullscreen,
            gamefullscreen: !!$scope.settings.gamefullscreen,
            fullscreenwidth: $scope.settings.fullscreenwidth,
            fullscreenheight: $scope.settings.fullscreenheight,
            windowedwidth: $scope.settings.windowedwidth,
            windowedheight: $scope.settings.windowedheight,
            mastervolume: Math.max(0, Math.min(100, Math.round($scope.settings.mastervolume))) / 10,
            edgepan: !!$scope.settings.edgepan,
            edgescrollspeed: Math.max(0, Math.min(100, Math.round($scope.settings.edgescrollspeed))),
            middlemousecameradrag: !!$scope.settings.middlemousecameradrag,
            movemodeattack: !!$scope.settings.movemodeattack,
            allowdeselect: !!$scope.settings.allowdeselect
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
                    // Legacy Quality.Autodetect (-1) → OA index 0.
                    $scope.settings[k] = res[k] < 0 ? OVERALL.Autodetect : clampInt(res[k], 0, 5, OVERALL.Custom);
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

    function loadUserSettings() {
        executeShell('/usersettings', null, function(res) {
            if (!res || typeof res !== 'object') {
                return;
            }
            if (typeof res.shellfullscreen === 'boolean') { $scope.settings.shellfullscreen = res.shellfullscreen; }
            if (typeof res.gamefullscreen === 'boolean') { $scope.settings.gamefullscreen = res.gamefullscreen; }
            if (typeof res.fullscreenwidth === 'number') { $scope.settings.fullscreenwidth = res.fullscreenwidth; }
            if (typeof res.fullscreenheight === 'number') { $scope.settings.fullscreenheight = res.fullscreenheight; }
            if (typeof res.windowedwidth === 'number') { $scope.settings.windowedwidth = res.windowedwidth; }
            if (typeof res.windowedheight === 'number') { $scope.settings.windowedheight = res.windowedheight; }
            if (typeof res.mastervolume === 'number') {
                var asPercent = res.mastervolume <= 10 ? (res.mastervolume * 10) : res.mastervolume;
                $scope.settings.mastervolume = Math.max(0, Math.min(100, Math.round(asPercent)));
            }
            if (typeof res.edgepan === 'boolean') { $scope.settings.edgepan = res.edgepan; }
            if (typeof res.edgescrollspeed === 'number') { $scope.settings.edgescrollspeed = Math.max(0, Math.min(100, Math.round(res.edgescrollspeed))); }
            if (typeof res.middlemousecameradrag === 'boolean') { $scope.settings.middlemousecameradrag = res.middlemousecameradrag; }
            if (typeof res.movemodeattack === 'boolean') { $scope.settings.movemodeattack = res.movemodeattack; }
            if (typeof res.allowdeselect === 'boolean') { $scope.settings.allowdeselect = res.allowdeselect; }
            readGraphicsFromResponse(res);
            syncResolutionModels();
            $scope.$applyAsync();
        });
    }

    function loadDisplayConfig() {
        executeShell('/config/display', null, function(res) {
            if (!res || typeof res !== 'object') {
                return;
            }
            var fullscreenResolutions = normalizeResolutionList(res.fullscreenResolutions);
            var windowedResolutions = normalizeResolutionList(res.windowedResolutions).filter(function(r) {
                return STANDARD_RESOLUTIONS.indexOf(r) !== -1;
            });
            if (fullscreenResolutions.length > 0) {
                $scope.fullscreenResolutionOptions = fullscreenResolutions;
            }
            if (windowedResolutions.length > 0) {
                $scope.windowedResolutionOptions = windowedResolutions;
            } else {
                $scope.windowedResolutionOptions = dedupeResolutionList(buildStandardResolutions(false).concat(['2560 X 1440', '3840 X 2160']));
            }
            syncResolutionModels();
            if ($scope.fullscreenResolutionOptions.indexOf($scope.settings.fullscreenResolution) === -1) {
                $scope.fullscreenResolutionOptions.unshift($scope.settings.fullscreenResolution);
            }
            if ($scope.windowedResolutionOptions.indexOf($scope.settings.windowedResolution) === -1) {
                $scope.windowedResolutionOptions.unshift($scope.settings.windowedResolution);
            }
            $scope.$applyAsync();
        });
    }

    function loadGraphicsOptions() {
        executeShell('/options/graphics/get', null, function(res) {
            if (!res || typeof res !== 'object') {
                return;
            }
            var fullscreenResolutions = normalizeResolutionList(res.fullscreenResolutions);
            var windowedResolutions = normalizeResolutionList(res.windowedResolutions).filter(function(r) {
                return STANDARD_RESOLUTIONS.indexOf(r) !== -1;
            });
            if (fullscreenResolutions.length > 0) {
                $scope.fullscreenResolutionOptions = fullscreenResolutions;
            }
            if (windowedResolutions.length > 0) {
                $scope.windowedResolutionOptions = windowedResolutions;
            } else {
                $scope.windowedResolutionOptions = dedupeResolutionList(buildStandardResolutions(false).concat(['2560 X 1440', '3840 X 2160']));
            }
            if ($scope.fullscreenResolutionOptions.indexOf($scope.settings.fullscreenResolution) === -1) {
                $scope.fullscreenResolutionOptions.unshift($scope.settings.fullscreenResolution);
            }
            if ($scope.windowedResolutionOptions.indexOf($scope.settings.windowedResolution) === -1) {
                $scope.windowedResolutionOptions.unshift($scope.settings.windowedResolution);
            }
            $scope.$applyAsync();
        });
    }

    $scope.setOptionsTab = function(tabName) {
        $scope.optionsTab = tabName;
    };

    $scope.applyGraphicsMode = function() {
        applyPartial({
            shellfullscreen: !!$scope.settings.shellfullscreen,
            gamefullscreen: !!$scope.settings.gamefullscreen
        });
    };

    $scope.applyFullscreenResolution = function() {
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
        var overall = clampInt($scope.settings.overallgraphicsquality, 0, 5, OVERALL.Custom);
        $scope.settings.overallgraphicsquality = overall;
        if (overall === OVERALL.Custom || overall === OVERALL.Autodetect) {
            // Autodetect: engine cascade fills children on commit. Custom: leave knobs as-is.
            applyPartial(buildGraphicsPayload());
            return;
        }
        applyPresetToSettings(overall);
        applyPartial(buildGraphicsPayload());
    };

    $scope.onGraphicsDetailChange = function() {
        if (!applyingOverallPreset && $scope.settings.overallgraphicsquality !== OVERALL.Custom) {
            $scope.settings.overallgraphicsquality = OVERALL.Custom;
        }
        applyPartial(buildGraphicsPayload());
    };

    $scope.onBrightnessChange = function() {
        var pct = clampInt($scope.settings.brightnessPercent, 0, 100, 50);
        $scope.settings.brightnessPercent = pct;
        $scope.settings.brightness = pct / 100;
        if (!applyingOverallPreset && $scope.settings.overallgraphicsquality !== OVERALL.Custom) {
            $scope.settings.overallgraphicsquality = OVERALL.Custom;
        }
        applyPartial(buildGraphicsPayload());
    };

    $scope.applyVolume = function() {
        var volume = Math.max(0, Math.min(100, Math.round($scope.settings.mastervolume)));
        $scope.settings.mastervolume = volume;
        applyPartial({mastervolume: volume / 10});
    };

    $scope.applyControls = function() {
        applyPartial({
            edgepan: !!$scope.settings.edgepan,
            edgescrollspeed: Math.max(0, Math.min(100, Math.round($scope.settings.edgescrollspeed))),
            middlemousecameradrag: !!$scope.settings.middlemousecameradrag
        });
    };

    $scope.applyGameplay = function() {
        applyPartial({
            movemodeattack: !!$scope.settings.movemodeattack,
            allowdeselect: !!$scope.settings.allowdeselect
        });
    };

    $scope.onShellThemeDraftChange = function() {
        if (!$scope.allowShellThemeSelect) {
            return;
        }
        refreshShellThemeHint();
    };

    $scope.applyShellTheme = function() {
        if (!$scope.allowShellThemeSelect) {
            return;
        }
        var id = $scope.settings.shellUiTheme || 'aurora';
        if (window.CncShellTheme && CncShellTheme.set) {
            id = CncShellTheme.set(id);
            $scope.settings.shellUiTheme = id;
            $scope.settings.shellUiThemeDefault = id;
        }
        refreshShellThemeHint();
    };

    $scope.applyShellThemeDefault = function() {
        if (!$scope.allowShellThemeSelect) {
            return;
        }
        var id = $scope.settings.shellUiThemeDefault || 'aurora';
        if (window.CncShellTheme && CncShellTheme.setDefault) {
            $scope.settings.shellUiThemeDefault = CncShellTheme.setDefault(id);
        }
    };

    $scope.restoreDefaults = function() {
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
        var payload = buildApplyPayloadFromSettings();
        var pendingTheme = null;
        if ($scope.allowShellThemeSelect) {
            pendingTheme = $scope.settings.shellUiTheme || 'aurora';
            themeSaveCommitted = true;
            themeSnapshot = {
                theme: pendingTheme,
                defaultTheme: pendingTheme
            };
            $scope.settings.shellUiTheme = pendingTheme;
            $scope.settings.shellUiThemeDefault = pendingTheme;
        }
        executeShell('/usersettings/apply', payload, function () {
            executeShell('/usersettings/applyAudio', null, function () {
                executeShell('/usersettings/save');
            });
        });
        $scope.closeOptions();
        if (pendingTheme && window.CncShellTheme && CncShellTheme.set) {
            CncShellTheme.set(pendingTheme);
            refreshShellThemeHint();
        }
    };

    $scope.actionCancel = function() {
        executeShell('/usersettings/discard');
        loadUserSettings();
        if ($scope.allowShellThemeSelect) {
            revertThemeDraft();
            themeSaveCommitted = false;
        }
        $scope.closeOptions();
    };

    $scope.$watch('optionsOpen', function(isOpen) {
        if (isOpen) {
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