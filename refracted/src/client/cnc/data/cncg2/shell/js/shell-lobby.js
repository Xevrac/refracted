/**
 * Shell lobby — Classic / Aurora match setup + Blaze start wiring.
 * Tutorial map rules + CreateGame path borrowed from lobby-test.js.
 * EAWebKit 13.2 / Angular 1.1.5 / ES5.
 */
(function (window, angular) {
    'use strict';

    if (!angular || !window.CCApp) {
        return;
    }

    var MAPS = [
        {
            id: 'Alpha_Tutorial',
            path: 'Levels/SP/Alpha_Tutorial/Alpha_Tutorial',
            label: 'Alpha Tutorial',
            slots: 1,
            forceTutorialGeneral: true,
            forceFaction: 'EU'
        },
        { id: 'excavation', label: 'Excavation', path: 'Levels/MP/PVP/DM_Smalltown_1v1_CR/DM_Smalltown_1v1_CR', slots: 2 },
        { id: 'nile-1v1', label: 'Nile Delta (1v1)', path: 'Levels/MP/PVP/DM_KapuKai_1v1_JKS/DM_KapuKai_1v1_JKS', slots: 2 },
        { id: 'redzone-1v1', label: 'Red Zone (1v1)', path: 'Levels/MP/PVP/DM_Oasis_2v2_JT/DM_Oasis_2v2_JT', slots: 2 },
        { id: 'nile-2v2', label: 'Nile Delta (2v2)', path: 'Levels/MP/PVP/DM_Oasis_2v2_JT/DM_Oasis_2v2_JT', slots: 4 },
        { id: 'monsoon-3v3', label: 'Monsoon (3v3)', path: 'Levels/MP/PVP/DM_Monsoon_3v3_MO/DM_Monsoon_3v3_MO', slots: 6 }
    ];

    var COLORS = ['#3a7bd5', '#2aa8a0', '#3fbf4a', '#e6b322', '#e67e22', '#c0392b', '#ececec', '#8e44ad'];
    var DIFFS = ['EASY', 'MEDIUM', 'HARD'];
    var FACTIONS = [
        { code: 'APA', label: 'APA' },
        { code: 'EU', label: 'EU' },
        { code: 'GLA', label: 'GLA' }
    ];
    var GENERALS_BY_FACTION = {
        APA: [
            { id: 2914080600, key: 'APA_ClassicGeneral', label: 'Classic', icon: 'AG_Basic_01.png' },
            { id: 3919700239, key: 'APA_AtomicGeneral', label: 'Atomic', icon: 'AG_Atomic_01.png' },
            { id: 1259669531, key: 'APA_EspionageGeneral', label: 'Espionage', icon: 'AG_Espionage_01.png' },
            { id: 73570905, key: 'APA_FlameGeneral', label: 'Flame', icon: 'AG_Tank_01.png' },
            { id: 1113071128, key: 'APA_RocketGeneral', label: 'Rocket', icon: 'AG_Rocket_01.png' },
            { id: 3042903044, key: 'APA_UrbanGeneral', label: 'Urban', icon: 'AG_UrbanCombat_01.png' },
            { id: 497011786, key: 'APA_TutorialGeneral', label: 'Tutorial', icon: 'AG_Basic_01.png' }
        ],
        EU: [
            { id: 232716472, key: 'EU_ClassicGeneral', label: 'Classic', icon: 'EG_Basic_01.png' },
            { id: 76852445, key: 'EU_AirforceGeneral', label: 'Airforce', icon: 'EG_Airforce_01.png' },
            { id: 162564633, key: 'EU_LaserGeneral', label: 'Laser', icon: 'EG_Railgun_01.png' },
            { id: 1523019091, key: 'EU_RapidAssaultGeneral', label: 'Rapid Assault', icon: 'EG_RapidAssault_01.png' },
            { id: 2373292237, key: 'EU_ReconGeneral', label: 'Recon', icon: 'EG_Recon_01.png' },
            { id: 3943761922, key: 'EU_ScienceGeneral', label: 'Science', icon: 'EG_ExperimentalWeapons_01.png' },
            { id: 3463861546, key: 'EU_TutorialGeneral', label: 'Tutorial', icon: 'EG_Basic_01.png' }
        ],
        GLA: [
            { id: 580378690, key: 'GLA_ClassicGeneral', label: 'Classic', icon: 'GG_Basic_01.png' },
            { id: 1632710820, key: 'GLA_DemolitionGeneral', label: 'Demolition', icon: 'GG_Demolition_01.png' },
            { id: 1975634889, key: 'GLA_EngineerGeneral', label: 'Engineer', icon: 'GG_Engineer_01.png' },
            { id: 2905920036, key: 'GLA_JunkyardGeneral', label: 'Junkyard', icon: 'GG_Salvage_01.png' },
            { id: 2837579969, key: 'GLA_MarauderGeneral', label: 'Marauder', icon: 'GG_Marauder_01.png' },
            { id: 901813698, key: 'GLA_ToxinGeneral', label: 'Toxin', icon: 'GG_Toxin_01.png' },
            { id: 145977592, key: 'GLA_TutorialGeneral', label: 'Tutorial', icon: 'GG_Basic_01.png' }
        ]
    };
    var GENERAL_ICON_DIR = 'view/image/generals/';
    var FACTION_ICON = {
        APA: 'view/image/Factionlogos/APA_sm.png',
        EU: 'view/image/Factionlogos/EU_sm.png',
        GLA: 'view/image/Factionlogos/GLA_sm.png'
    };
    var DEFAULT_AVATAR = 'view/image/default_profile_img.png';
    var CODENAME_FALLBACK = { APA: 'TACTICIAN', EU: 'CLEAVER', GLA: 'GHOST' };
    var FACTION_BASIC_ICON = {
        APA: 'AG_Basic_01.png',
        EU: 'EG_Basic_01.png',
        GLA: 'GG_Basic_01.png'
    };

    function normalizeFaction(code) {
        var c = String(code || 'APA').toUpperCase();
        if (c === 'ESC') {
            return 'EU';
        }
        return c;
    }

    function generalsForFaction(code) {
        return GENERALS_BY_FACTION[normalizeFaction(code)] || [];
    }

    function tutorialGeneralId(code) {
        var list = generalsForFaction(code);
        var i;
        for (i = 0; i < list.length; i++) {
            if (/Tutorial/i.test(list[i].key) && list[i].id) {
                return list[i].id;
            }
        }
        return 0;
    }

    function classicGeneralId(code) {
        var list = generalsForFaction(code);
        var i;
        for (i = 0; i < list.length; i++) {
            if (/Classic/i.test(list[i].key) && list[i].id) {
                return list[i].id;
            }
        }
        return list.length ? list[0].id : 0;
    }

    function defaultGeneralId(code, map) {
        if (map && map.forceTutorialGeneral) {
            return tutorialGeneralId(code) || classicGeneralId(code);
        }
        return classicGeneralId(code);
    }

    function generalLabel(code, generalId) {
        var id = Number(generalId) || 0;
        var list = generalsForFaction(code);
        var i;
        for (i = 0; i < list.length; i++) {
            if (list[i].id === id) {
                return list[i].label;
            }
        }
        return 'GENERAL';
    }

    function generalIconFile(code, generalId) {
        var id = Number(generalId) || 0;
        var faction = normalizeFaction(code);
        var list = generalsForFaction(faction);
        var i;
        for (i = 0; i < list.length; i++) {
            if (list[i].id === id && list[i].icon) {
                return list[i].icon;
            }
        }
        return FACTION_BASIC_ICON[faction] || null;
    }

    function avatarForSlot(slot) {
        if (!slot || !slot.occupied) {
            return DEFAULT_AVATAR;
        }
        if (slot.invitePending) {
            return DEFAULT_AVATAR;
        }
        var file = generalIconFile(slot.faction, slot.general);
        return file ? (GENERAL_ICON_DIR + file) : DEFAULT_AVATAR;
    }

    function codenameForSlot(slot) {
        if (!slot) {
            return 'GENERAL';
        }
        var label = generalLabel(slot.faction, slot.general);
        if (label && label !== 'GENERAL') {
            return String(label).toUpperCase();
        }
        return CODENAME_FALLBACK[normalizeFaction(slot.faction)] || 'GENERAL';
    }

    function difficultyAttrValue(diff) {
        var map = { EASY: '0', MEDIUM: '1', HARD: '2' };
        return map[String(diff || 'HARD').toUpperCase()] || '2';
    }

    function emptySlot() {
        return {
            occupied: false,
            isLocal: false,
            isAi: false,
            invitePending: false,
            codename: '',
            displayName: '',
            faction: 'APA',
            general: 0,
            difficulty: 'HARD',
            color: COLORS[0],
            avatar: DEFAULT_AVATAR,
            teamNum: 1,
            startpoint: 1,
            pid: 0
        };
    }

    function fillLocalSlot(name) {
        var s = emptySlot();
        s.occupied = true;
        s.isLocal = true;
        s.faction = 'APA';
        s.general = classicGeneralId('APA');
        s.codename = codenameForSlot(s);
        s.displayName = name || 'UnknownPlayer';
        s.teamNum = 1;
        s.startpoint = 1;
        return s;
    }

    function makeTeam(size, teamNum) {
        var t = [];
        var i;
        for (i = 0; i < size; i++) {
            var s = emptySlot();
            s.teamNum = teamNum;
            s.startpoint = teamNum === 1 ? (i + 1) : (i + 2);
            t.push(s);
        }
        return t;
    }

    function httpRequest(method, url) {
        return {
            then: function (resolve) {
                try {
                    if (window.jQuery && jQuery.ajax) {
                        jQuery.ajax({
                            url: url,
                            type: method,
                            dataType: 'json',
                            timeout: 8000,
                            success: function (body) { resolve(body); },
                            error: function () { resolve(false); }
                        });
                        return;
                    }
                } catch (e) { /* fall through */ }
                try {
                    var xhr = new XMLHttpRequest();
                    xhr.open(method, url, true);
                    xhr.onreadystatechange = function () {
                        if (xhr.readyState !== 4) {
                            return;
                        }
                        if (xhr.status < 200 || xhr.status >= 300) {
                            resolve(false);
                            return;
                        }
                        try {
                            resolve(window.JSON ? JSON.parse(xhr.responseText) : true);
                        } catch (pe) {
                            resolve(true);
                        }
                    };
                    xhr.send(null);
                } catch (e2) {
                    resolve(false);
                }
            }
        };
    }

    function whenAll(tasks, onOk, onFail) {
        var left = tasks.length;
        var failed = false;
        if (!left) {
            onOk();
            return;
        }
        function done(ok) {
            if (!ok) {
                failed = true;
            }
            left -= 1;
            if (left > 0) {
                return;
            }
            if (failed && onFail) {
                onFail();
            } else {
                onOk();
            }
        }
        var i;
        for (i = 0; i < tasks.length; i++) {
            tasks[i].then(done);
        }
    }

    CCApp.controller('ShellLobbyController', function ($scope, $rootScope, $timeout) {
        $scope.maps = MAPS;
        $scope.selectedMap = MAPS[1];
        $scope.mapPath = $scope.selectedMap.path;
        $scope.mapForcesTutorial = !!$scope.selectedMap.forceTutorialGeneral;
        $scope.mapMenuOpen = false;
        $scope.lobbySubTab = 'GENERALS';
        $scope.colors = COLORS;
        $scope.diffs = DIFFS;
        $scope.factions = FACTIONS;
        $scope.slotMenu = null;
        $scope.gameId = '1';
        $scope._starting = false;
        $scope.team1 = makeTeam(3, 1);
        $scope.team2 = makeTeam(3, 2);
        $scope.team1[0] = fillLocalSlot($rootScope.playerName);

        $scope.factionIcon = function (slot) {
            if (!slot || !slot.faction) {
                return FACTION_ICON.APA;
            }
            return FACTION_ICON[normalizeFaction(slot.faction)] || FACTION_ICON.APA;
        };

        $scope.generalAvatar = function (slot) {
            return avatarForSlot(slot);
        };

        $scope.slotMenuKey = function (team, index) {
            return String(team) + ':' + String(index);
        };

        $scope.availableFactions = function () {
            if ($scope.mapForcesTutorial && $scope.selectedMap && $scope.selectedMap.forceFaction) {
                var forced = normalizeFaction($scope.selectedMap.forceFaction);
                var out = [];
                var i;
                for (i = 0; i < FACTIONS.length; i++) {
                    if (normalizeFaction(FACTIONS[i].code) === forced) {
                        out.push(FACTIONS[i]);
                    }
                }
                return out;
            }
            return FACTIONS;
        };

        $scope.availableGenerals = function (slot) {
            if (!slot) {
                return [];
            }
            var list = generalsForFaction(slot.faction);
            if (!$scope.mapForcesTutorial) {
                return list;
            }
            var tid = tutorialGeneralId(slot.faction);
            var out = [];
            var i;
            for (i = 0; i < list.length; i++) {
                if (list[i].id === tid) {
                    out.push(list[i]);
                }
            }
            return out.length ? out : list;
        };

        $scope.canChangeGeneral = function () {
            return !$scope.mapForcesTutorial;
        };

        $scope.canChangeFaction = function () {
            if (!$scope.mapForcesTutorial) {
                return true;
            }
            return $scope.availableFactions().length > 1;
        };

        function applyTutorialConstraints() {
            if (!$scope.mapForcesTutorial) {
                return;
            }
            var forcedFaction = $scope.selectedMap && $scope.selectedMap.forceFaction
                ? normalizeFaction($scope.selectedMap.forceFaction)
                : 'EU';
            var teams = [$scope.team1, $scope.team2];
            var t;
            var i;
            for (t = 0; t < teams.length; t++) {
                for (i = 0; i < teams[t].length; i++) {
                    var slot = teams[t][i];
                    if (!slot || !slot.occupied || slot.invitePending) {
                        continue;
                    }
                    slot.faction = forcedFaction;
                    slot.general = tutorialGeneralId(forcedFaction) || classicGeneralId(forcedFaction);
                    slot.codename = codenameForSlot(slot);
                }
            }
        }

        $scope.selectMap = function (map) {
            if (!map) {
                return;
            }
            $scope.selectedMap = map;
            $scope.mapPath = map.path || '';
            $scope.mapForcesTutorial = !!map.forceTutorialGeneral;
            $scope.mapMenuOpen = false;
            $scope.lobbySubTab = 'GENERALS';
            applyTutorialConstraints();
            syncMapToServer();
        };

        $scope.toggleMapMenu = function ($event) {
            if ($event) {
                if ($event.stopPropagation) {
                    $event.stopPropagation();
                }
                if ($event.preventDefault) {
                    $event.preventDefault();
                }
            }
            $scope.mapMenuOpen = !$scope.mapMenuOpen;
            $scope.slotMenu = null;
        };

        $scope.closeMenus = function () {
            $scope.mapMenuOpen = false;
            $scope.slotMenu = null;
        };
        $scope.closeMapMenu = $scope.closeMenus;

        $scope.setLobbySubTab = function (tab) {
            $scope.lobbySubTab = tab;
            $scope.slotMenu = null;
            $scope.mapMenuOpen = (tab === 'MAPS');
        };

        $scope.toggleSlotMenu = function (team, index, $event) {
            if ($event) {
                if ($event.stopPropagation) {
                    $event.stopPropagation();
                }
                if ($event.preventDefault) {
                    $event.preventDefault();
                }
            }
            var slots = team === 2 ? $scope.team2 : $scope.team1;
            var slot = slots[index];
            if (!slot || !slot.occupied || slot.invitePending) {
                return;
            }
            var key = $scope.slotMenuKey(team, index);
            $scope.slotMenu = ($scope.slotMenu === key) ? null : key;
            $scope.mapMenuOpen = false;
        };

        $scope.setFaction = function (slot, code, $event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            if (!slot || slot.invitePending || !$scope.canChangeFaction()) {
                return;
            }
            if ($scope.mapForcesTutorial && $scope.selectedMap && $scope.selectedMap.forceFaction) {
                code = $scope.selectedMap.forceFaction;
            }
            slot.faction = normalizeFaction(code);
            slot.general = defaultGeneralId(slot.faction, $scope.selectedMap);
            slot.codename = codenameForSlot(slot);
        };

        $scope.setGeneral = function (slot, generalId, $event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            if (!slot || slot.invitePending || !$scope.canChangeGeneral()) {
                return;
            }
            slot.general = Number(generalId) || 0;
            slot.codename = codenameForSlot(slot);
            $scope.slotMenu = null;
        };

        $scope.setColor = function (slot, color, $event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            if (slot && slot.occupied && !slot.invitePending) {
                slot.color = color;
            }
        };

        $scope.addAi = function (team) {
            var slots = team === 2 ? $scope.team2 : $scope.team1;
            var i;
            for (i = 0; i < slots.length; i++) {
                if (!slots[i].occupied) {
                    var faction = team === 2 ? 'EU' : 'GLA';
                    if ($scope.mapForcesTutorial && $scope.selectedMap && $scope.selectedMap.forceFaction) {
                        faction = normalizeFaction($scope.selectedMap.forceFaction);
                    }
                    var slot = emptySlot();
                    slot.occupied = true;
                    slot.isAi = true;
                    slot.faction = faction;
                    slot.general = defaultGeneralId(faction, $scope.selectedMap);
                    slot.codename = codenameForSlot(slot);
                    slot.displayName = 'AI';
                    slot.difficulty = 'HARD';
                    slot.color = COLORS[(i + 2) % COLORS.length];
                    slot.teamNum = team;
                    slot.startpoint = team === 1 ? (i + 1) : (i + 2);
                    slots[i] = slot;
                    return;
                }
            }
        };

        $scope.inviteFriend = function (team) {
            var slots = team === 2 ? $scope.team2 : $scope.team1;
            var i;
            for (i = 0; i < slots.length; i++) {
                if (!slots[i].occupied) {
                    var slot = emptySlot();
                    slot.occupied = true;
                    slot.invitePending = true;
                    slot.codename = 'PENDING INVITE...';
                    slot.displayName = 'Invite sent';
                    slot.faction = 'EU';
                    slot.teamNum = team;
                    slots[i] = slot;
                    return;
                }
            }
        };

        $scope.clearSlot = function (team, index, $event) {
            if ($event) {
                if ($event.stopPropagation) {
                    $event.stopPropagation();
                }
                if ($event.preventDefault) {
                    $event.preventDefault();
                }
            }
            var slots = team === 2 ? $scope.team2 : $scope.team1;
            var slot = slots[index];
            if (!slot || slot.isLocal) {
                return;
            }
            var cleared = emptySlot();
            cleared.teamNum = team;
            cleared.startpoint = team === 1 ? (index + 1) : (index + 2);
            slots.splice(index, 1, cleared);
            $scope.slotMenu = null;
        };

        $scope.setDifficulty = function (slot, diff) {
            if (slot && slot.isAi) {
                slot.difficulty = diff;
            }
        };

        $scope.teamHasEmpty = function (team) {
            var slots = team === 2 ? $scope.team2 : $scope.team1;
            var i;
            for (i = 0; i < slots.length; i++) {
                if (!slots[i].occupied) {
                    return true;
                }
            }
            return false;
        };

        function syncMapToServer() {
            var level = $scope.mapPath || '';
            var url = '/cnc/select-map?gid=' + encodeURIComponent($scope.gameId) +
                '&path=' + encodeURIComponent(level);
            return httpRequest('POST', url);
        }

        function syncPlayerAttrsToServer(slot) {
            if (!slot || !slot.occupied || slot.invitePending) {
                return httpRequest('GET', '/cnc/online-count');
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
            q += slot.isAi ? '&isai=1' : '&isai=0';
            if (slot.difficulty) {
                q += '&difficulty=' + encodeURIComponent(difficultyAttrValue(slot.difficulty));
            }
            return httpRequest('POST', q);
        }

        function aiSlots() {
            var out = [];
            var i;
            for (i = 0; i < $scope.team2.length; i++) {
                if ($scope.team2[i].occupied && $scope.team2[i].isAi) {
                    out.push($scope.team2[i]);
                }
            }
            for (i = 0; i < $scope.team1.length; i++) {
                if ($scope.team1[i].occupied && $scope.team1[i].isAi) {
                    out.push($scope.team1[i]);
                }
            }
            return out;
        }

        $scope.startBattle = function () {
            if (!window.CncProbe || !CncProbe.runGame) {
                if (window.console && console.log) {
                    console.log('[lobby] Start Battle needs in-game shell / CncProbe');
                }
                return;
            }
            if ($scope._starting) {
                return;
            }
            $scope._starting = true;
            applyTutorialConstraints();

            var host = $scope.team1[0];
            var gname = (host && host.displayName) || 'Player1';
            var ais = aiSlots();
            var capacity = Math.max(2, 1 + ais.length);
            var level = $scope.mapPath || MAPS[0].path;

            if (CncProbe.markBlazeCreatePending) {
                CncProbe.markBlazeCreatePending();
            }

            var pending = [syncMapToServer()];
            if (host) {
                pending.push(syncPlayerAttrsToServer(host));
            }
            ais.forEach(function (slot) {
                pending.push(syncPlayerAttrsToServer(slot));
            });

            whenAll(pending, beginCreate, beginCreate);

            function beginCreate() {
                if (CncProbe.log) {
                    CncProbe.log('Lobby START: shell createGame ' + gname + ' players=' + capacity +
                        ' level=' + level + ' faction=' + (host && host.faction) +
                        ' general=' + (host && host.general));
                }
                CncProbe.runBlazeUrl(CncProbe.blazeUrlFromResource('creategame', {
                    gameName: gname,
                    players: capacity,
                    level: level
                }));
                CncProbe._inBlazeGame = true;

                $timeout(function () {
                    if (host) {
                        syncPlayerAttrsToServer(host);
                    }
                    ais.forEach(function (slot, idx) {
                        $timeout(function () {
                            if (CncProbe.runAddRemotePlayer) {
                                CncProbe.runAddRemotePlayer(slot.teamNum || 2, slot.startpoint || 2, {
                                    gameId: $scope.gameId,
                                    pollDelayMs: 600
                                });
                            } else {
                                CncProbe.runGame('RtsClient.AddRemotePlayer ' + (slot.teamNum || 2) + ' ' + (slot.startpoint || 2));
                            }
                            $timeout(function () {
                                syncPlayerAttrsToServer(slot);
                            }, 1200);
                        }, 300 * idx);
                    });
                }, 2500);

                var startDelay = 3500 + (ais.length * 300) + 1400;
                $timeout(function () {
                    if (CncProbe.log) {
                        CncProbe.log('Lobby START: RtsClient.StartGame');
                    }
                    CncProbe.runGame('RtsClient.StartGame');
                    $scope._starting = false;
                }, startDelay);
            }
        };

        $scope.exitLobby = function () {
            if ($rootScope.exitLobby) {
                $rootScope.exitLobby();
            }
        };

        $scope.openServerBrowser = function () {
            if ($rootScope.openServerBrowser) {
                $rootScope.openServerBrowser();
            }
        };

        $scope.closeServerBrowser = function () {
            if ($rootScope.closeServerBrowser) {
                $rootScope.closeServerBrowser();
            }
        };

        $scope.$watch(function () {
            return $rootScope.playerName;
        }, function (name) {
            if ($scope.team1[0] && $scope.team1[0].isLocal) {
                $scope.team1[0].displayName = name || 'UnknownPlayer';
            }
        });
    });
})(window, window.angular);
