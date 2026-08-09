/**
 * Shell lobby — Classic / Aurora match setup + Blaze start wiring.
 * Tutorial map rules + CreateGame path borrowed from lobby-test.js.
 */
(function (window, angular) {
    'use strict';

    if (!angular || !window.CCApp) {
        return;
    }

    var MAP_THUMB = 'images/cnc_background.png';
    var MAPS = [
        {
            id: 'Alpha_Tutorial',
            path: 'Levels/SP/Alpha_Tutorial/Alpha_Tutorial',
            label: 'Alpha Tutorial',
            slots: 1,
            mode: 'TUTORIAL',
            image: MAP_THUMB,
            accent: '#5a8f3a',
            forceTutorialGeneral: true,
            forceFaction: 'EU'
        },
        {
            id: 'excavation',
            label: 'Excavation',
            path: 'Levels/MP/PVP/DM_Smalltown_1v1_CR/DM_Smalltown_1v1_CR',
            slots: 2,
            mode: '1v1',
            image: MAP_THUMB,
            accent: '#c07828'
        },
        {
            id: 'nile-1v1',
            label: 'Nile Delta',
            path: 'Levels/MP/PVP/DM_KapuKai_1v1_JKS/DM_KapuKai_1v1_JKS',
            slots: 2,
            mode: '1v1',
            image: MAP_THUMB,
            accent: '#2a7ab8'
        },
        {
            id: 'oasis-2v2',
            label: 'Oasis',
            path: 'Levels/MP/PVP/DM_Oasis_2v2_JT/DM_Oasis_2v2_JT',
            slots: 4,
            mode: '2v2',
            image: MAP_THUMB,
            accent: '#b83a3a'
        },
        {
            id: 'monsoon-3v3',
            label: 'Monsoon',
            path: 'Levels/MP/PVP/DM_Monsoon_3v3_MO/DM_Monsoon_3v3_MO',
            slots: 6,
            mode: '3v3',
            image: MAP_THUMB,
            accent: '#6a5ab0'
        },
        {
            id: 'overpass-3v3',
            label: 'Overpass',
            path: 'Levels/MP/PVP/DM_Overpass_3v3_JKS/DM_Overpass_3v3_JKS',
            slots: 6,
            mode: '3v3',
            image: MAP_THUMB,
            accent: '#3a9a8a'
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
            startpoint: 1,
            pid: 0
        };
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
        $scope.lobbyOptions = {
            startingCash: 'standard',
            startingUnits: 'standard',
            noBaseBuilding: false,
            noFogOfWar: false
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
        $scope._serverLostError = '';
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

        $scope.mapCardStyle = function (map) {
            if (!map) {
                return {};
            }
            var img = map.image || MAP_THUMB;
            return {
                'background-image': 'url(' + img + ')'
            };
        };

        $scope.mapModeLabel = function (map) {
            if (!map) {
                return '';
            }
            return map.mode || ((map.slots || 2) + 'P');
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

        $scope.selectMap = function (map) {
            if (!map) {
                return;
            }
            $scope.selectedMap = map;
            $scope.mapPath = map.path || '';
            $scope.mapForcesTutorial = !!map.forceTutorialGeneral;
            $scope.mapMenuOpen = false;
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
                        cleared.startpoint = teamNum === 1 ? (i + 1) : (i + 2);
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
        };
        $scope.closeMapMenu = $scope.closeMenus;

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
            $scope.lobbySubTab = 'OPTIONS';
            $scope.lobbyOptionsOpen = true;
        };

        $scope.closeLobbyOptions = function () {
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
                    slot.startpoint = team === 1 ? (i + 1) : (i + 2);
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
            cleared.startpoint = team === 1 ? (index + 1) : (index + 2);
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
                return { label: 'Standby', image: MAP_THUMB, mode: 'IDLE' };
            }
            var path = g.mapPath || '';
            var leaf = g.map || (path ? path.split('/').pop() : '');
            var i;
            for (i = 0; i < MAPS.length; i++) {
                if (path && MAPS[i].path === path) {
                    return {
                        label: MAPS[i].label,
                        image: MAPS[i].image || MAP_THUMB,
                        mode: MAPS[i].mode || ''
                    };
                }
                if (leaf && MAPS[i].path && MAPS[i].path.split('/').pop() === leaf) {
                    return {
                        label: MAPS[i].label,
                        image: MAPS[i].image || MAP_THUMB,
                        mode: MAPS[i].mode || ''
                    };
                }
                if (leaf && MAPS[i].label &&
                        String(MAPS[i].label).toLowerCase() === String(leaf).toLowerCase()) {
                    return {
                        label: MAPS[i].label,
                        image: MAPS[i].image || MAP_THUMB,
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
            $scope._serverLostError = '';
        };

        function forceServerLostKick(message) {
            var inLobby = !!$scope._joinedGameroom;
            var inBlaze = !!(window.CncProbe && CncProbe._inBlazeGame);
            if (!inLobby && !inBlaze) {
                return;
            }
            if ($scope._starting || inBlaze) {
                $scope._serverLostError = message || 'The connectivity to the server was lost.';
                $scope._starting = false;
                $scope._startError = '';
                $scope._findingMatch = false;
                $scope._matchError = '';
                if (window.CncProbe && CncProbe.log) {
                    CncProbe.log('Lobby SERVER LOST (start/match — no leave-reclaim): ' +
                        $scope._serverLostError);
                }
                stopRosterPoll();
                return;
            }
            $scope._serverLostError = message || 'The connectivity to the server was lost.';
            $scope._starting = false;
            $scope._startError = '';
            $scope._findingMatch = false;
            $scope._matchError = '';
            if (window.CncProbe && CncProbe.log) {
                CncProbe.log('Lobby SERVER LOST: ' + $scope._serverLostError);
            }
            if (inLobby) {
                if ($rootScope.leaveGameRoom) {
                    $rootScope.leaveGameRoom();
                } else {
                    postLeaveGameRoom();
                }
            } else if (window.CncProbe) {
                CncProbe._inBlazeGame = false;
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
                        var idlePool = (pool && typeof pool.idle === 'number') ? pool.idle : 0;
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
                            failFind('No idle servers found, check back later.');
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
            }
        }

        function clearRemoteHumanSlots() {
            function wipe(slots) {
                var i;
                for (i = 0; i < slots.length; i++) {
                    if (slots[i] && slots[i].occupied && !slots[i].isLocal && !slots[i].isAi) {
                        slots[i] = emptySlot();
                        slots[i].teamNum = slots === $scope.team2 ? 2 : 1;
                        slots[i].startpoint = slots === $scope.team2 ? (i + 2) : (i + 1);
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
                    s.startpoint = teamNum === 1 ? (i + 1) : (i + 2);
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
                    }
                } else {
                    remotes.push(p);
                }
            }
            if ($scope._rosterSawSelf && !localStillIn && $scope._joinedGameroom) {
                $scope._rosterMissSelf = ($scope._rosterMissSelf || 0) + 1;
                if ($scope._rosterMissSelf >= 3) {
                    forceServerLostKick('The connectivity to the server was lost.');
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
            var inSession = $scope._joinedGameroom ||
                (window.CncProbe && CncProbe._inBlazeGame);
            if (!inSession) {
                stopRosterPoll();
                return;
            }
            var gid = $scope.gameId || '1';
            httpRequest('GET', '/cnc/lobby-roster?gid=' + encodeURIComponent(gid)).then(function (data) {
                $timeout(function () {
                    if (data && data.serverLost) {
                        forceServerLostKick(data.message ||
                            'The connectivity to the server was lost.');
                        return;
                    }
                    if (data && data.ok === false) {
                        inSession = $scope._joinedGameroom ||
                            (window.CncProbe && CncProbe._inBlazeGame);
                        if (inSession) {
                            $scope._rosterPoll = $timeout(pollLobbyRoster, 1500);
                        }
                        return;
                    }
                    if (data && $scope._joinedGameroom) {
                        applyRosterFlags(data);
                    }
                    inSession = $scope._joinedGameroom ||
                        (window.CncProbe && CncProbe._inBlazeGame);
                    if (inSession) {
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
                '&path=' + encodeURIComponent(level);
            if (window.CncProbe && CncProbe.log) {
                CncProbe.log('select-map → gid=' + $scope.gameId + ' path=' + level +
                    ' label=' + (($scope.selectedMap && $scope.selectedMap.label) || ''));
            }
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
            var startUnix = Math.floor(Date.now() / 1000);
            applyTutorialConstraints();

            var host = $scope.team1[0];
            var gname = (host && host.displayName) || 'Player1';
            var ais = aiSlots();
            var capacity = $scope.isSoloMap() ? 1 : Math.max(2, 1 + ais.length);
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

            function failStart(msg) {
                clearStartTimer();
                CncProbe.onLobbyStartResult = null;
                if (CncProbe._pendingBlazeCreate) {
                    CncProbe._pendingBlazeCreate = false;
                }
                CncProbe._inBlazeGame = false;
                $scope._starting = false;
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
                        CncProbe._inBlazeGame = true;
                        $scope._joinedGameroom = false;
                        stopRosterPoll();
                        try {
                            sessionStorage.setItem('cnc_match_gid', String($scope.gameId || '1'));
                            var pid = localPersonaId();
                            if (pid) {
                                sessionStorage.setItem('cnc_match_pid', String(pid));
                            }
                        } catch (e) { /* ignore */ }
                        CncProbe.onLobbyStartResult = null;
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
            }, 90000);

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

            whenAll(pending, beginCreate, function () {
                failStart('Failed to communicate with the backend, please try again.');
            });

            function beginCreate() {
                httpRequest('GET', '/cnc/dedicated-pool').then(function (pool) {
                    $timeout(function () {
                        if (!$scope._starting || $scope._startError) {
                            return;
                        }
                        if (pool && typeof pool.idle === 'number' && pool.idle <= 0) {
                            failStart('No idle servers found, check back later.');
                            return;
                        }
                        fireCreate();
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
