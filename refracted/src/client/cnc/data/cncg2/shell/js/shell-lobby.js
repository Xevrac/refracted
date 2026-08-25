/**
 * Shell lobby — Classic / Aurora match setup + Blaze start wiring.
 * Tutorial map rules + CreateGame path borrowed from lobby-test.js.
 */
(function (window, angular) {
    'use strict';

    if (!angular || !window.CCApp) {
        return;
    }

    var MINIMAP_DIR = '/cnc/utfwin/images/MiniMap/';
    var LOBBY_MINIMAP_DIR = '/cncg2/shell/view/image/lobby-minimaps/';
    var MAP_THUMB = MINIMAP_DIR + 'MissingMap.png';
    // UTFWin MiniMaps are 512²; lobby PNGs crop the playable island. Pip UV uses map.world.
    var WORLD_XZ = { minX: -512, maxX: 512, minZ: -512, maxZ: 512 };
    var LOCAL_MINIMAPS = {
        'Alpha_Tutorial.png': 1,
        'DM_KapuKai_1v1_JKS.png': 1,
        'DM_Monsoon_3v3_MO.png': 1,
        'DM_Oasis_2v2_JT.png': 1,
        'DM_Overpass_3v3_JKS.png': 1,
        'DM_Smalltown_1v1_CR.png': 1,
        'FirstPlayable_MPHorde_Final.png': 1
    };

    function minimapUrl(file) {
        return MINIMAP_DIR + file;
    }

    function lobbyMinimapUrl(file) {
        if (file && LOCAL_MINIMAPS[file]) {
            return LOBBY_MINIMAP_DIR + file + '?v=3';
        }
        return minimapUrl(file);
    }

    var MAPS = [
        {
            id: 'Alpha_Tutorial',
            path: 'Levels/SP/Alpha_Tutorial/Alpha_Tutorial',
            label: 'Alpha Tutorial',
            slots: 1,
            startCount: 1,
            mode: 'TUTORIAL',
            minimap: 'Alpha_Tutorial.png',
            image: minimapUrl('Alpha_Tutorial.png'),
            accent: '#5a8f3a',
            forceTutorialGeneral: true,
            forceFaction: 'EU',
            texCrop: { u0: 0.086, v0: 0.210, u1: 0.914, v1: 0.790 },
            world: { minX: -423.9, maxX: 423.9, minZ: -297.0, maxZ: 297.0 },
            // StartPointID_01. SP02 unused.
            starts: [
                { id: 1, x: 307.116, z: 197.373, u: 0.961, v: 0.927 }
            ]
        },
        {
            id: 'smalltown',
            label: 'Smalltown',
            path: 'Levels/MP/PVP/DM_Smalltown_1v1_CR/DM_Smalltown_1v1_CR',
            slots: 2,
            startCount: 2,
            mode: '1v1',
            minimap: 'DM_Smalltown_1v1_CR.png',
            image: minimapUrl('DM_Smalltown_1v1_CR.png'),
            accent: '#c07828',
            texCrop: { u0: 0.217, v0: 0.084, u1: 0.799, v1: 0.910 },
            world: { minX: -289.8, maxX: 306.2, minZ: -419.8, maxZ: 426.0 },
            // u/v = pip locations (not worldToUv).
            starts: [
                { id: 1, x: -179.934, z: 315.258, u: 0.175, v: 0.859 },
                { id: 2, x: 179.934, z: -309.879, u: 0.829, v: 0.131 }
            ]
        },
        {
            id: 'kapukai-1v1',
            label: 'Kapu Kai',
            path: 'Levels/MP/PVP/DM_KapuKai_1v1_JKS/DM_KapuKai_1v1_JKS',
            slots: 2,
            startCount: 2,
            mode: '1v1',
            minimap: 'DM_KapuKai_1v1_JKS.png',
            image: minimapUrl('DM_KapuKai_1v1_JKS.png'),
            accent: '#2a7ab8',
            texCrop: { u0: 0.318, v0: 0.299, u1: 0.693, v1: 0.703 },
            world: { minX: -379.7, maxX: 379.7, minZ: -379.7, maxZ: 379.7 },
            // id/x/z = StartPointID. u/v follow design corners on the lobby crop.
            starts: [
                { id: 1, x: -276.257, z: 321.746, u: 0.126, v: 0.911 },
                { id: 2, x: 276.257, z: -321.746, u: 0.841, v: 0.098 }
            ]
        },
        {
            id: 'oasis-2v2',
            label: 'Oasis',
            path: 'Levels/MP/PVP/DM_Oasis_2v2_JT/DM_Oasis_2v2_JT',
            slots: 4,
            startCount: 4,
            mode: '2v2',
            minimap: 'DM_Oasis_2v2_JT.png',
            image: minimapUrl('DM_Oasis_2v2_JT.png'),
            accent: '#b83a3a',
            texCrop: { u0: 0.311, v0: 0.303, u1: 0.689, v1: 0.693 },
            world: { minX: -396.4, maxX: 394.4, minZ: -383.3, maxZ: 407.6 },
            // id/x/z = camera. u/v = worldToUv.
            starts: [
                { id: 1, x: -112.923, z: 347.257, u: 0.358, v: 0.076 },
                { id: 2, x: 93.062, z: -322.937, u: 0.620, v: 0.892 },
                { id: 3, x: 315.061, z: -98.814, u: 0.900, v: 0.609 },
                { id: 4, x: -317.105, z: 109.958, u: 0.100, v: 0.346 }
            ]
        },
        {
            id: 'monsoon-3v3',
            label: 'Monsoon',
            path: 'Levels/MP/PVP/DM_Monsoon_3v3_MO/DM_Monsoon_3v3_MO',
            slots: 6,
            startCount: 6,
            mode: '3v3',
            minimap: 'DM_Monsoon_3v3_MO.png',
            image: minimapUrl('DM_Monsoon_3v3_MO.png'),
            accent: '#6a5ab0',
            texCrop: { u0: 0.219, v0: 0.215, u1: 0.783, v1: 0.762 },
            world: { minX: -447.6, maxX: 452.9, minZ: -448.9, maxZ: 451.6 },
            // id/x/z = StartPointID. u/v swapped on 1↔5 and 2↔6.
            starts: [
                { id: 1, x: -313.067, z: 84.901, u: 0.21, v: 0.59 },
                { id: 2, x: 97.692, z: -380.222, u: 0.61, v: 0.14 },
                { id: 3, x: -258.072, z: 217.144, u: 0.30, v: 0.74 },
                { id: 4, x: 253.280, z: -224.941, u: 0.74, v: 0.32 },
                { id: 5, x: -97.507, z: 382.894, u: 0.42, v: 0.86 },
                { id: 6, x: 318.375, z: -79.735, u: 0.83, v: 0.49 }
            ]
        },
        {
            id: 'overpass-3v3',
            label: 'Overpass',
            path: 'Levels/MP/PVP/DM_Overpass_3v3_JKS/DM_Overpass_3v3_JKS',
            slots: 6,
            startCount: 6,
            mode: '3v3',
            minimap: 'DM_Overpass_3v3_JKS.png',
            image: minimapUrl('DM_Overpass_3v3_JKS.png'),
            accent: '#3a9a8a',
            world: { minX: -453.8, maxX: 448.8, minZ: -443.8, maxZ: 458.8 },
            // u/v = pip locations (not worldToUv). 1↔5 and 2↔6 UVs swapped.
            starts: [
                { id: 1, x: -340, z: 180, u: 0.189, v: 0.670 },
                { id: 2, x: 335, z: -155, u: 0.848, v: 0.334 },
                { id: 3, x: -260, z: 305, u: 0.270, v: 0.779 },
                { id: 4, x: 260, z: -285, u: 0.761, v: 0.225 },
                { id: 5, x: -135, z: 390, u: 0.391, v: 0.848 },
                { id: 6, x: 130, z: -375, u: 0.634, v: 0.147 }
            ]
        },
        {
            // FirstPlayable_MPHorde_Final — player-facing map name is Dreadzone (mode Onslaught).
            id: 'onslaught-fp',
            label: 'Dreadzone',
            path: 'Levels/MP/PVE/FirstPlayable_MPHorde_Final/FirstPlayable_MPHorde_Final',
            slots: 2,
            startCount: 2,
            mode: 'ONSLAUGHT',
            minimap: 'FirstPlayable_MPHorde_Final.png',
            image: minimapUrl('FirstPlayable_MPHorde_Final.png'),
            accent: '#b84a2a',
            world: { minX: -512, maxX: 512, minZ: -512, maxZ: 512 },
            // Human seats only.
            starts: [
                { id: 1, x: -127.090, z: 236.141, u: 0.871, v: 0.139 },
                { id: 2, x: -248.085, z: 118.928, u: 0.129, v: 0.857 }
            ]
        }
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

    function defaultFactionForMap(map) {
        if (map && map.forceFaction) {
            return normalizeFaction(map.forceFaction);
        }
        if (map && map.forceTutorialGeneral) {
            return 'EU';
        }
        return 'APA';
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
            isHost: false,
            ready: false,
            invitePending: false,
            codename: '',
            displayName: '',
            faction: 'APA',
            general: 0,
            difficulty: 'HARD',
            color: COLORS[0],
            avatar: DEFAULT_AVATAR,
            teamNum: 1,
            startpoint: 0,
            pid: 0
        };
    }

    function mapStartCount(map) {
        if (map && map.startCount) {
            return Math.max(1, Number(map.startCount) || 1);
        }
        if (map && Number(map.slots) > 0) {
            return Number(map.slots);
        }
        return 2;
    }

    function startpointIds(map) {
        var out = [];
        var i;
        var seen = {};
        if (map && map.starts && map.starts.length) {
            for (i = 0; i < map.starts.length; i++) {
                var id = Number(map.starts[i].id) || 0;
                if (id > 0 && !seen[id]) {
                    seen[id] = true;
                    out.push(id);
                }
            }
            out.sort(function (a, b) { return a - b; });
            if (out.length) {
                return out;
            }
        }
        var n = mapStartCount(map);
        for (i = 1; i <= n; i++) {
            out.push(i);
        }
        return out;
    }

    function mapWorld(map) {
        if (map && map.world) {
            return map.world;
        }
        return WORLD_XZ;
    }

    function worldToUv(x, z, ext) {
        ext = ext || WORLD_XZ;
        var dx = ext.maxX - ext.minX;
        var dz = ext.maxZ - ext.minZ;
        if (dx < 1 || dz < 1) {
            return { u: 0.5, v: 0.5 };
        }
        return {
            u: (x - ext.minX) / dx,
            v: 1 - ((z - ext.minZ) / dz)
        };
    }

    function lobbyUvForStart(sp, map) {
        if (!sp) {
            return { u: 0.5, v: 0.5 };
        }
        // u/v are pip positions. x/z drive `_startpoint` only.
        if (sp.u != null && sp.v != null) {
            return { u: sp.u, v: sp.v };
        }
        if (sp.x != null && sp.z != null) {
            return worldToUv(sp.x, sp.z, mapWorld(map));
        }
        return { u: 0.5, v: 0.5 };
    }

    function mapStartpoints(map) {
        var n = mapStartCount(map);
        var out;
        var i;
        if (map && map.starts && map.starts.length) {
            return map.starts;
        }
        if (map && map._ringStarts && map._ringStarts.length === n) {
            return map._ringStarts;
        }
        out = [];
        for (i = 0; i < n; i++) {
            var a = (-Math.PI / 2) + ((2 * Math.PI * i) / n);
            out.push({
                id: i + 1,
                u: 0.5 + Math.cos(a) * 0.32,
                v: 0.5 + Math.sin(a) * 0.32
            });
        }
        if (map) {
            map._ringStarts = out;
        }
        return out;
    }

    function fillLocalSlot(name, map) {
        var s = emptySlot();
        s.occupied = true;
        s.isLocal = true;
        s.ready = false;
        s.faction = defaultFactionForMap(map);
        s.general = defaultGeneralId(s.faction, map);
        s.codename = codenameForSlot(s);
        s.displayName = name || 'UnknownPlayer';
        s.teamNum = 1;
        s.startpoint = 0;
        return s;
    }

    function makeTeam(size, teamNum) {
        var t = [];
        var i;
        for (i = 0; i < size; i++) {
            var s = emptySlot();
            s.teamNum = teamNum;
            s.startpoint = 0;
            t.push(s);
        }
        return t;
    }

    function httpRequest(method, url, bodyObj) {
        return {
            then: function (resolve) {
                var payload = null;
                var contentType = null;
                if (bodyObj != null) {
                    try {
                        payload = window.JSON ? JSON.stringify(bodyObj) : null;
                        contentType = 'application/json';
                    } catch (je) {
                        payload = null;
                    }
                }
                try {
                    if (window.jQuery && jQuery.ajax) {
                        jQuery.ajax({
                            url: url,
                            type: method,
                            dataType: 'json',
                            cache: false,
                            data: payload,
                            contentType: contentType || 'application/x-www-form-urlencoded; charset=UTF-8',
                            processData: false,
                            timeout: 8000,
                            success: function (body) { resolve(body); },
                            error: function (xhr) {
                                if (xhr && xhr.responseText) {
                                    try {
                                        resolve(window.JSON ? JSON.parse(xhr.responseText) : false);
                                        return;
                                    } catch (pe) { /* fall through */ }
                                }
                                resolve(false);
                            }
                        });
                        return;
                    }
                } catch (e) { /* fall through */ }
                try {
                    var xhr = new XMLHttpRequest();
                    xhr.open(method, url, true);
                    if (contentType) {
                        xhr.setRequestHeader('Content-Type', contentType);
                    }
                    xhr.onreadystatechange = function () {
                        if (xhr.readyState !== 4) {
                            return;
                        }
                        if (xhr.status < 200 || xhr.status >= 300) {
                            try {
                                resolve(window.JSON ? JSON.parse(xhr.responseText) : false);
                            } catch (pe2) {
                                resolve(false);
                            }
                            return;
                        }
                        try {
                            resolve(window.JSON ? JSON.parse(xhr.responseText) : true);
                        } catch (pe) {
                            resolve(true);
                        }
                    };
                    xhr.send(payload);
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
        $scope.selectedMap = MAPS[0];
        $scope.mapPath = $scope.selectedMap.path;
        $scope.mapForcesTutorial = !!$scope.selectedMap.forceTutorialGeneral;
        $timeout(function () {
            syncMapToServer();
            if ($scope.team1[0]) {
                syncPlayerAttrsToServer($scope.team1[0]);
            }
        }, 0);
        $scope.mapMenuOpen = false;
        $scope.lobbySubTab = 'GENERALS';
        $scope.lobbyOptionsOpen = false;
        $scope.mapPickerOpen = false;
        $scope.mapPickerFocus = null;
        $scope.startModalSlot = null;
        $scope.lobbyChat = [];
        $scope.lobbyChatDraft = '';
        $scope._startpointHoldUntil = 0;
        $scope._startpointResyncAt = 0;
        $scope.lobbyOptions = {
            startingCash: 'standard',
            startingUnits: 'standard',
            noBaseBuilding: false,
            noFogOfWar: false,
            enableSpecialAbilities: true,
            enableTechTree: true,
            enableOilEconomy: false,
            enableInfiniteResourceCenters: false
        };
        $scope.colors = COLORS;
        $scope.diffs = DIFFS;
        $scope.factions = FACTIONS;
        $scope.slotMenu = null;
        $scope.gameId = '1';
        $scope.passwordProtected = false;
        $scope.roomPasswordDraft = '';
        $scope._joinPasswordPrompt = false;
        $scope._joinPasswordValue = '';
        $scope._joinPasswordError = '';
        $scope._joinPasswordTarget = null;
        $scope._starting = false;
        $scope._startError = '';
        $scope._startTimer = null;
        $scope.browserGames = [];
        $scope.selectedBrowserGame = null;
        $scope.browserListFilter = 'all';
        $scope.serverName = '';
        $scope.lobbyAdminPersona = 0;
        $scope._joinedGameroom = false;
        $scope._localIsLobbyHost = false;
        $scope.allHumansReady = false;
        $scope.localReady = false;
        $scope._findingMatch = false;
        $scope._matchError = '';
        $scope.matchmakeMinPlayers = 2;
        $scope.matchmakeMaxPlayers = 8;
        $scope.team1 = makeTeam(3, 1);
        $scope.team2 = makeTeam(3, 2);
        $scope.team1[0] = fillLocalSlot($rootScope.playerName, $scope.selectedMap);

        $scope.factionIcon = function (slot) {
            if (!slot || !slot.faction) {
                return FACTION_ICON.APA;
            }
            return FACTION_ICON[normalizeFaction(slot.faction)] || FACTION_ICON.APA;
        };

        $scope.generalAvatar = function (slot) {
            return avatarForSlot(slot);
        };

        $scope.mapCardStyle = function () {
            return {};
        };

        $scope.lobbyMinimapSrc = function (map) {
            if (!map) {
                return MAP_THUMB;
            }
            if (map.lobbyMinimap) {
                return map.lobbyMinimap;
            }
            if (map.minimap) {
                return lobbyMinimapUrl(map.minimap);
            }
            return map.image || MAP_THUMB;
        };

        $scope.minimapCropStyle = function (map) {
            var fill = {
                width: '100%',
                height: '100%',
                left: '0',
                top: '0'
            };
            if (!map) {
                return fill;
            }
            if (map.minimap && LOCAL_MINIMAPS[map.minimap]) {
                return fill;
            }
            var c = map.texCrop;
            if (!c) {
                return fill;
            }
            var w = c.u1 - c.u0;
            var h = c.v1 - c.v0;
            if (w < 0.05 || h < 0.05) {
                return fill;
            }
            return {
                width: (100 / w).toFixed(2) + '%',
                height: (100 / h).toFixed(2) + '%',
                left: ((-c.u0 / w) * 100).toFixed(2) + '%',
                top: ((-c.v0 / h) * 100).toFixed(2) + '%'
            };
        };

        $scope.mapModeLabel = function (map) {
            if (!map) {
                return '';
            }
            return map.mode || ((map.slots || 2) + 'P');
        };

        $scope.mapPlayerLabel = function (map) {
            var n = map && Number(map.slots);
            if (!n) {
                return '';
            }
            return n === 1 ? '1 player' : (n + ' players');
        };

        $scope.pickerMap = function () {
            return $scope.mapPickerFocus || $scope.selectedMap || MAPS[0];
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

        function eachOccupiedSlot(fn) {
            var teams = [$scope.team1, $scope.team2];
            var t;
            var i;
            for (t = 0; t < teams.length; t++) {
                for (i = 0; i < teams[t].length; i++) {
                    var slot = teams[t][i];
                    if (slot && slot.occupied && !slot.invitePending) {
                        fn(slot);
                    }
                }
            }
        }

        function takenStartpoints(exceptSlot) {
            var used = {};
            eachOccupiedSlot(function (slot) {
                if (slot === exceptSlot) {
                    return;
                }
                var id = parseStartId(slot.startpoint);
                if (id > 0) {
                    used[id] = slot;
                }
            });
            return used;
        }

        function unusedStartpoint(exceptSlot) {
            var free = freeStartpointIds(exceptSlot);
            return free.length ? free[0] : 0;
        }

        function freeStartpointIds(exceptSlot) {
            var used = takenStartpoints(exceptSlot);
            var ids = startpointIds($scope.selectedMap);
            var free = [];
            var i;
            for (i = 0; i < ids.length; i++) {
                if (!used[ids[i]]) {
                    free.push(ids[i]);
                }
            }
            return free;
        }

        function shuffleStartIds(ids) {
            var i;
            for (i = ids.length - 1; i > 0; i--) {
                var j = Math.floor(Math.random() * (i + 1));
                var tmp = ids[i];
                ids[i] = ids[j];
                ids[j] = tmp;
            }
            return ids;
        }

        function randomUnusedStartpoint(exceptSlot) {
            var free = freeStartpointIds(exceptSlot);
            if (!free.length) {
                return 0;
            }
            shuffleStartIds(free);
            return free[0];
        }

        function parseStartId(v) {
            var n = Number(v);
            if (isNaN(n) || n < 0) {
                return 0;
            }
            return n;
        }

        function isValidStartpointForMap(id) {
            var n = parseStartId(id);
            if (n <= 0) {
                return true;
            }
            var ids = startpointIds($scope.selectedMap);
            var i;
            for (i = 0; i < ids.length; i++) {
                if (ids[i] === n) {
                    return true;
                }
            }
            return false;
        }

        function clampStartIdToMap(id) {
            var n = parseStartId(id);
            return isValidStartpointForMap(n) ? n : 0;
        }

        function assignDefaultStartpoints() {
            eachOccupiedSlot(function (slot) {
                if (slot.startpoint == null) {
                    slot.startpoint = 0;
                }
            });
        }

        // Drop picks that are not on the current map.
        function clampStartpointsToMap(syncInvalid) {
            eachOccupiedSlot(function (slot) {
                var id = parseStartId(slot.startpoint);
                if (id > 0 && !isValidStartpointForMap(id)) {
                    slot.startpoint = 0;
                    if (syncInvalid) {
                        syncPlayerAttrsToServer(slot, true);
                    }
                } else if (slot.startpoint == null) {
                    slot.startpoint = 0;
                }
            });
        }

        function resolveExclusiveStartpoints() {
            var used = {};
            var dupes = [];
            eachOccupiedSlot(function (slot) {
                var id = parseStartId(slot.startpoint);
                slot.startpoint = id;
                if (id === 0) {
                    return;
                }
                if (!used[id]) {
                    used[id] = slot;
                } else {
                    dupes.push(slot);
                }
            });
            var i;
            for (i = 0; i < dupes.length; i++) {
                dupes[i].startpoint = unusedStartpoint(dupes[i]);
            }
        }

        function assignRandomStartpoints() {
            var free = shuffleStartIds(freeStartpointIds(null));
            var next = 0;
            eachOccupiedSlot(function (slot) {
                if (parseStartId(slot.startpoint) === 0) {
                    slot.startpoint = (next < free.length)
                        ? free[next++]
                        : (unusedStartpoint(slot) || 1);
                }
            });
        }

        $scope.mapStartpoints = function () {
            return mapStartpoints($scope.selectedMap);
        };

        $scope.pickerMapStartpoints = function () {
            return mapStartpoints($scope.pickerMap());
        };

        $scope.startpointIds = function () {
            return startpointIds($scope.selectedMap);
        };

        function rebuildStartChoices() {
            var opts = [{ id: 0, label: '?' }];
            var ids = startpointIds($scope.selectedMap);
            var i;
            for (i = 0; i < ids.length; i++) {
                opts.push({ id: ids[i], label: String(ids[i]) });
            }
            $scope.startChoices = opts;
            return opts;
        }

        $scope.startpointOptions = function () {
            return $scope.startChoices || rebuildStartChoices();
        };

        $scope.startpointLabel = function (n) {
            var id = parseStartId(n);
            if (id <= 0) {
                return '?';
            }
            return String(id);
        };

        $scope.canChangeStartpoint = function (slot) {
            if (!slot || !slot.occupied || slot.invitePending) {
                return false;
            }
            if ($scope.isLobbyHost && $scope.isLobbyHost()) {
                return true;
            }
            return !!slot.isLocal;
        };

        $scope.setStartpoint = function (slot, id, $event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            if (!$scope.canChangeStartpoint(slot)) {
                return;
            }
            var next = parseStartId(id);
            var max = mapStartCount($scope.selectedMap);
            if (next > max) {
                next = 0;
            }
            if (next > 0 && $scope.startTakenByOther(next, slot)) {
                return;
            }
            slot.startpoint = next;
            // Hold roster until the server confirms; stale init/join POSTs must not overwrite.
            $scope._startpointHoldUntil = Date.now() + 15000;
            syncPlayerAttrsToServer(slot, true);
        };

        $scope.openStartModal = function (slot, $event) {
            if ($event) {
                if ($event.stopPropagation) {
                    $event.stopPropagation();
                }
                if ($event.preventDefault) {
                    $event.preventDefault();
                }
            }
            if (!$scope.canChangeStartpoint(slot)) {
                return;
            }
            $scope.slotMenu = null;
            $scope.mapMenuOpen = false;
            $scope.startModalSlot = slot;
        };

        $scope.closeStartModal = function () {
            $scope.startModalSlot = null;
        };

        $scope.pickStartFromModal = function (id) {
            if (!$scope.startModalSlot) {
                return;
            }
            if ($scope.startTakenByOther(id, $scope.startModalSlot)) {
                return;
            }
            $scope.setStartpoint($scope.startModalSlot, id);
        };

        $scope.startTakenByOther = function (id, slot) {
            var owner = $scope.startpointOwner(id);
            return !!(owner && owner !== slot);
        };

        $scope.startpointOwner = function (id) {
            var want = Number(id) || 0;
            if (want <= 0) {
                return null;
            }
            var found = null;
            eachOccupiedSlot(function (slot) {
                if (!found && Number(slot.startpoint) === want) {
                    found = slot;
                }
            });
            return found;
        };

        $scope.startPipStyle = function (sp, map) {
            if (!sp) {
                return {};
            }
            var uv = lobbyUvForStart(sp, map || $scope.selectedMap);
            var u = Math.max(0.04, Math.min(0.96, uv.u));
            var v = Math.max(0.04, Math.min(0.96, uv.v));
            var style = {
                left: (u * 100).toFixed(2) + '%',
                top: (v * 100).toFixed(2) + '%'
            };
            var owner = $scope.startpointOwner(sp.id);
            if (owner && owner.color) {
                style['background-color'] = owner.color;
                style['border-color'] = '#fff';
            }
            return style;
        };

        $scope.pickerStartPipStyle = function (sp) {
            return $scope.startPipStyle(sp, $scope.pickerMap());
        };

        $scope.startPipLabel = function (sp) {
            return sp ? $scope.startpointLabel(sp.id) : '?';
        };

        $scope.pickStartpointOnMap = function (id, $event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
        };

        function applyTutorialConstraints() {
            if (!$scope.mapForcesTutorial) {
                return;
            }
            var forcedFaction = defaultFactionForMap($scope.selectedMap);
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
                    slot.avatar = avatarForSlot(slot);
                }
            }
        }

        applyTutorialConstraints();
        rebuildStartChoices();
        assignDefaultStartpoints();

        $scope.selectMap = function (map) {
            if (!map) {
                return;
            }
            $scope.selectedMap = map;
            $scope.mapPath = map.path || '';
            $scope.mapForcesTutorial = !!map.forceTutorialGeneral;
            $scope.mapMenuOpen = false;
            $scope.mapPickerOpen = false;
            $scope.lobbySubTab = 'GENERALS';
            if (map.slots) {
                $scope.matchmakeMaxPlayers = Number(map.slots) === 1
                    ? 1
                    : Math.max(2, map.slots);
            }
            applyTutorialConstraints();
            if (Number(map.slots) === 1) {
                clearExtraOccupiedSlotsForSolo();
            }
            rebuildStartChoices();
            assignDefaultStartpoints();
            clampStartpointsToMap(true);
            syncMapToServer();
        };

        function clearExtraOccupiedSlotsForSolo() {
            function wipeExtras(slots, teamNum) {
                var i;
                for (i = 0; i < slots.length; i++) {
                    var s = slots[i];
                    if (!s || !s.occupied || s.isLocal) {
                        continue;
                    }
                    if (s.isAi || s.invitePending) {
                        var cleared = emptySlot();
                        cleared.teamNum = teamNum;
                        cleared.startpoint = 0;
                        slots[i] = cleared;
                    }
                }
            }
            wipeExtras($scope.team1, 1);
            wipeExtras($scope.team2, 2);
        }

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
            $scope.startModalSlot = null;
        };

        $scope.openMapPicker = function ($event) {
            if ($event) {
                if ($event.stopPropagation) {
                    $event.stopPropagation();
                }
                if ($event.preventDefault) {
                    $event.preventDefault();
                }
            }
            if (!$scope.isLobbyHost || !$scope.isLobbyHost()) {
                return;
            }
            $scope.slotMenu = null;
            $scope.mapMenuOpen = false;
            $scope.startModalSlot = null;
            $scope.lobbyOptionsOpen = false;
            $scope.mapPickerFocus = $scope.selectedMap || MAPS[0];
            $scope.mapPickerOpen = true;
        };

        $scope.closeMapPicker = function () {
            $scope.mapPickerOpen = false;
            $scope.mapPickerFocus = null;
        };

        $scope.previewMapInPicker = function (map) {
            if (!map) {
                return;
            }
            $scope.mapPickerFocus = map;
        };

        $scope.confirmMapPicker = function () {
            var map = $scope.mapPickerFocus || $scope.selectedMap;
            if (map) {
                $scope.selectMap(map);
            }
            $scope.mapPickerOpen = false;
            $scope.mapPickerFocus = null;
        };

        $scope.pickMapFromModal = $scope.previewMapInPicker;
        $scope.closeMapMenu = $scope.closeMenus;

        function lobbyGid() {
            var raw = String($scope.gameId == null ? '' : $scope.gameId);
            if (/^\d+$/.test(raw)) {
                return raw;
            }
            var n = parseInt(raw, 10);
            return isNaN(n) ? '1' : String(n);
        }

        function scrollLobbyChat() {
            $timeout(function () {
                var nodes = document.getElementsByClassName('au-lobby__chat-log');
                if (!nodes || !nodes.length) {
                    nodes = document.getElementsByClassName('cc-lobby__chat-log');
                }
                var i;
                for (i = 0; i < nodes.length; i++) {
                    nodes[i].scrollTop = nodes[i].scrollHeight;
                }
            }, 0);
        }

        function applyLobbyChat(data) {
            if (!data || data.ok === false || !data.messages) {
                return;
            }
            if (!data.messages.length) {
                return;
            }
            $scope.lobbyChat = data.messages;
            scrollLobbyChat();
        }

        function pollLobbyChat() {
            var gid = lobbyGid();
            httpRequest('GET', '/cnc/lobby-chat?gid=' + encodeURIComponent(gid)).then(function (data) {
                $timeout(function () {
                    applyLobbyChat(data);
                });
            });
        }

        $scope.sendLobbyChat = function ($event) {
            if ($event) {
                if ($event.preventDefault) {
                    $event.preventDefault();
                }
                if ($event.stopPropagation) {
                    $event.stopPropagation();
                }
            }
            var text = String($scope.lobbyChatDraft || '').replace(/^\s+|\s+$/g, '');
            if (!text) {
                return false;
            }
            $scope.lobbyChatDraft = '';
            var user = ($scope.team1[0] && $scope.team1[0].displayName) ||
                $rootScope.playerName || 'Player';
            $scope.lobbyChat = ($scope.lobbyChat || []).concat([{ user: user, text: text }]);
            scrollLobbyChat();
            var gid = lobbyGid();
            httpRequest('POST', '/cnc/lobby-chat?gid=' + encodeURIComponent(gid), {
                gid: gid,
                user: user,
                text: text
            }).then(function (data) {
                $timeout(function () {
                    applyLobbyChat(data);
                });
            });
            return false;
        };

        $scope.setLobbySubTab = function (tab) {
            $scope.lobbySubTab = tab;
            $scope.slotMenu = null;
            $scope.mapMenuOpen = (tab === 'MAPS');
            if (tab === 'MAPS') {
                $scope.lobbyOptionsOpen = false;
            }
        };

        $scope.openLobbyOptions = function ($event) {
            if ($event) {
                if ($event.stopPropagation) {
                    $event.stopPropagation();
                }
                if ($event.preventDefault) {
                    $event.preventDefault();
                }
            }
            if (!$scope.isLobbyHost || !$scope.isLobbyHost()) {
                return;
            }
            $scope.mapMenuOpen = false;
            $scope.slotMenu = null;
            $scope.startModalSlot = null;
            $scope.mapPickerOpen = false;
            $scope.lobbySubTab = 'OPTIONS';
            $scope.lobbyOptionsOpen = true;
        };

        $scope.closeLobbyOptions = function () {
            $scope.applyLobbyMatchOptions();
            $scope.lobbyOptionsOpen = false;
            if ($scope.lobbySubTab === 'OPTIONS') {
                $scope.lobbySubTab = 'GENERALS';
            }
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
            if ($scope.isSoloMap && $scope.isSoloMap()) {
                return;
            }
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
                    slot.startpoint = unusedStartpoint(slot);
                    ensureAiPersonaId(slot);
                    slots[i] = slot;
                    return;
                }
            }
        };

        $scope.inviteFriend = function (team) {
            if ($scope.isSoloMap && $scope.isSoloMap()) {
                return;
            }
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
            cleared.startpoint = 0;
            slots.splice(index, 1, cleared);
            $scope.slotMenu = null;
        };

        $scope.setDifficulty = function (slot, diff) {
            if (slot && slot.isAi) {
                slot.difficulty = diff;
            }
        };

        function applyBrowserFilter(games) {
            if (!games || !games.length) {
                return [];
            }
            var wantPath = $scope.mapPath || '';
            var wantLeaf = wantPath ? wantPath.split('/').pop() : '';
            var out = [];
            var i;
            for (i = 0; i < games.length; i++) {
                var g = games[i];
                if (!g) {
                    continue;
                }
                if (g.isStandby || g.kind === 'standby' || g.map === 'Standby') {
                    out.push(g);
                    continue;
                }
                if (!wantPath) {
                    out.push(g);
                    continue;
                }
                if (g.mapPath && g.mapPath === wantPath) {
                    out.push(g);
                    continue;
                }
                if (g.map && wantLeaf && String(g.map).toLowerCase() === String(wantLeaf).toLowerCase()) {
                    out.push(g);
                }
            }
            return out.length ? out : games;
        }

        function refreshPingForRows(rows) {
            var i;
            for (i = 0; i < rows.length; i++) {
                (function (row) {
                    if (!row || !row.pingHost) {
                        return;
                    }
                    var port = row.pingPort != null ? row.pingPort : 18387;
                    var url = '/cnc/server-ping?host=' + encodeURIComponent(row.pingHost) +
                        '&port=' + encodeURIComponent(port);
                    httpRequest('GET', url).then(function (data) {
                        if (data && data.pingMs != null) {
                            $timeout(function () {
                                row.pingMs = data.pingMs;
                            });
                        }
                    });
                })(rows[i]);
            }
        }

        $scope.formatPing = function (g) {
            if (!g || g.pingMs == null) {
                return '—';
            }
            return String(g.pingMs) + ' ms';
        };

        $scope.pingTone = function (g) {
            if (!g || g.pingMs == null) {
                return 'ping--unk';
            }
            var ms = Number(g.pingMs);
            if (ms < 50) {
                return 'ping--good';
            }
            if (ms < 120) {
                return 'ping--ok';
            }
            return 'ping--bad';
        };

        function resolveBrowserMap(g) {
            if (!g) {
                return { label: '—', image: MAP_THUMB, mode: '' };
            }
            if (g.isStandby || g.kind === 'standby' || g.map === 'Standby') {
                var phase = (g.phase || '').toString();
                if (/recycl/i.test(phase)) {
                    return { label: 'Standby', image: MAP_THUMB, mode: 'RECYCLING' };
                }
                return { label: 'Standby', image: MAP_THUMB, mode: 'STANDBY' };
            }
            var path = g.mapPath || '';
            var leaf = g.map || (path ? path.split('/').pop() : '');
            var i;
            for (i = 0; i < MAPS.length; i++) {
                if (path && MAPS[i].path === path) {
                    return {
                        label: MAPS[i].label,
                        image: MAP_THUMB,
                        mode: MAPS[i].mode || ''
                    };
                }
                if (leaf && MAPS[i].path && MAPS[i].path.split('/').pop() === leaf) {
                    return {
                        label: MAPS[i].label,
                        image: MAP_THUMB,
                        mode: MAPS[i].mode || ''
                    };
                }
                if (leaf && MAPS[i].label &&
                        String(MAPS[i].label).toLowerCase() === String(leaf).toLowerCase()) {
                    return {
                        label: MAPS[i].label,
                        image: MAP_THUMB,
                        mode: MAPS[i].mode || ''
                    };
                }
            }
            return {
                label: leaf || 'Unknown',
                image: MAP_THUMB,
                mode: g.phase || ''
            };
        }

        function findMapEntryForBrowserGame(g) {
            if (!g || g.isStandby || g.kind === 'standby' || g.map === 'Standby') {
                return null;
            }
            var path = g.mapPath || '';
            var leaf = g.map || (path ? path.split('/').pop() : '');
            var i;
            for (i = 0; i < MAPS.length; i++) {
                if (path && MAPS[i].path === path) {
                    return MAPS[i];
                }
                if (leaf && MAPS[i].path && MAPS[i].path.split('/').pop() === leaf) {
                    return MAPS[i];
                }
                if (leaf && MAPS[i].label &&
                        String(MAPS[i].label).toLowerCase() === String(leaf).toLowerCase()) {
                    return MAPS[i];
                }
            }
            return null;
        }

        function applyMapFromBrowserGame(g) {
            var map = findMapEntryForBrowserGame(g);
            if (map) {
                $scope.selectMap(map);
            }
        }

        $scope.browserMapLabel = function (g) {
            return resolveBrowserMap(g).label;
        };

        $scope.browserMapThumb = function (g) {
            return resolveBrowserMap(g).image;
        };

        $scope.browserMapMode = function (g) {
            return resolveBrowserMap(g).mode || (g && g.phase) || '';
        };

        $scope.setBrowserListFilter = function (filter) {
            $scope.browserListFilter = filter || 'all';
            syncBrowserSelection($scope.visibleBrowserGames());
        };

        $scope.visibleBrowserGames = function () {
            var list = $scope.browserGames || [];
            var filter = $scope.browserListFilter || 'all';
            if (filter === 'all') {
                return list;
            }
            var out = [];
            var i;
            for (i = 0; i < list.length; i++) {
                var g = list[i];
                if (!g) {
                    continue;
                }
                if (filter === 'joinable') {
                    if (g.joinable !== false) {
                        out.push(g);
                    }
                } else if (filter === 'idle') {
                    if (g.isStandby || g.kind === 'standby' || g.map === 'Standby' ||
                            !(Number(g.humans) > 0)) {
                        out.push(g);
                    }
                }
            }
            return out;
        };

        $scope.selectBrowserGame = function (g) {
            $scope.selectedBrowserGame = g || null;
        };

        $scope.isBrowserGameSelected = function (g) {
            if (!$scope.selectedBrowserGame || !g) {
                return false;
            }
            if ($scope.selectedBrowserGame === g) {
                return true;
            }
            return String($scope.selectedBrowserGame.gid) === String(g.gid) &&
                String($scope.selectedBrowserGame.name || '') === String(g.name || '');
        };

        function syncBrowserSelection(list) {
            list = list || [];
            var prev = $scope.selectedBrowserGame;
            var found = null;
            var i;
            if (prev) {
                for (i = 0; i < list.length; i++) {
                    if (String(list[i].gid) === String(prev.gid) &&
                            String(list[i].name || '') === String(prev.name || '')) {
                        found = list[i];
                        break;
                    }
                }
            }
            $scope.selectedBrowserGame = found || (list.length ? list[0] : null);
        }

        function localPersonaId() {
            var pid = ($scope.team1[0] && $scope.team1[0].pid) || 0;
            if (!pid && window.CncBlazeState) {
                if (CncBlazeState.getPersonaId) {
                    pid = CncBlazeState.getPersonaId() || 0;
                } else if (CncBlazeState.personaId != null && CncBlazeState.personaId !== '') {
                    pid = Number(CncBlazeState.personaId) || 0;
                }
            }
            return Number(pid) || 0;
        }

        function localDisplayName() {
            if ($scope.team1[0] && $scope.team1[0].displayName) {
                return String($scope.team1[0].displayName).toLowerCase();
            }
            if (window.CncBlazeState && CncBlazeState.getPlayerName) {
                return String(CncBlazeState.getPlayerName() || '').toLowerCase();
            }
            return '';
        }

        function rosterEntryIsLocal(p, localPid) {
            if (!p || p.isAi) {
                return false;
            }
            if (localPid && Number(p.pid) === Number(localPid)) {
                return true;
            }
            if (!localPid && p.name) {
                var mine = localDisplayName();
                if (mine && String(p.name).toLowerCase() === mine) {
                    return true;
                }
            }
            return false;
        }

        $scope.isLobbyHost = function () {
            if (!$scope._joinedGameroom) {
                return true;
            }
            if ($scope._localIsLobbyHost) {
                return true;
            }
            var localPid = localPersonaId();
            if ($scope.lobbyAdminPersona && localPid) {
                return Number($scope.lobbyAdminPersona) === Number(localPid);
            }
            return !($scope.lobbyAdminPersona > 0);
        };

        $scope.findServers = function () {
            syncMapToServer();
            if ($rootScope.openServerBrowser) {
                $rootScope.openServerBrowser();
            }
        };

        $scope.dismissMatchError = function () {
            $scope._matchError = '';
            $scope._findingMatch = false;
        };

        $scope.dismissServerLostError = function () {
            if ($rootScope.dismissServerLostError) {
                $rootScope.dismissServerLostError();
            }
        };

        function showPostMatchLostModal() {
            if ($rootScope.showServerLostModal) {
                $rootScope.showServerLostModal();
            }
            $scope._starting = false;
            $scope._startError = '';
            $scope._findingMatch = false;
            $scope._matchError = '';
            $scope._joinedGameroom = false;
            stopMatchLostPoll();
            stopRosterPoll();
            try {
                sessionStorage.setItem('cnc_connection_lost', '1');
                sessionStorage.removeItem('cnc_match_gid');
                sessionStorage.removeItem('cnc_match_pid');
            } catch (e) { /* ignore */ }
            if (window.CncProbe) {
                CncProbe._inBlazeGame = false;
                CncProbe._matchWatchArmed = false;
            }
            if ($rootScope.openLobby) {
                $rootScope.openLobby();
            } else {
                $rootScope.lobbyOpen = true;
                $rootScope.lobbyView = 'matchmake';
            }
        }

        function stopMatchLostPoll() {
            if ($scope._matchLostPoll) {
                $timeout.cancel($scope._matchLostPoll);
                $scope._matchLostPoll = null;
            }
        }

        function startMatchLostPoll() {
            stopMatchLostPoll();
            function tick() {
                if (!window.CncProbe || !CncProbe._matchWatchArmed) {
                    return;
                }
                var gid = '0';
                var pid = localPersonaId() || 0;
                try {
                    gid = sessionStorage.getItem('cnc_match_gid') || '0';
                    if (!pid) {
                        pid = Number(sessionStorage.getItem('cnc_match_pid')) || 0;
                    }
                } catch (e) { /* ignore */ }
                httpRequest('GET', '/cnc/match-connection-status?gid=' +
                    encodeURIComponent(gid) + '&pid=' + encodeURIComponent(pid)).then(function (data) {
                    if (data && (data.lost || data.serverLost || data.shellLost)) {
                        showPostMatchLostModal();
                        return;
                    }
                    if (window.CncProbe && CncProbe._matchWatchArmed) {
                        $scope._matchLostPoll = $timeout(tick, 800);
                    }
                });
            }
            tick();
        }

        function forceServerLostKick(message) {
            var inLobby = !!$scope._joinedGameroom;
            var inBlaze = !!(window.CncProbe && CncProbe._inBlazeGame);
            if ($scope._starting) {
                stopRosterPoll();
                if (window.CncProbe && CncProbe.log) {
                    CncProbe.log('Lobby SERVER LOST ignored (starting match): ' +
                        (message || 'Server connection lost.'));
                }
                return;
            }
            if (!inLobby && !inBlaze) {
                return;
            }
            if ($rootScope.showServerLostModal) {
                $rootScope.showServerLostModal(message);
            }
            $scope._starting = false;
            $scope._startError = '';
            $scope._findingMatch = false;
            $scope._matchError = '';
            if (window.CncProbe && CncProbe.log) {
                CncProbe.log('Lobby SERVER LOST: ' + ($rootScope.serverLostError || message));
            }
            if (inLobby) {
                if ($rootScope.leaveGameRoom) {
                    $rootScope.leaveGameRoom();
                } else {
                    postLeaveGameRoom();
                }
            } else if (window.CncProbe) {
                CncProbe._inBlazeGame = false;
                CncProbe._matchWatchArmed = false;
                try {
                    if (CncProbe.clientDisconnect) {
                        CncProbe.clientDisconnect();
                    }
                } catch (e) { /* best-effort */ }
            }
            if ($rootScope.openLobby) {
                $rootScope.openLobby();
            } else if ($rootScope.lobbyOpen === false) {
                $rootScope.lobbyOpen = true;
                $rootScope.lobbyView = 'matchmake';
            } else {
                $rootScope.lobbyView = 'matchmake';
            }
            stopRosterPoll();
        }

        function gameHasRoom(g) {
            if (!g || g.joinable === false || !(g.gid > 0)) {
                return false;
            }
            var humans = Number(g.humans) || 0;
            var max = Number(g.maxPlayers) || 8;
            return humans < max;
        }

        function mapMatchesSelected(g) {
            var wantPath = $scope.mapPath || '';
            if (!wantPath) {
                return true;
            }
            if (g.mapPath && g.mapPath === wantPath) {
                return true;
            }
            var wantLeaf = wantPath.split('/').pop();
            return !!(g.map && wantLeaf &&
                String(g.map).toLowerCase() === String(wantLeaf).toLowerCase());
        }

        function poolAssignableCount(pool) {
            if (!pool) {
                return 0;
            }
            if (typeof pool.assignable === 'number') {
                return pool.assignable;
            }
            if (typeof pool.idle === 'number') {
                return pool.idle;
            }
            return 0;
        }

        function poolStabilizingHint(pool) {
            if (!pool || typeof pool.assignableIn !== 'number') {
                return 0;
            }
            return Math.max(0, pool.assignableIn);
        }

        function pickMatchTarget(games) {
            if (!games || !games.length) {
                return null;
            }
            var joinable = [];
            var i;
            for (i = 0; i < games.length; i++) {
                if (gameHasRoom(games[i])) {
                    joinable.push(games[i]);
                }
            }
            if (!joinable.length) {
                return null;
            }

            function sortByHumansDesc(a, b) {
                return (Number(b.humans) || 0) - (Number(a.humans) || 0);
            }

            var populated = [];
            for (i = 0; i < joinable.length; i++) {
                if ((Number(joinable[i].humans) || 0) > 0) {
                    populated.push(joinable[i]);
                }
            }
            if (populated.length) {
                var populatedMap = [];
                for (i = 0; i < populated.length; i++) {
                    if (mapMatchesSelected(populated[i])) {
                        populatedMap.push(populated[i]);
                    }
                }
                var pool = populatedMap.length ? populatedMap : populated;
                pool.sort(sortByHumansDesc);
                return pool[0];
            }

            var idle = [];
            for (i = 0; i < joinable.length; i++) {
                var g = joinable[i];
                var humans = Number(g.humans) || 0;
                if (humans > 0) {
                    continue;
                }
                if (g.isStandby || g.kind === 'standby' || g.map === 'Standby' || humans === 0) {
                    idle.push(g);
                }
            }
            if (idle.length) {
                return idle[0];
            }
            return null;
        }

        $scope.findMatch = function () {
            if ($scope._findingMatch || $scope._joinedGameroom) {
                return;
            }
            if (!window.CncProbe || !CncProbe.runBlazeUrl) {
                $scope._matchError = 'No idle servers found, check back later.';
                return;
            }
            $scope._findingMatch = true;
            $scope._matchError = '';
            syncMapToServer();

            function failFind(msg) {
                $scope._findingMatch = false;
                $scope._matchError = msg || 'No idle servers found, check back later.';
                if (window.CncProbe && CncProbe.log) {
                    CncProbe.log('Lobby FIND MATCH: ' + $scope._matchError);
                }
            }

            httpRequest('GET', '/cnc/dedicated-pool').then(function (pool) {
                httpRequest('GET', '/cnc/game-list').then(function (data) {
                    $timeout(function () {
                        if (!$scope._findingMatch) {
                            return;
                        }
                        var idlePool = poolAssignableCount(pool);
                        var stabilizingIn = poolStabilizingHint(pool);
                        var games = (data && data.games) ? data.games : [];
                        var target = pickMatchTarget(games);
                        var populated = !!(target && (Number(target.humans) || 0) > 0);

                        if (populated) {
                            if (window.CncProbe && CncProbe.log) {
                                CncProbe.log('Lobby FIND MATCH: joining gid=' + target.gid +
                                    ' humans=' + (target.humans || 0));
                            }
                            $scope._findingMatch = false;
                            $scope.joinBrowserGame(target);
                            return;
                        }

                        if (idlePool <= 0 || !target) {
                            if (stabilizingIn > 0) {
                                failFind('Server is still recycling — try again in a moment.');
                            } else {
                                failFind('No idle servers found, check back later.');
                            }
                            return;
                        }
                        if (window.CncProbe && CncProbe.log) {
                            CncProbe.log('Lobby FIND MATCH: joining gid=' + target.gid +
                                ' humans=' + (target.humans || 0) +
                                ' standby=' + !!(target.isStandby || target.kind === 'standby') +
                                ' idlePool=' + idlePool);
                        }
                        $scope._findingMatch = false;
                        $scope.joinBrowserGame(target);
                    });
                });
            });
        };

        $scope.leaveGameRoom = function () {
            postLeaveGameRoom();
            if ($rootScope.leaveGameRoom) {
                $rootScope.leaveGameRoom();
            }
        };

        $scope.$on('cnc:leaveGameRoom', function () {
            postLeaveGameRoom();
        });

        function postLeaveGameRoom() {
            if (!$scope._joinedGameroom) {
                return;
            }
            var gid = $scope.gameId || '1';
            var localPid = localPersonaId();
            stopRosterPoll();
            var leaveUrl = '/cnc/leave-game?gid=' + encodeURIComponent(gid) +
                '&pid=' + encodeURIComponent(localPid || 0) +
                '&force=1';
            if (window.CncProbe && CncProbe.log) {
                CncProbe.log('Lobby LEAVE: POST ' + leaveUrl);
            }
            httpRequest('POST', leaveUrl).then(function (data) {
                if (window.CncProbe && CncProbe.log) {
                    CncProbe.log('Lobby LEAVE: result ' + (window.JSON ? JSON.stringify(data) : String(data)));
                }
            });
            if (window.CncProbe && CncProbe.runBlazeUrl) {
                try {
                    CncProbe.runBlazeUrl(CncProbe.blazeUrlFromResource('removePlayer', {
                        gameID: gid,
                        playerID: localPid || 0
                    }));
                } catch (e) { /* shell route may not exist yet */ }
            }
            $scope._joinedGameroom = false;
            $scope.lobbyChat = [];
            $scope.lobbyChatDraft = '';
            $scope.startModalSlot = null;
            $scope.mapPickerOpen = false;
            $scope._rosterSawSelf = false;
            $scope._rosterMissSelf = 0;
            $scope.lobbyOptionsOpen = false;
            $scope.serverName = '';
            $scope.lobbyAdminPersona = 0;
            $scope._localIsLobbyHost = false;
            $scope.allHumansReady = false;
            $scope.localReady = false;
            $scope.passwordProtected = false;
            $scope.roomPasswordDraft = '';
            $scope.lobbyOptions.enableSpecialAbilities = true;
            $scope.lobbyOptions.enableTechTree = true;
            $scope.lobbyOptions.enableOilEconomy = false;
            $scope.lobbyOptions.enableInfiniteResourceCenters = false;
            $scope.gameId = '1';
            try {
                sessionStorage.removeItem('cnc_match_gid');
                sessionStorage.removeItem('cnc_match_pid');
            } catch (e) { /* ignore */ }
            if ($scope.team1[0]) {
                $scope.team1[0].isHost = false;
                $scope.team1[0].ready = false;
            }
            clearRemoteHumanSlots();
            if (window.CncProbe) {
                CncProbe._inBlazeGame = false;
                CncProbe._matchWatchArmed = false;
            }
        }

        function clearRemoteHumanSlots() {
            function wipe(slots) {
                var i;
                for (i = 0; i < slots.length; i++) {
                    if (slots[i] && slots[i].occupied && !slots[i].isLocal && !slots[i].isAi) {
                        slots[i] = emptySlot();
                        slots[i].teamNum = slots === $scope.team2 ? 2 : 1;
                        slots[i].startpoint = 0;
                    }
                }
            }
            wipe($scope.team1);
            wipe($scope.team2);
        }

        function placeRemoteHuman(p) {
            if (!p || p.isAi) {
                return;
            }
            if (rosterEntryIsLocal(p, localPersonaId())) {
                return;
            }
            var team = Number(p.team) === 2 ? $scope.team2 : $scope.team1;
            var teamNum = Number(p.team) === 2 ? 2 : 1;
            var i;
            for (i = 0; i < team.length; i++) {
                if (team[i] && team[i].occupied && !team[i].isLocal && !team[i].isAi &&
                        Number(team[i].pid) === Number(p.pid)) {
                    team[i].ready = !!p.ready;
                    team[i].isHost = !!p.isHost;
                    team[i].displayName = p.name || team[i].displayName;
                    if (p.startpoint != null) {
                        team[i].startpoint = clampStartIdToMap(p.startpoint);
                    }
                    return;
                }
            }
            for (i = 0; i < team.length; i++) {
                if (team[i] && !team[i].occupied) {
                    var s = emptySlot();
                    s.occupied = true;
                    s.isLocal = false;
                    s.isAi = false;
                    s.ready = !!p.ready;
                    s.isHost = !!p.isHost;
                    s.pid = p.pid || 0;
                    s.displayName = p.name || ('Player' + (i + 1));
                    s.faction = defaultFactionForMap($scope.selectedMap);
                    s.general = defaultGeneralId(s.faction, $scope.selectedMap);
                    s.codename = codenameForSlot(s);
                    s.avatar = avatarForSlot(s);
                    s.teamNum = teamNum;
                    s.startpoint = (p.startpoint != null)
                        ? clampStartIdToMap(p.startpoint)
                        : 0;
                    team[i] = s;
                    return;
                }
            }
        }

        $scope.toggleReady = function () {
            if (!$scope._joinedGameroom) {
                return;
            }
            var next = !$scope.localReady;
            var gid = $scope.gameId || '1';
            var localPid = localPersonaId();
            httpRequest('POST', '/cnc/player-ready?gid=' + encodeURIComponent(gid) +
                '&pid=' + encodeURIComponent(localPid || 0) +
                '&ready=' + (next ? '1' : '0')).then(function (data) {
                $timeout(function () {
                    if (!data || data.ok === false) {
                        return;
                    }
                    $scope.localReady = !!data.ready;
                    $scope.allHumansReady = !!data.allReady;
                    if ($scope.team1[0] && $scope.team1[0].isLocal) {
                        $scope.team1[0].ready = $scope.localReady;
                    }
                    if (data.admin) {
                        $scope.lobbyAdminPersona = data.admin;
                    }
                });
            });
        };

        function applyRosterFlags(data) {
            if (!data || !data.ok) {
                return;
            }
            if (data.admin != null) {
                $scope.lobbyAdminPersona = data.admin;
            }
            if (data.passwordProtected != null) {
                $scope.passwordProtected = !!data.passwordProtected;
            }
            if (!$scope.lobbyOptionsOpen) {
                if (data.enableSpecialAbilities != null) {
                    $scope.lobbyOptions.enableSpecialAbilities = !!data.enableSpecialAbilities;
                }
                if (data.enableTechTree != null) {
                    $scope.lobbyOptions.enableTechTree = !!data.enableTechTree;
                }
                if (data.enableOilEconomy != null) {
                    $scope.lobbyOptions.enableOilEconomy = false;
                }
                if (data.enableInfiniteResourceCenters != null) {
                    $scope.lobbyOptions.enableInfiniteResourceCenters = false;
                }
            }
            $scope.allHumansReady = !!data.allReady;
            var localPid = localPersonaId();
            var players = data.players || [];
            var localStillIn = false;
            var remotes = [];
            var i;
            for (i = 0; i < players.length; i++) {
                var p = players[i];
                if (!p || p.isAi) {
                    continue;
                }
                if (rosterEntryIsLocal(p, localPid)) {
                    localStillIn = true;
                    $scope._rosterSawSelf = true;
                    if ($scope.team1[0] && $scope.team1[0].isLocal) {
                        $scope.team1[0].isHost = !!p.isHost || $scope._localIsLobbyHost;
                        if (p.pid) {
                            $scope.team1[0].pid = p.pid;
                            localPid = Number(p.pid) || localPid;
                        }
                    }
                    var asHost = !!(p.isHost || $scope._localIsLobbyHost ||
                        ($scope.isLobbyHost && $scope.isLobbyHost()));
                    $scope.localReady = asHost ? true : !!p.ready;
                    if ($scope.team1[0] && $scope.team1[0].isLocal) {
                        $scope.team1[0].ready = $scope.localReady;
                        $scope.team1[0].isHost = asHost || !!p.isHost;
                        if (p.startpoint != null && Date.now() >= ($scope._startpointHoldUntil || 0)) {
                            var serverSp = clampStartIdToMap(p.startpoint);
                            var localSp = parseStartId($scope.team1[0].startpoint);
                            if (!isValidStartpointForMap(localSp)) {
                                localSp = 0;
                                $scope.team1[0].startpoint = 0;
                            }
                            if (serverSp > 0 || localSp <= 0) {
                                $scope.team1[0].startpoint = serverSp;
                            } else if (localSp > 0 && serverSp === 0) {
                                maybeResyncStartpoint($scope.team1[0]);
                            }
                        }
                    }
                } else {
                    remotes.push(p);
                }
            }
            if ($scope._rosterSawSelf && !localStillIn && $scope._joinedGameroom) {
                $scope._rosterMissSelf = ($scope._rosterMissSelf || 0) + 1;
                if ($scope._rosterMissSelf >= 3) {
                    forceServerLostKick('Server connection lost.');
                    return;
                }
            } else if (localStillIn) {
                $scope._rosterMissSelf = 0;
            }
            clearRemoteHumanSlots();
            for (i = 0; i < remotes.length; i++) {
                placeRemoteHuman(remotes[i]);
            }
            if ($scope.team1[0] && $scope.team1[0].isLocal && $scope.lobbyAdminPersona) {
                var pid = $scope.team1[0].pid || localPid;
                $scope.team1[0].isHost = Number(pid) === Number($scope.lobbyAdminPersona) ||
                    (!$scope.team1[0].pid && $scope._localIsLobbyHost);
            }
            if ($scope.team1[0] && $scope.team1[0].isLocal && $scope.isLobbyHost()) {
                $scope.team1[0].ready = true;
                $scope.team1[0].isHost = true;
                $scope.localReady = true;
            }
            if ($scope.isLobbyHost()) {
                var guestBlock = false;
                for (i = 0; i < remotes.length; i++) {
                    if (remotes[i] && !remotes[i].ready && !remotes[i].isHost) {
                        guestBlock = true;
                        break;
                    }
                }
                if (!guestBlock) {
                    $scope.allHumansReady = true;
                }
            }
        }

        function stopRosterPoll() {
            if ($scope._rosterPoll) {
                $timeout.cancel($scope._rosterPoll);
                $scope._rosterPoll = null;
            }
        }

        function pollLobbyRoster() {
            if ($scope._starting || !$scope._joinedGameroom) {
                stopRosterPoll();
                return;
            }
            var gid = $scope.gameId || '1';
            httpRequest('GET', '/cnc/lobby-roster?gid=' + encodeURIComponent(gid)).then(function (data) {
                $timeout(function () {
                    if (data && data.serverLost) {
                        forceServerLostKick(data.message ||
                            'Server connection lost.');
                        return;
                    }
                    if (data && data.ok === false) {
                        if ($scope._joinedGameroom && !$scope._starting) {
                            $scope._rosterPoll = $timeout(pollLobbyRoster, 1500);
                        }
                        return;
                    }
                    if (data && $scope._joinedGameroom) {
                        applyRosterFlags(data);
                        pollLobbyChat();
                    }
                    if ($scope._joinedGameroom && !$scope._starting) {
                        $scope._rosterPoll = $timeout(pollLobbyRoster, 1500);
                    }
                });
            });
        }

        function startRosterPoll() {
            stopRosterPoll();
            pollLobbyRoster();
        }

        $scope.teamHasEmpty = function (team) {
            if ($scope.isSoloMap && $scope.isSoloMap()) {
                return false;
            }
            var slots = team === 2 ? $scope.team2 : $scope.team1;
            var i;
            for (i = 0; i < slots.length; i++) {
                if (!slots[i].occupied) {
                    return true;
                }
            }
            return false;
        };

        $scope.isSoloMap = function () {
            var map = $scope.selectedMap;
            return !!(map && Number(map.slots) === 1);
        };

        $scope.canAddPlayers = function () {
            return !$scope.isSoloMap() && $scope.isLobbyHost();
        };

        $scope.canStartBattle = function () {
            if (!$scope.isLobbyHost()) {
                return false;
            }
            if ($scope.isSoloMap()) {
                return true;
            }
            var remotes = 0;
            eachOccupiedSlot(function (slot) {
                if (slot && slot.occupied && !slot.isAi && !slot.isLocal && !slot.invitePending) {
                    remotes += 1;
                }
            });
            if (remotes === 0) {
                return true;
            }
            return !!$scope.allHumansReady;
        };

        function syncMapToServer() {
            var level = ($scope.selectedMap && $scope.selectedMap.path) || $scope.mapPath ||
                (MAPS[0] && MAPS[0].path) || '';
            $scope.mapPath = level;
            if (!level) {
                return httpRequest('GET', '/cnc/online-count');
            }
            var url = '/cnc/select-map?gid=' + encodeURIComponent($scope.gameId) +
                '&path=' + encodeURIComponent(level) +
                '&startCount=' + encodeURIComponent(mapStartCount($scope.selectedMap));
            if (window.CncProbe && CncProbe.log) {
                CncProbe.log('select-map → gid=' + $scope.gameId + ' path=' + level +
                    ' label=' + (($scope.selectedMap && $scope.selectedMap.label) || ''));
            }
            return httpRequest('POST', url);
        }

        function ensureAiPersonaId(slot) {
            if (!slot || !slot.isAi) {
                return slot && slot.pid ? Number(slot.pid) : 0;
            }
            var pid = Number(slot.pid) || 0;
            if (pid < 0) {
                return pid;
            }
            var sp = parseStartId(slot.startpoint);
            if (!(sp > 0)) {
                sp = unusedStartpoint(slot) || 1;
                slot.startpoint = sp;
            }
            pid = -(1000 + sp);
            slot.pid = pid;
            return pid;
        }

        function maybeResyncStartpoint(slot) {
            if (!slot || parseStartId(slot.startpoint) <= 0) {
                return;
            }
            var now = Date.now();
            if (($scope._startpointResyncAt || 0) > now) {
                return;
            }
            $scope._startpointResyncAt = now + 3000;
            syncPlayerAttrsToServer(slot, true);
        }

        function syncPlayerAttrsToServer(slot, includeStartpoint) {
            if (!slot || !slot.occupied || slot.invitePending) {
                return httpRequest('GET', '/cnc/online-count');
            }
            var pid = slot.isAi ? ensureAiPersonaId(slot) : (slot.pid || 0);
            var q = '/cnc/player-attrs?gid=' + encodeURIComponent($scope.gameId) +
                '&pid=' + encodeURIComponent(pid);
            if (slot.faction) {
                q += '&faction=' + encodeURIComponent(slot.faction);
            }
            if (slot.teamNum != null) {
                q += '&team=' + encodeURIComponent(slot.teamNum);
            }
            var sp = parseStartId(slot.startpoint);
            if (includeStartpoint || sp > 0) {
                q += '&startpoint=' + encodeURIComponent(sp);
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
            if ($scope._joinedGameroom && !$scope.isLobbyHost()) {
                return;
            }
            if ($scope._joinedGameroom && !$scope.canStartBattle()) {
                $scope._startError = 'All players must be ready before starting.';
                return;
            }
            if ($scope._starting) {
                return;
            }
            $scope.closeLobbyOptions();
            $scope._starting = true;
            $scope._startError = '';
            if ($rootScope.serverLostError) {
                $rootScope.serverLostError = '';
            }
            try {
                sessionStorage.removeItem('cnc_connection_lost');
            } catch (e) { /* ignore */ }
            stopRosterPoll();
            $scope._rosterMissSelf = 0;
            var startUnix = Math.floor(Date.now() / 1000);
            applyTutorialConstraints();
            resolveExclusiveStartpoints();
            assignRandomStartpoints();

            var host = $scope.team1[0];
            var gname = (host && host.displayName) || 'Player1';
            var ais = aiSlots();
            var occupied = 0;
            eachOccupiedSlot(function () { occupied += 1; });
            var capacity = Math.max(1, occupied);
            var level = ($scope.selectedMap && $scope.selectedMap.path) ||
                $scope.mapPath || MAPS[0].path;
            $scope.mapPath = level;

            function clearStartTimer() {
                if ($scope._startTimer) {
                    $timeout.cancel($scope._startTimer);
                    $scope._startTimer = null;
                }
                if ($scope._poolPollTimer) {
                    $timeout.cancel($scope._poolPollTimer);
                    $scope._poolPollTimer = null;
                }
            }

            function isBackendCommFail(msg) {
                return !!(msg && String(msg).indexOf('Failed to communicate with the backend') >= 0);
            }

            function failStart(msg) {
                clearStartTimer();
                CncProbe.onLobbyStartResult = null;
                if (CncProbe._pendingBlazeCreate) {
                    CncProbe._pendingBlazeCreate = false;
                }
                CncProbe._inBlazeGame = false;
                CncProbe._matchWatchArmed = false;
                $scope._starting = false;
                // Backend unreachable: leave the room. Soft errors like "no idle" stay.
                if (isBackendCommFail(msg) && $scope._joinedGameroom) {
                    $scope._startError = '';
                    forceServerLostKick(msg);
                    return;
                }
                $scope._startError = msg;
            }

            function pollNoDedicated(attemptsLeft) {
                if (!$scope._starting || $scope._startError) {
                    return;
                }
                httpRequest('GET', '/cnc/dedicated-pool').then(function (data) {
                    $timeout(function () {
                        if (!$scope._starting || $scope._startError) {
                            return;
                        }
                        if (data && data.lastNoDedicatedAt
                            && data.lastNoDedicatedAt >= (startUnix - 1)) {
                            failStart('No idle servers found, check back later.');
                            return;
                        }
                        if (attemptsLeft > 0) {
                            $scope._poolPollTimer = $timeout(function () {
                                pollNoDedicated(attemptsLeft - 1);
                            }, 400);
                        }
                    });
                });
            }

            CncProbe.onLobbyStartResult = function (info) {
                $timeout(function () {
                    if (!$scope._starting) {
                        return;
                    }
                    if (info && info.ok) {
                        clearStartTimer();
                        $scope._starting = false;
                        CncProbe._inBlazeGame = true;
                        CncProbe._matchWatchArmed = true;
                        startMatchLostPoll();
                        $scope._joinedGameroom = false;
                        $rootScope.serverLostError = '';
                        stopRosterPoll();
                        try {
                            sessionStorage.setItem('cnc_match_gid', String($scope.gameId || '1'));
                            var pid = localPersonaId();
                            if (pid) {
                                sessionStorage.setItem('cnc_match_pid', String(pid));
                            }
                        } catch (e) { /* ignore */ }
                        CncProbe.onLobbyStartResult = null;
                        if ($rootScope.exitLobby) {
                            $rootScope.exitLobby();
                        } else {
                            $rootScope.lobbyOpen = false;
                        }
                        pollNoDedicated(12);
                        return;
                    }
                    if (info && info.noIdle) {
                        failStart('No idle servers found, check back later.');
                    } else {
                        failStart('Failed to communicate with the backend, please try again.');
                    }
                });
            };

            $scope._startTimer = $timeout(function () {
                if ($scope._starting && !$scope._startError) {
                    failStart('Failed to communicate with the backend, please try again.');
                }
            }, 150000);

            if (CncProbe.markBlazeCreatePending) {
                CncProbe.markBlazeCreatePending();
            }

            var pending = [syncMapToServer()];
            eachOccupiedSlot(function (slot) {
                pending.push(syncPlayerAttrsToServer(slot, true));
            });

            whenAll(pending, beginCreate, function () {
                failStart('Failed to communicate with the backend, please try again.');
            });

            function beginCreate() {
                waitForAssignable(45, fireCreate);
            }

            function waitForAssignable(attemptsLeft, then) {
                httpRequest('GET', '/cnc/dedicated-pool').then(function (pool) {
                    $timeout(function () {
                        if (!$scope._starting || $scope._startError) {
                            return;
                        }
                        var assignable = poolAssignableCount(pool);
                        if (assignable > 0) {
                            then();
                            return;
                        }
                        var waitHint = poolStabilizingHint(pool);
                        if (attemptsLeft > 0) {
                            var delayMs = waitHint > 0
                                ? Math.min(3000, waitHint * 1000 + 400)
                                : 1000;
                            $scope._poolPollTimer = $timeout(function () {
                                waitForAssignable(attemptsLeft - 1, then);
                            }, delayMs);
                            return;
                        }
                        if (waitHint > 0) {
                            failStart('Server is still recycling — try again in a moment.');
                        } else {
                            failStart('No idle servers found, check back later.');
                        }
                    });
                });
            }

            function fireCreate() {
                if (CncProbe.log) {
                    CncProbe.log('Lobby START: shell createGame ' + gname + ' players=' + capacity +
                        ' level=' + level + ' faction=' + (host && host.faction) +
                        ' general=' + (host && host.general) + ' gid=' + $scope.gameId);
                }
                try {
                    CncProbe.runBlazeUrl(CncProbe.blazeUrlFromResource('creategame', {
                        gameName: gname,
                        players: capacity,
                        level: level,
                        gameID: $scope.gameId
                    }));
                } catch (e) {
                    failStart('Failed to communicate with the backend, please try again.');
                    return;
                }

                pollNoDedicated(20);

                $timeout(function () {
                    if (host) {
                        syncPlayerAttrsToServer(host);
                    }
                    ais.forEach(function (slot, idx) {
                        $timeout(function () {
                            var sp = Number(slot.startpoint);
                            if (!(sp > 0)) {
                                sp = randomUnusedStartpoint(slot) || unusedStartpoint(slot) || 2;
                                slot.startpoint = sp;
                            }
                            if (CncProbe.runAddRemotePlayer) {
                                CncProbe.runAddRemotePlayer(slot.teamNum || 2, sp, {
                                    gameId: $scope.gameId,
                                    pollDelayMs: 600
                                });
                            } else {
                                CncProbe.runGame('RtsClient.AddRemotePlayer ' + (slot.teamNum || 2) + ' ' + sp);
                            }
                            $timeout(function () {
                                syncPlayerAttrsToServer(slot);
                            }, 1200);
                        }, 300 * idx);
                    });
                }, 2500);

                if (CncProbe.log) {
                    CncProbe.log(
                        'Lobby START: waiting for Blaze GameReady / LeaveIngame (no RtsClient.StartGame)');
                }
            }
        };

        $scope.dismissStartError = function () {
            $scope._startError = '';
            $scope._starting = false;
            if ($scope._startTimer) {
                $timeout.cancel($scope._startTimer);
                $scope._startTimer = null;
            }
            if ($scope._poolPollTimer) {
                $timeout.cancel($scope._poolPollTimer);
                $scope._poolPollTimer = null;
            }
            if (window.CncProbe) {
                CncProbe.onLobbyStartResult = null;
            }
        };

        $scope.refreshGameList = function () {
            httpRequest('GET', '/cnc/game-list').then(function (data) {
                $timeout(function () {
                    if (data && data.games) {
                        var filtered = applyBrowserFilter(data.games);
                        $scope.browserGames = filtered;
                        syncBrowserSelection($scope.visibleBrowserGames());
                        refreshPingForRows(filtered);
                        if (window.CncProbe && CncProbe.log) {
                            CncProbe.log('Lobby browser: ' + filtered.length + ' row(s)');
                        }
                    } else if (data === false) {
                        $scope.browserGames = [];
                        $scope.selectedBrowserGame = null;
                        if (window.CncProbe && CncProbe.log) {
                            CncProbe.log('Lobby browser: /cnc/game-list failed');
                        }
                    } else {
                        $scope.browserGames = [];
                        $scope.selectedBrowserGame = null;
                    }
                });
            });
        };

        function stopBrowserPoll() {
            if ($scope._browserPoll) {
                $timeout.cancel($scope._browserPoll);
                $scope._browserPoll = null;
            }
        }

        function startBrowserPoll() {
            stopBrowserPoll();
            $scope.refreshGameList();
            function poll() {
                $scope.refreshGameList();
                $scope._browserPoll = $timeout(poll, 2000);
            }
            $scope._browserPoll = $timeout(poll, 2000);
        }

        $scope.$watch(function () {
            return ($rootScope.lobbyOpen && $rootScope.lobbyView === 'browser') ? 'on' : 'off';
        }, function (state) {
            if (state === 'on') {
                startBrowserPoll();
            } else {
                stopBrowserPoll();
            }
        });

        $scope.$on('$destroy', function () {
            stopBrowserPoll();
            stopRosterPoll();
            stopMatchLostPoll();
            if (unloadBound) {
                try {
                    if (window.removeEventListener) {
                        window.removeEventListener('beforeunload', unloadBound);
                        window.removeEventListener('pagehide', unloadBound);
                    }
                } catch (e) { /* ignore */ }
                unloadBound = null;
            }
        });

        var unloadBound = null;
        function bindLeaveOnUnload() {
            if (unloadBound) {
                return;
            }
            unloadBound = function () {
                if (!$scope._joinedGameroom) {
                    return;
                }
                if ($scope._starting || (window.CncProbe && CncProbe._inBlazeGame)) {
                    return;
                }
                var gid = $scope.gameId || '1';
                var localPid = localPersonaId() || 0;
                var url = '/cnc/leave-game?gid=' + encodeURIComponent(gid) +
                    '&pid=' + encodeURIComponent(localPid) + '&force=1';
                try {
                    if (navigator.sendBeacon) {
                        navigator.sendBeacon(url);
                        return;
                    }
                } catch (e) { /* fall through */ }
                try {
                    var xhr = new XMLHttpRequest();
                    xhr.open('POST', url, false);
                    xhr.send(null);
                } catch (e2) { /* ignore */ }
            };
            try {
                if (window.addEventListener) {
                    window.addEventListener('beforeunload', unloadBound);
                    window.addEventListener('pagehide', unloadBound);
                }
            } catch (e3) { /* ignore */ }
        }
        bindLeaveOnUnload();

        function restoreLostModalIfNeeded() {
            if ($rootScope.serverLostError) {
                return;
            }
            var pid = localPersonaId() || 0;
            httpRequest('GET', '/cnc/match-connection-status?gid=0&pid=' +
                encodeURIComponent(pid)).then(function (data) {
                if (data && (data.lost || data.serverLost || data.shellLost || data.clientLost)) {
                    if ($rootScope.showServerLostModal) {
                        $rootScope.showServerLostModal();
                    }
                    return;
                }
                if (pid > 0) {
                    try {
                        sessionStorage.removeItem('cnc_connection_lost');
                    } catch (e) { /* ignore */ }
                }
            });
        }
        restoreLostModalIfNeeded();

        function doJoinBrowserGame(g) {
            if (!g || g.joinable === false) {
                return;
            }
            if (!window.CncProbe || !CncProbe.runBlazeUrl) {
                return;
            }
            var gid = g.gid != null ? g.gid : 1;
            var localPid = localPersonaId();
            $scope.gameId = String(gid);
            $scope.serverName = g.name || ('Game ' + gid);
            $scope.lobbyAdminPersona = g.admin || 0;
            $scope.passwordProtected = !!g.passwordProtected;
            $scope._localIsLobbyHost = (!g.humans || g.humans === 0) ||
                !(g.admin > 0) ||
                (localPid > 0 && Number(g.admin) === Number(localPid));
            $scope._joinedGameroom = true;
            $scope.localReady = !!$scope._localIsLobbyHost;
            $scope.allHumansReady = !!$scope._localIsLobbyHost;
            $scope._rosterSawSelf = false;
            $scope._rosterMissSelf = 0;
            try {
                sessionStorage.setItem('cnc_match_gid', String(gid));
                if (localPid) {
                    sessionStorage.setItem('cnc_match_pid', String(localPid));
                }
            } catch (e) { /* ignore */ }
            applyMapFromBrowserGame(g);
            if ($scope.team1[0] && $scope.team1[0].isLocal) {
                $scope.team1[0].ready = !!$scope._localIsLobbyHost;
                $scope.team1[0].isHost = !!$scope._localIsLobbyHost;
                if (window.CncBlazeState && CncBlazeState.getPersonaId) {
                    var blazePid = CncBlazeState.getPersonaId();
                    if (blazePid) {
                        $scope.team1[0].pid = blazePid;
                    }
                }
            }
            syncMapToServer();
            applyTutorialConstraints();
            if ($scope.team1[0]) {
                syncPlayerAttrsToServer($scope.team1[0]);
            }
            if (CncProbe.log) {
                CncProbe.log('Lobby JOIN: game ' + gid + ' server=' + ($scope.serverName) +
                    ' map=' + (($scope.selectedMap && $scope.selectedMap.path) || g.map || '') +
                    ' (Blaze joinGame → GameRoom lobby)');
            }
            CncProbe.runBlazeUrl(CncProbe.blazeUrlFromResource('joingame', {
                gameID: gid
            }));
            CncProbe._inBlazeGame = true;
            if ($rootScope.closeServerBrowser) {
                $rootScope.closeServerBrowser();
            }
            if ($rootScope.openGameRoom) {
                $rootScope.openGameRoom();
            }
            startRosterPoll();
            $timeout(function () {
                httpRequest('GET', '/cnc/game-list').then(function (data) {
                    if (!data || !data.games) {
                        return;
                    }
                    var i;
                    for (i = 0; i < data.games.length; i++) {
                        if (String(data.games[i].gid) === String(gid)) {
                            $timeout(function () {
                                if (data.games[i].admin) {
                                    $scope.lobbyAdminPersona = data.games[i].admin;
                                    if ($scope.team1[0] && $scope.team1[0].isLocal) {
                                        var lp = $scope.team1[0].pid ||
                                            (window.CncBlazeState && CncBlazeState.getPersonaId &&
                                                CncBlazeState.getPersonaId()) || 0;
                                        $scope.team1[0].isHost =
                                            Number(lp) === Number(data.games[i].admin) ||
                                            $scope._localIsLobbyHost;
                                        if ($scope.team1[0].isHost) {
                                            $scope.team1[0].ready = true;
                                            $scope.localReady = true;
                                        }
                                    }
                                }
                                if (data.games[i].passwordProtected != null) {
                                    $scope.passwordProtected = !!data.games[i].passwordProtected;
                                }
                            });
                            break;
                        }
                    }
                });
            }, 1200);
        }

        $scope.joinBrowserGame = function (g) {
            if (!g || g.joinable === false) {
                return;
            }
            var needsPassword = !!g.passwordProtected && (g.humans > 0);
            if (needsPassword) {
                $scope._joinPasswordTarget = g;
                $scope._joinPasswordValue = '';
                $scope._joinPasswordError = '';
                $scope._joinPasswordPrompt = true;
                return;
            }
            doJoinBrowserGame(g);
        };

        $scope.cancelJoinPassword = function () {
            $scope._joinPasswordPrompt = false;
            $scope._joinPasswordValue = '';
            $scope._joinPasswordError = '';
            $scope._joinPasswordTarget = null;
        };

        $scope.confirmJoinPassword = function () {
            var g = $scope._joinPasswordTarget;
            if (!g) {
                $scope.cancelJoinPassword();
                return;
            }
            var gid = g.gid != null ? g.gid : 1;
            var localPid = localPersonaId();
            var pwd = $scope._joinPasswordValue || '';
            httpRequest('POST', '/cnc/verify-game-password?gid=' + encodeURIComponent(gid) +
                '&pid=' + encodeURIComponent(localPid || 0), { password: pwd }).then(function (data) {
                $timeout(function () {
                    if (!data || data.ok === false) {
                        $scope._joinPasswordError = (data && data.error) || 'Wrong password';
                        return;
                    }
                    $scope.cancelJoinPassword();
                    doJoinBrowserGame(g);
                });
            });
        };

        $scope.applyRoomPassword = function () {
            if (!$scope.isLobbyHost || !$scope.isLobbyHost()) {
                return;
            }
            var gid = $scope.gameId || '1';
            var localPid = localPersonaId();
            var pwd = $scope.roomPasswordDraft || '';
            httpRequest('POST', '/cnc/game-password?gid=' + encodeURIComponent(gid) +
                '&pid=' + encodeURIComponent(localPid || 0), { password: pwd }).then(function (data) {
                $timeout(function () {
                    if (!data || data.ok === false) {
                        return;
                    }
                    $scope.passwordProtected = !!data.passwordProtected;
                    if (!$scope.passwordProtected) {
                        $scope.roomPasswordDraft = '';
                    }
                });
            });
        };

        $scope.applyLobbyMatchOptions = function () {
            if (!$scope.isLobbyHost || !$scope.isLobbyHost()) {
                return;
            }
            if (!$scope._joinedGameroom) {
                return;
            }
            var gid = $scope.gameId || '1';
            var localPid = localPersonaId();
            var special = $scope.lobbyOptions.enableSpecialAbilities !== false;
            var tech = $scope.lobbyOptions.enableTechTree !== false;
            var oil = false;
            var infinite = false;
            httpRequest('POST', '/cnc/lobby-options?gid=' + encodeURIComponent(gid) +
                '&pid=' + encodeURIComponent(localPid || 0), {
                specialAbilities: special,
                techTree: tech,
                oilEconomy: oil,
                infiniteResourceCenters: infinite
            }).then(function (data) {
                $timeout(function () {
                    if (!data || data.ok === false) {
                        return;
                    }
                    if (data.enableSpecialAbilities != null) {
                        $scope.lobbyOptions.enableSpecialAbilities = !!data.enableSpecialAbilities;
                    }
                    if (data.enableTechTree != null) {
                        $scope.lobbyOptions.enableTechTree = !!data.enableTechTree;
                    }
                    if (data.enableOilEconomy != null) {
                        $scope.lobbyOptions.enableOilEconomy = false;
                    }
                    if (data.enableInfiniteResourceCenters != null) {
                        $scope.lobbyOptions.enableInfiniteResourceCenters = false;
                    }
                });
            });
        };

        $scope.clearRoomPassword = function () {
            $scope.roomPasswordDraft = '';
            $scope.applyRoomPassword();
        };

        $scope.exitLobby = function () {
            if ($scope._joinedGameroom) {
                postLeaveGameRoom();
            }
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
