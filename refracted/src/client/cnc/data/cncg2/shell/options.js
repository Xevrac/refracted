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
        windowedResolution: systemWindowedResolution
    };

    $scope.defaultSettings = angular.copy($scope.settings);

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
        return {
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

    $scope.restoreDefaults = function() {
        $scope.settings = angular.copy($scope.defaultSettings);
        syncResolutionModels();
        applyPartial(buildApplyPayloadFromSettings());
        executeShell('/usersettings/applyAudio');
    };

    $scope.actionSave = function() {
        var payload = buildApplyPayloadFromSettings();
        executeShell('/usersettings/apply', payload, function () {
            executeShell('/usersettings/applyAudio', null, function () {
                executeShell('/usersettings/save');
            });
        });
        $scope.closeOptions();
    };

    $scope.actionCancel = function() {
        executeShell('/usersettings/discard');
        loadUserSettings();
        $scope.closeOptions();
    };

    $scope.$watch('optionsOpen', function(isOpen) {
        if (isOpen) {
            loadUserSettings();
            loadDisplayConfig();
            loadGraphicsOptions();
        }
    });

    syncResolutionModels();
    loadUserSettings();
    loadDisplayConfig();
    loadGraphicsOptions();
});