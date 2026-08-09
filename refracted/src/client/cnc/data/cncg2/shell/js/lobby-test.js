/**
 * Skirmish lobby — map + general pick (1 host + 1 AI).
 *
 * StaticData/Generals + Faction*_Blueprints GeneralsToLoad — not UI slot 1..3.
 *
 */
(function (window) {
    'use strict';

    // Paths from initfs_Win32 level.cfg entries (skip placeholder Levels/name/name).
    var MAPS = [
        {
            id: 'Alpha_Tutorial',
            path: 'Levels/SP/Alpha_Tutorial/Alpha_Tutorial',
            label: 'Alpha Tutorial',
            forceTutorialGeneral: true
        },
        {
            id: 'DM_Smalltown_1v1_CR',
            path: 'Levels/MP/PVP/DM_Smalltown_1v1_CR/DM_Smalltown_1v1_CR',
            label: 'Smalltown 1v1'
        },
        {
            id: 'DM_KapuKai_1v1_JKS',
            path: 'Levels/MP/PVP/DM_KapuKai_1v1_JKS/DM_KapuKai_1v1_JKS',
            label: 'Kapu Kai 1v1'
        },
        {
            id: 'DM_Oasis_2v2_JT',
            path: 'Levels/MP/PVP/DM_Oasis_2v2_JT/DM_Oasis_2v2_JT',
            label: 'Oasis 2v2'
        },
        {
            id: 'DM_Monsoon_3v3_MO',
            path: 'Levels/MP/PVP/DM_Monsoon_3v3_MO/DM_Monsoon_3v3_MO',
            label: 'Monsoon 3v3'
        },
        {
            id: 'DM_Overpass_3v3_JKS',
            path: 'Levels/MP/PVP/DM_Overpass_3v3_JKS/DM_Overpass_3v3_JKS',
            label: 'Overpass 3v3'
        },
        {
            id: 'FirstPlayable_MPHorde_Final',
            path: 'Levels/MP/PVE/FirstPlayable_MPHorde_Final/FirstPlayable_MPHorde_Final',
            label: 'MP Horde (PVE)'
        },
        {
            id: 'XLevel_SP_final',
            path: 'Levels/XLevel_SP_final/XLevel_SP_final',
            label: 'XLevel SP'
        },
        {
            id: 'FrontEndTest',
            path: 'Levels/FrontEndTest/FrontEndTest',
            label: 'FrontEnd Test'
        }
    ];
    var BENCHMARK_MAP = MAPS[0];

    function findMapByPath(path) {
        var p = String(path || '');
        for (var i = 0; i < MAPS.length; i++) {
            if (MAPS[i].path.toLowerCase() === p.toLowerCase()) {
                return MAPS[i];
            }
        }
        return null;
    }

    function isTutorialMap(pathOrMap) {
        var m = typeof pathOrMap === 'object' && pathOrMap
            ? pathOrMap
            : findMapByPath(pathOrMap);
        return !!(m && m.forceTutorialGeneral);
    }

    // Faction code → playable generals (exclude Test*). Default = Classic.
    var GENERALS_BY_FACTION = {
        USA: [
            { id: 0, key: 'None', label: 'None (no Alpha data)' }
        ],
        APA: [
            { id: 2914080600, key: 'APA_ClassicGeneral', label: 'Classic' },
            { id: 3919700239, key: 'APA_AtomicGeneral', label: 'Atomic' },
            { id: 1259669531, key: 'APA_EspionageGeneral', label: 'Espionage' },
            { id: 73570905, key: 'APA_FlameGeneral', label: 'Flame' },
            { id: 1113071128, key: 'APA_RocketGeneral', label: 'Rocket' },
            { id: 3042903044, key: 'APA_UrbanGeneral', label: 'Urban' },
            { id: 497011786, key: 'APA_TutorialGeneral', label: 'Tutorial' }
        ],
        EU: [
            { id: 232716472, key: 'EU_ClassicGeneral', label: 'Classic' },
            { id: 76852445, key: 'EU_AirforceGeneral', label: 'Airforce' },
            { id: 162564633, key: 'EU_LaserGeneral', label: 'Laser' },
            { id: 1523019091, key: 'EU_RapidAssaultGeneral', label: 'Rapid Assault' },
            { id: 2373292237, key: 'EU_ReconGeneral', label: 'Recon' },
            { id: 3943761922, key: 'EU_ScienceGeneral', label: 'Science' },
            { id: 3463861546, key: 'EU_TutorialGeneral', label: 'Tutorial' }
        ],
        ESC: null, // alias → EU
        GLA: [
            { id: 580378690, key: 'GLA_ClassicGeneral', label: 'Classic' },
            { id: 1632710820, key: 'GLA_DemolitionGeneral', label: 'Demolition' },
            { id: 1975634889, key: 'GLA_EngineerGeneral', label: 'Engineer' },
            { id: 2905920036, key: 'GLA_JunkyardGeneral', label: 'Junkyard' },
            { id: 2837579969, key: 'GLA_MarauderGeneral', label: 'Marauder' },
            { id: 901813698, key: 'GLA_ToxinGeneral', label: 'Toxin' },
            { id: 145977592, key: 'GLA_TutorialGeneral', label: 'Tutorial' }
        ]
    };
    GENERALS_BY_FACTION.ESC = GENERALS_BY_FACTION.EU;

    function emptySlot() {
        return { occupied: false };
    }

    /** Thenable helper — game CEF may lack Promise; never touch Promise at controller init. */
    function whenAll(tasks, onDone, onFail) {
        var list = tasks || [];
        var left = list.length;
        var failed = false;
        if (!left) {
            if (onDone) {
                onDone();
            }
            return;
        }
        function finishOk() {
            if (failed) {
                return;
            }
            left -= 1;
            if (left <= 0 && onDone) {
                onDone();
            }
        }
        function finishFail() {
            if (failed) {
                return;
            }
            failed = true;
            if (onFail) {
                onFail();
            }
        }
        for (var i = 0; i < list.length; i++) {
            (function (t) {
                try {
                    if (t && typeof t.then === 'function') {
                        t.then(function (ok) {
                            if (ok === false) {
                                finishFail();
                            } else {
                                finishOk();
                            }
                        }, finishFail);
                    } else {
                        finishOk();
                    }
                } catch (e) {
                    finishFail();
                }
            })(list[i]);
        }
    }

    function syncOk(value) {
        // Prefer native Promise when present; else a minimal thenable (no Promise global needed).
        if (typeof Promise !== 'undefined' && Promise.resolve) {
            return Promise.resolve(value);
        }
        return {
            then: function (resolve) {
                try {
                    resolve(value);
                } catch (e) { /* empty */ }
                return this;
            },
            catch: function () {
                return this;
            }
        };
    }

    /**
     * HTTP helper for game CEF (often no window.fetch). Prefers jQuery, then XHR, then fetch.
     * Returns a multi-subscriber thenable resolving to parsed JSON, true, or false.
     */
    function httpRequest(method, url, opts) {
        opts = opts || {};
        var wantJson = opts.json !== false;
        var settled = false;
        var value = false;
        var waiters = [];

        function settle(v) {
            if (settled) {
                return;
            }
            settled = true;
            value = v;
            for (var i = 0; i < waiters.length; i++) {
                try {
                    waiters[i](value);
                } catch (e) { /* empty */ }
            }
            waiters = [];
        }

        function parseAndSettle(status, text) {
            if (status < 200 || status >= 300) {
                settle(false);
                return;
            }
            if (!wantJson) {
                settle(true);
                return;
            }
            if (!text) {
                settle(true);
                return;
            }
            try {
                settle(JSON.parse(text));
            } catch (e) {
                settle(true);
            }
        }

        try {
            if (window.jQuery && typeof window.jQuery.ajax === 'function') {
                window.jQuery.ajax({
                    type: method,
                    url: url,
                    dataType: 'text',
                    cache: false,
                    success: function (data, _st, xhr) {
                        parseAndSettle((xhr && xhr.status) || 200, data == null ? '' : String(data));
                    },
                    error: function (xhr) {
                        parseAndSettle((xhr && xhr.status) || 0, '');
                    }
                });
            } else if (typeof XMLHttpRequest !== 'undefined') {
                var xhr = new XMLHttpRequest();
                xhr.open(method, url, true);
                xhr.onreadystatechange = function () {
                    if (xhr.readyState !== 4) {
                        return;
                    }
                    parseAndSettle(xhr.status, xhr.responseText || '');
                };
                xhr.send(null);
            } else if (window.fetch) {
                fetch(url, { method: method, credentials: 'same-origin' })
                    .then(function (r) {
                        return r.text().then(function (t) {
                            parseAndSettle(r.status, t || '');
                        });
                    })
                    .catch(function () {
                        settle(false);
                    });
            } else {
                settle(false);
            }
        } catch (e) {
            settle(false);
        }

        return {
            then: function (resolve) {
                if (typeof resolve !== 'function') {
                    return this;
                }
                if (settled) {
                    try {
                        resolve(value);
                    } catch (e) { /* empty */ }
                } else {
                    waiters.push(resolve);
                }
                return this;
            },
            catch: function () {
                return this;
            }
        };
    }

    function normalizeFactionCode(code) {
        var c = String(code || '').toUpperCase();
        if (c === 'ESC') {
            return 'EU';
        }
        return c;
    }

    function generalsForFaction(code) {
        var c = normalizeFactionCode(code);
        return GENERALS_BY_FACTION[c] || [];
    }

    function tutorialGeneralId(code) {
        var list = generalsForFaction(code);
        for (var i = 0; i < list.length; i++) {
            if (/Tutorial/i.test(list[i].key) && list[i].id) {
                return list[i].id;
            }
        }
        return 0;
    }

    function defaultGeneralId(code, mapPath) {
        if (isTutorialMap(mapPath)) {
            var tut = tutorialGeneralId(code);
            if (tut) {
                return tut;
            }
        }
        var list = generalsForFaction(code);
        if (!list.length) {
            return 0;
        }
        // Prefer Classic when present; else first entry (USA sentinel id=0).
        for (var i = 0; i < list.length; i++) {
            if (/Classic/i.test(list[i].key) && list[i].id) {
                return list[i].id;
            }
        }
        return list[0].id;
    }

    function generalLabel(code, generalId) {
        var id = Number(generalId) || 0;
        var list = generalsForFaction(code);
        for (var i = 0; i < list.length; i++) {
            if (list[i].id === id) {
                return list[i].label;
            }
        }
        if (!id) {
            return 'NONE';
        }
        return 'GENERAL';
    }

    function codenameForSlot(slot) {
        if (!slot) {
            return 'GENERAL';
        }
        var label = generalLabel(slot.faction, slot.general);
        if (label && label !== 'NONE' && label !== 'GENERAL') {
            return String(label).toUpperCase();
        }
        var map = { USA: 'TASKMASTER', APA: 'TACTICIAN', ESC: 'RED ARROW', GLA: 'GHOST', EU: 'CLEAVER' };
        return map[normalizeFactionCode(slot.faction)] || 'GENERAL';
    }

    function difficultyIndex(diff) {
        var map = { EASY: 0, MEDIUM: 1, HARD: 2 };
        var key = String(diff || 'MEDIUM').toUpperCase();
        return map[key] != null ? map[key] : 1;
    }

    function difficultyAttrValue(diff) {
        return String(difficultyIndex(diff));
    }
    var appModule;
    try {
        appModule = angular.module('CCApp');
    } catch (e) {
        appModule = angular.module('CCApp', []);
    }

    appModule.controller('LobbyController', function ($scope, $timeout, $rootScope) {
        $rootScope.lobbySidebarTab = 'CHAT';
        $scope.maps = MAPS;
        $scope.mapLabel = BENCHMARK_MAP.label;
        $scope.mapPath = BENCHMARK_MAP.path;
        $scope.mapId = BENCHMARK_MAP.id;
        $scope.mapForcesTutorial = !!BENCHMARK_MAP.forceTutorialGeneral;
        $scope.matchLabel = '1v1 Skirmish';
        $scope.gameId = '1';
        $scope.onlineCount = 0;
        $scope.debugLines = [];
        $scope.hostReady = false;
        $scope.aiDifficulties = ['EASY', 'MEDIUM', 'HARD'];
        // Alpha build: USA has PlayerData but no StaticData/Generals — prefer APA default.
        $scope.factions = [
            { code: 'APA' },
            { code: 'EU' },
            { code: 'GLA' },
            { code: 'USA' }
        ];

        $scope.team1 = [emptySlot(), emptySlot(), emptySlot()];
        $scope.team2 = [emptySlot(), emptySlot(), emptySlot()];

        $scope.selectedSlot = null;
        $scope.selectedTeam = 0;

        $scope.pushDebug = function (msg) {
            var line = String(msg || '');
            if (!line) {
                return;
            }
            var stamp = '';
            try {
                var d = new Date();
                stamp = (d.getHours() < 10 ? '0' : '') + d.getHours() + ':' +
                    (d.getMinutes() < 10 ? '0' : '') + d.getMinutes() + ':' +
                    (d.getSeconds() < 10 ? '0' : '') + d.getSeconds() + ' ';
            } catch (e) { /* empty */ }
            $scope.debugLines.push(stamp + line);
            if ($scope.debugLines.length > 80) {
                $scope.debugLines.splice(0, $scope.debugLines.length - 80);
            }
            $timeout(function () {
                var el = document.getElementById('cnc-lobby-debug-log');
                if (el) {
                    el.scrollTop = el.scrollHeight;
                }
            }, 0);
            if (window.CncProbe && CncProbe.log) {
                CncProbe.log('[lobby] ' + line);
            }
        };

        $scope.clearDebug = function ($event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            $scope.debugLines = [];
        };

        $scope.generalsForFaction = generalsForFaction;
        $scope.generalLabel = generalLabel;
        $scope.generalOptions = function (slot) {
            var list = generalsForFaction(slot && slot.faction);
            if (!$scope.mapForcesTutorial) {
                return list;
            }
            var out = [];
            for (var i = 0; i < list.length; i++) {
                if (/Tutorial/i.test(list[i].key) && list[i].id) {
                    out.push(list[i]);
                }
            }
            return out.length ? out : list;
        };
        $scope.openMenu = null;
        $scope.toggleMenu = function (id, $event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            $scope.openMenu = ($scope.openMenu === id) ? null : id;
        };
        $scope.closeMenu = function () {
            $scope.openMenu = null;
        };

        $scope.team1Occupied = function () {
            var n = 0;
            for (var i = 0; i < $scope.team1.length; i++) {
                if ($scope.team1[i].occupied) {
                    n++;
                }
            }
            return n;
        };

        $scope.team2Occupied = function () {
            var n = 0;
            for (var i = 0; i < $scope.team2.length; i++) {
                if ($scope.team2[i].occupied) {
                    n++;
                }
            }
            return n;
        };

        // String ng-class — CEF/old Angular: object literals from functions can miss styles.
        $scope.factionSwatchClass = function (code, active) {
            var c = String(code || '').toLowerCase();
            var out = 'swatch-' + c;
            if (active) {
                out += ' active';
            }
            return out;
        };

        $scope.factionCssClass = function (code) {
            if (!code) {
                return '';
            }
            return 'faction-' + String(code).toLowerCase();
        };

        $scope.startpointLabel = function (n) {
            if (n == null || n === '') {
                return '—';
            }
            return 'Slot ' + n;
        };

        function syncMapToServer() {
            var level = $scope.mapPath || BENCHMARK_MAP.path;
            var url = '/cnc/select-map?gid=' + encodeURIComponent($scope.gameId) +
                '&path=' + encodeURIComponent(level);
            var t = httpRequest('POST', url, { json: false });
            return {
                then: function (resolve) {
                    t.then(function (ok) {
                        resolve(ok !== false);
                    });
                    return this;
                },
                catch: function () {
                    return this;
                }
            };
        }

        /** When Alpha Tutorial is selected, force Tutorial general for every occupied slot. */
        function applyTutorialGeneralConstraint(syncAttrs) {
            if (!$scope.mapForcesTutorial) {
                return;
            }
            var teams = [$scope.team1, $scope.team2];
            for (var t = 0; t < teams.length; t++) {
                for (var i = 0; i < teams[t].length; i++) {
                    var slot = teams[t][i];
                    if (!slot || !slot.occupied) {
                        continue;
                    }
                    var want = tutorialGeneralId(slot.faction);
                    if (!want || slot.general === want) {
                        continue;
                    }
                    slot.general = want;
                    slot.codename = codenameForSlot(slot);
                    if (syncAttrs) {
                        sendAttr('_general', String(want), slot);
                    }
                }
            }
        }

        $scope.setMap = function (map) {
            if (!map || !map.path) {
                return;
            }
            $scope.mapId = map.id;
            $scope.mapPath = map.path;
            $scope.mapLabel = map.label;
            $scope.mapForcesTutorial = !!map.forceTutorialGeneral;
            $scope.closeMenu();
            applyTutorialGeneralConstraint(true);
            syncMapToServer();
            $scope.pushDebug('Map ' + map.label +
                ($scope.mapForcesTutorial ? ' · Tutorial general required' : ''));
            if (window.CncProbe && CncProbe.log) {
                CncProbe.log('select-map → ' + map.path);
            }
        };

        function syncPlayerAttrsToServer(slot) {
            if (!slot) {
                return syncOk(false);
            }
            var pid = slot.pid || 0;
            var q = '/cnc/player-attrs?gid=' + encodeURIComponent($scope.gameId) +
                '&pid=' + encodeURIComponent(pid);
            if (slot.faction) {
                q += '&faction=' + encodeURIComponent(slot.faction);
            }
            if (slot.teamNum != null) {
                q += '&team=' + encodeURIComponent(slot.teamNum);
            }
            if (slot.startpoint != null) {
                q += '&startpoint=' + encodeURIComponent(slot.startpoint);
            }
            if (slot.general != null && slot.general !== '') {
                q += '&general=' + encodeURIComponent(slot.general);
            }
            if (slot.isAi) {
                q += '&isai=1';
            } else {
                q += '&isai=0';
            }
            if (slot.difficulty) {
                q += '&difficulty=' + encodeURIComponent(difficultyAttrValue(slot.difficulty));
            }
            var t = httpRequest('POST', q, { json: true });
            return {
                then: function (resolve) {
                    t.then(function (body) {
                        if (window.CncProbe && CncProbe.log && body && body !== true && body !== false) {
                            CncProbe.log('player-attrs ok=' + !!body.ok +
                                ' faction=' + (slot.faction || '?') +
                                ' general=' + (slot.general != null ? slot.general : '?') +
                                ' team=' + (slot.teamNum || '?') +
                                ' start=' + (slot.startpoint || '?'));
                        }
                        if (body === false) {
                            resolve(false);
                            return;
                        }
                        if (body === true || (body && body.ok)) {
                            resolve(true);
                            return;
                        }
                        resolve(false);
                    });
                    return this;
                },
                catch: function () {
                    return this;
                }
            };
        }

        function probePlayerData() {
            httpRequest('GET', '/cnc/player-probe?gid=' + encodeURIComponent($scope.gameId), { json: true })
                .then(function (body) {
                    if (!body || body === true) {
                        return;
                    }
                    if (window.CncProbe && CncProbe.log) {
                        CncProbe.log('player-probe ' + JSON.stringify(body));
                    }
                    if (body.issues && body.issues.length && !$scope.$$phase) {
                        $scope.$apply(function () {
                            $scope.pushDebug('Player data: ' + body.issues.join('; '));
                        });
                    }
                });
        }

        function refreshOnlineCount() {
            httpRequest('GET', '/cnc/online-count', { json: true }).then(function (body) {
                if (!body || body === true || !body.ok) {
                    return;
                }
                var n = body.count != null ? body.count : body.active;
                if (n == null) {
                    return;
                }
                if (!$scope.$$phase) {
                    $scope.$apply(function () {
                        $scope.onlineCount = n;
                    });
                } else {
                    $scope.onlineCount = n;
                }
            });
        }

        function sendDifficulty(slot, diff) {
            if (!slot || !slot.isAi) {
                return;
            }
            var idx = difficultyIndex(diff);
            if (window.CncProbe && CncProbe.runGame) {
                CncProbe.runGame('Network.DifficultyChanged ' + idx);
            }
            sendAttr('_difficulty', difficultyAttrValue(diff), slot);
        }

        function syncHostFromSession() {
            if (window.CncBlazeState && CncBlazeState.applyExternalHints) {
                CncBlazeState.applyExternalHints();
            }
            var pid = window.CncProbe ? CncProbe.resolveHostPid() : '';
            var name = window.CncProbe ? CncProbe.resolveHostName() : '';
            if (!name && window.CncBlazeState) {
                name = CncBlazeState.getPlayerName();
            }
            if (!name || name === 'Guest') {
                name = 'Player';
            }
            var display = String(name).toUpperCase();
            var local = String(name);

            var host = $scope.team1[0];
            host.occupied = true;
            host.isLocal = true;
            host.isAi = false;
            host.pid = pid || host.pid || '';
            host.displayName = display;
            host.localName = local;
            host.faction = host.faction || 'APA';
            if (host.general == null || host.general === '') {
                host.general = defaultGeneralId(host.faction, $scope.mapPath);
            } else if ($scope.mapForcesTutorial) {
                var tut = tutorialGeneralId(host.faction);
                if (tut) {
                    host.general = tut;
                }
            }
            host.codename = codenameForSlot(host);
            host.startpoint = host.startpoint || 1;
            host.teamNum = 1;
            host.ready = !!pid;
            host.difficulty = 'MEDIUM';
            if (host.faction) {
                syncPlayerAttrsToServer(host);
            }

            $scope.hostReady = !!pid;
            if (!pid) {
                $scope.pushDebug('Waiting for persona ID — authenticate via shell first.');
            } else {
                $scope.pushDebug('Host ready · PID ' + pid);
            }

            if (!$scope.selectedSlot || $scope.selectedSlot.isLocal) {
                $scope.selectedSlot = host;
                $scope.selectedTeam = 1;
            }
            // NOTE: faction is LOCAL during the lobby (matches retail). It is NOT sent to Blaze here —
            // there is no Blaze game yet (the game is created at Start Battle). Faction + all slot attrs
            // are applied in startBattle() AFTER blazeCreateGame, once a joined Game object exists.
        }

        function firstEmptySlot(team) {
            for (var i = 0; i < team.length; i++) {
                if (!team[i].occupied) {
                    return team[i];
                }
            }
            return null;
        }

        function startpointForSlot(teamNum, slotIndex) {
            if (teamNum === 1) {
                return slotIndex + 1;
            }
            return slotIndex + 2;
        }

        function fillAiSlot(slot, aiPid, teamNum, startpoint) {
            slot.occupied = true;
            slot.isAi = true;
            slot.isLocal = false;
            slot.pid = aiPid ? String(aiPid) : '';
            slot.displayName = 'AI_1';
            slot.faction = 'APA';
            slot.general = defaultGeneralId('APA', $scope.mapPath);
            slot.codename = codenameForSlot(slot);
            slot.startpoint = startpoint != null ? startpoint : 2;
            slot.teamNum = teamNum != null ? teamNum : 2;
            slot.difficulty = 'MEDIUM';
            slot.ready = true;
        }

        function persistAiPid(pid) {
            if (!pid) {
                return;
            }
            var s = String(pid).trim();
            if (!s) {
                return;
            }
            if (window.CncProbe) {
                CncProbe._lobbyAiPid = s;
            }
            try {
                sessionStorage.setItem('cnc_lobby_ai_pid', s);
            } catch (e) { /* empty */ }
        }

        function applyAiAttrs(slot) {
            if (!slot || !slot.occupied || !slot.isAi) {
                return;
            }
            var pid = slot.pid || (window.CncProbe && CncProbe._lobbyAiPid) || '';
            if (!pid) {
                $scope.pushDebug('AI slot open — waiting for persona ID from blazeGetPlayers / server log.');
                if (window.CncProbe && CncProbe.runGame) {
                    CncProbe.runGame('RtsClient.blazeGetPlayers ' + $scope.gameId);
                }
                return;
            }
            slot.pid = pid;
            persistAiPid(pid);
            $scope.selectedSlot = slot;
            $scope.selectedTeam = slot.teamNum || 2;
            sendAttr('_isai', '1', slot);
            $timeout(function () {
                sendAttr('_faction', slot.faction, slot);
            }, 250);
            $timeout(function () {
                sendAttr('_general', String(slot.general || 0), slot);
            }, 400);
            $timeout(function () {
                sendAttr('_startpoint', String(slot.startpoint), slot);
            }, 550);
            $timeout(function () {
                sendAttr('_team', String(slot.teamNum), slot);
            }, 750);
            $timeout(function () {
                sendDifficulty(slot, slot.difficulty || 'MEDIUM');
            }, 1000);
            $scope.pushDebug('AI ready · PID ' + pid);
        }

        function loadAiFromStorage() {
            try {
                var raw = sessionStorage.getItem('cnc_lobby_ai_pid');
                if (!raw && window.CncProbe && CncProbe._lobbyAiPid) {
                    raw = CncProbe._lobbyAiPid;
                }
                if (!raw) {
                    return;
                }
                var aiPid = String(raw).trim();
                if (!aiPid) {
                    return;
                }
                for (var i = 0; i < $scope.team2.length; i++) {
                    if ($scope.team2[i].occupied && $scope.team2[i].pid === aiPid) {
                        return;
                    }
                }
                var slot = firstEmptySlot($scope.team2);
                if (!slot) {
                    return;
                }
                fillAiSlot(slot, aiPid, 2, 2);
                persistAiPid(aiPid);
            } catch (e) { /* empty */ }
        }

        function sendAttr(key, value, slot) {
            if (!slot) {
                $scope.pushDebug('No slot for attribute ' + key);
                return;
            }
            // Blaze setPlayerAttributes deadlocks / crashes the shell path. Push via Refracted
            // HTTP side-channel (same pattern as /cnc/select-map) so CreateGame + ServerHello
            // see lobby faction/team/start/general.
            if (key === '_faction') {
                slot.faction = value;
            } else if (key === '_team') {
                slot.teamNum = parseInt(value, 10) || slot.teamNum;
            } else if (key === '_startpoint') {
                slot.startpoint = parseInt(value, 10) || slot.startpoint;
            } else if (key === '_general') {
                slot.general = Number(value) || 0;
                slot.codename = codenameForSlot(slot);
            } else if (key === '_isai') {
                slot.isAi = value === '1' || value === 'true';
            } else if (key === '_difficulty') {
                /* difficultyAttrValue already applied on slot in callers */
            }
            syncPlayerAttrsToServer(slot);
            $scope.pushDebug(key + '=' + value + ' (Refracted /cnc/player-attrs)');
            if (window.CncProbe && CncProbe.log) {
                CncProbe.log('sendAttr → HTTP ' + key + '=' + value);
            }
        }

        $scope.applySelectedSlot = function () {
            var slot = $scope.selectedSlot;
            if (!slot || !slot.occupied) {
                return;
            }
            slot.codename = codenameForSlot(slot);
            sendAttr('_faction', slot.faction, slot);
            $timeout(function () {
                sendAttr('_general', String(slot.general || 0), slot);
            }, 200);
            $timeout(function () {
                sendAttr('_startpoint', String(slot.startpoint), slot);
            }, 400);
            $timeout(function () {
                sendAttr('_team', String(slot.teamNum), slot);
            }, 600);
            $timeout(function () {
                sendAttr('_isai', slot.isAi ? '1' : '0', slot);
            }, 800);
        };

        $scope.setFaction = function (code) {
            var host = $scope.team1[0];
            if (!host || !host.occupied || !host.isLocal) {
                return;
            }
            host.faction = code;
            host.general = defaultGeneralId(code, $scope.mapPath);
            host.codename = codenameForSlot(host);
            $scope.selectedSlot = host;
            $scope.selectedTeam = 1;
            sendAttr('_faction', code, host);
            $timeout(function () {
                sendAttr('_general', String(host.general || 0), host);
            }, 150);
            $scope.pushDebug('Faction ' + code + ' · ' + host.codename +
                (host.general ? '' : ' (no Alpha general)'));
        };

        $scope.setGeneral = function (generalId) {
            var host = $scope.team1[0];
            if (!host || !host.occupied || !host.isLocal) {
                return;
            }
            var id = Number(generalId) || 0;
            if ($scope.mapForcesTutorial) {
                var forced = tutorialGeneralId(host.faction);
                if (forced) {
                    id = forced;
                }
            }
            host.general = id;
            host.codename = codenameForSlot(host);
            $scope.selectedSlot = host;
            $scope.selectedTeam = 1;
            sendAttr('_general', String(host.general), host);
            $scope.pushDebug('General ' + host.codename + ' (' + host.general + ')');
        };

        $scope.setAiFaction = function (slot, code, $event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            if (!slot || !slot.isAi) {
                return;
            }
            slot.faction = code;
            slot.general = defaultGeneralId(code, $scope.mapPath);
            slot.codename = codenameForSlot(slot);
            $scope.selectedSlot = slot;
            $scope.selectedTeam = slot.teamNum || 2;
            syncPlayerAttrsToServer(slot);
            $scope.pushDebug('AI faction ' + code + ' · ' + slot.codename);
        };

        $scope.setAiGeneral = function (slot, generalId, $event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            if (!slot || !slot.isAi) {
                return;
            }
            var id = Number(generalId) || 0;
            if ($scope.mapForcesTutorial) {
                var forced = tutorialGeneralId(slot.faction);
                if (forced) {
                    id = forced;
                }
            }
            slot.general = id;
            slot.codename = codenameForSlot(slot);
            $scope.selectedSlot = slot;
            $scope.selectedTeam = slot.teamNum || 2;
            syncPlayerAttrsToServer(slot);
            $scope.pushDebug('AI general ' + slot.codename + ' (' + slot.general + ')');
        };

        $scope.selectSlot = function (slot, teamNum, $event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            if (!slot || !slot.occupied) {
                return;
            }
            $scope.selectedSlot = slot;
            $scope.selectedTeam = teamNum;
        };

        $scope.setAiDifficulty = function (slot, diff, $event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            if (!slot || !slot.isAi) {
                return;
            }
            slot.difficulty = diff;
            $scope.selectedSlot = slot;
            $scope.selectedTeam = slot.teamNum || 2;
            // Local only — applied to Blaze in startBattle() after the AI is added to the game.
            $scope.pushDebug('AI difficulty ' + diff + ' (applied at Start Battle)');
        };

        $scope.removeAiSlot = function (slot, $event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            var teams = [$scope.team1, $scope.team2];
            for (var t = 0; t < teams.length; t++) {
                for (var i = 0; i < teams[t].length; i++) {
                    if (teams[t][i] === slot) {
                        teams[t][i] = emptySlot();
                        try {
                            sessionStorage.removeItem('cnc_lobby_ai_pid');
                        } catch (e) { /* empty */ }
                        if (window.CncProbe) {
                            CncProbe._lobbyAiPid = null;
                        }
                        if ($scope.selectedSlot === slot) {
                            $scope.selectedSlot = $scope.team1[0];
                            $scope.selectedTeam = 1;
                        }
                        return;
                    }
                }
            }
        };

        $scope.inviteFriend = function ($event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            $scope.pushDebug('Invite friend — not wired in test lobby.');
        };

        $scope.addAi = function ($event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            if (!window.CncProbe || !CncProbe.runGame) {
                $scope.pushDebug('gameclient unavailable — open lobby from in-game shell.');
                return;
            }
            var slot = firstEmptySlot($scope.team2);
            if (!slot) {
                $scope.pushDebug('Team 2 has no open slot.');
                return;
            }
            var teamNum = 2;
            var slotIndex = 0;
            for (var i = 0; i < $scope.team2.length; i++) {
                if ($scope.team2[i] === slot) {
                    slotIndex = i;
                    break;
                }
            }
            var startpoint = startpointForSlot(teamNum, slotIndex);
            // Reserve the AI slot LOCALLY only. AddRemotePlayer (GMGR addQueuedPlayerToGame) requires a
            // created/joined game, which does not exist until Start Battle. The slot is materialized in
            // startBattle(): blazeCreateGame -> AddRemotePlayer for each reserved AI slot -> attrs -> start.
            fillAiSlot(slot, '', teamNum, startpoint);
            $scope.selectedSlot = slot;
            $scope.selectedTeam = teamNum;
            $scope.pushDebug('AI slot reserved (team ' + teamNum + ', start ' + startpoint + ') — added at Start Battle');
        };

        function aiSlots() {
            var out = [];
            for (var i = 0; i < $scope.team2.length; i++) {
                if ($scope.team2[i].occupied && $scope.team2[i].isAi) {
                    out.push($scope.team2[i]);
                }
            }
            return out;
        }

        //      = the "Setting up game" step. Synchronous runner: returns once the game is created/joined.
        //   3. Apply host faction + add AI slots (AddRemotePlayer) + AI attrs   (now there IS a Game to act on)
        //   4. RtsClient.StartGame -> ClientNetworkAdapter::startGame -> level load -> ingame.
        // blaze* commands block until their RPC completes, so we space the stages with $timeout to let
        // NotifyGameSetup / addQueuedPlayerToGame settle and to keep the UI responsive.
        $scope.startBattle = function () {
            if (!window.CncProbe || !CncProbe.runGame) {
                $scope.pushDebug('gameclient unavailable — open lobby from in-game shell.');
                return;
            }
            if ($scope._starting) {
                return;
            }
            $scope._starting = true;
            $scope.pushDebug('Setting up game…');
            applyTutorialGeneralConstraint(false);

            var host = $scope.team1[0];
            var gname = (host && host.localName) || 'Player1';
            var ais = aiSlots();
            var capacity = Math.max(2, 1 + ais.length);
            var level = $scope.mapPath || BENCHMARK_MAP.path;

            // 1. Create / claim the dedicated game via the ASYNC shell path (/blaze/createGame).
            if (CncProbe.markBlazeCreatePending) {
                CncProbe.markBlazeCreatePending();
            }
            // Record map + host/AI attrs with Refracted and WAIT for HTTP before createGame.
            // Prior race: fire-and-forget fetch lost to resetDedicatedServer → seed used USA/general=0.
            // Use whenAll (not Promise.all) — in-game CEF may lack Promise.
            var pending = [syncMapToServer()];
            if (host) {
                pending.push(syncPlayerAttrsToServer(host));
            }
            ais.forEach(function (slot) {
                pending.push(syncPlayerAttrsToServer(slot));
            });
            whenAll(pending, function () {
                beginCreate();
            }, function () {
                // Soft-fail: CEF/XHR glitch must not block Start Battle — server defaults APA Classic.
                if (window.CncProbe && CncProbe.log) {
                    CncProbe.log('Lobby attr sync soft-fail — continuing createGame');
                }
                $scope.pushDebug('Attr sync weak — starting with server defaults…');
                beginCreate();
            });

            function beginCreate() {
                probePlayerData();
                CncProbe.log('Lobby START: shell createGame ' + gname + ' players=' + capacity +
                    ' (level ' + level + ') faction=' + (host && host.faction) +
                    ' general=' + (host && host.general));
                CncProbe.runBlazeUrl(CncProbe.blazeUrlFromResource('creategame', {
                    gameName: gname, players: capacity, level: level
                }));
                CncProbe._inBlazeGame = true;

                $timeout(function () {
                    if (host) {
                        syncPlayerAttrsToServer(host);
                        probePlayerData();
                    }
                    ais.forEach(function (slot, idx) {
                        $timeout(function () {
                            CncProbe.log('Lobby START: AddRemotePlayer ' + (slot.teamNum || 2) + ' ' + (slot.startpoint || 2));
                            if (CncProbe.runAddRemotePlayer) {
                                CncProbe.runAddRemotePlayer(slot.teamNum || 2, slot.startpoint || 2, {
                                    gameId: $scope.gameId,
                                    pollDelayMs: 600
                                });
                            } else {
                                CncProbe.runGame('RtsClient.AddRemotePlayer ' + (slot.teamNum || 2) + ' ' + (slot.startpoint || 2));
                            }
                            $timeout(function () {
                                loadAiFromStorage();
                                applyAiAttrs(slot);
                            }, 1200);
                        }, 300 * idx);
                    });
                }, 2500);

                CncProbe.log(
                    'Lobby START: waiting for Blaze GameReady / LeaveIngame (no RtsClient.StartGame)');
                $scope.pushDebug('Waiting for dedicated match ready…');
            }
        };

        $scope.exitLobby = function () {
            if ($rootScope) {
                $rootScope.lobbySidebarTab = null;
            }
            if (/lobby\.html/i.test(window.location.pathname || '')) {
                window.location.href = 'index.html';
            } else {
                window.location.hash = '#/';
            }
        };

        syncHostFromSession();
        syncMapToServer();
        loadAiFromStorage();
        refreshOnlineCount();

        var onlinePoll = setInterval(refreshOnlineCount, 30000);
        $scope.$on('$destroy', function () {
            clearInterval(onlinePoll);
        });

        if (window.CncBlazeState) {
            CncBlazeState.subscribe(function () {
                if (!$scope.$$phase) {
                    $scope.$apply(function () {
                        syncHostFromSession();
                    });
                } else {
                    syncHostFromSession();
                }
            });
        }

        [200, 800, 2000, 5000].forEach(function (ms) {
            $timeout(function () {
                syncHostFromSession();
                loadAiFromStorage();
            }, ms);
        });
    });
})(window);
