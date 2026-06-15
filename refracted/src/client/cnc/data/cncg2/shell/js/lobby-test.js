/**
 * Skirmish lobby — Alpha_Tutorial benchmark (1 host + 1 AI).
 */
(function (window) {
    'use strict';

    var BENCHMARK_MAP = {
        id: 'Alpha_Tutorial',
        path: 'levels/SP/Alpha_Tutorial/Alpha_Tutorial',
        label: 'Alpha Tutorial'
    };

    function emptySlot() {
        return { occupied: false };
    }

    function codenameForFaction(code) {
        var map = { USA: 'TASKMASTER', APA: 'TACTICIAN', ESC: 'RED ARROW', GLA: 'GHOST', EU: 'CLEAVER' };
        return map[code] || 'GENERAL';
    }

    var appModule;
    try {
        appModule = angular.module('CCApp');
    } catch (e) {
        appModule = angular.module('CCApp', []);
    }

    appModule.controller('LobbyController', function ($scope, $timeout, $rootScope) {
        $rootScope.lobbySidebarTab = 'CHAT';
        $scope.subTab = 'GENERALS';
        $scope.mapLabel = BENCHMARK_MAP.label;
        $scope.mapPath = BENCHMARK_MAP.path;
        $scope.matchLabel = '1v1 Skirmish';
        $scope.gameId = '1';
        $scope.onlineCount = 133580;
        $scope.statusLine = '';
        $scope.hostReady = false;
        $scope.aiDifficulties = ['EASY', 'MEDIUM', 'HARD'];
        $scope.factions = [
            { code: 'USA' },
            { code: 'APA' },
            { code: 'EU' },
            { code: 'GLA' }
        ];

        $scope.team1 = [emptySlot(), emptySlot(), emptySlot()];
        $scope.team2 = [emptySlot(), emptySlot(), emptySlot()];

        $scope.selectedSlot = null;
        $scope.selectedTeam = 0;

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

        $scope.factionSwatchClass = function (code, active) {
            var cls = { active: !!active };
            cls['swatch-' + String(code).toLowerCase()] = true;
            return cls;
        };

        $scope.factionCssClass = function (code) {
            if (!code) {
                return '';
            }
            return 'faction-' + String(code).toLowerCase();
        };

        $scope.setSubTab = function (name) {
            $scope.subTab = name;
        };

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
            host.codename = codenameForFaction(host.faction || 'USA');
            host.faction = host.faction || 'USA';
            host.startpoint = host.startpoint || 1;
            host.teamNum = 1;
            host.ready = !!pid;
            host.difficulty = 'MEDIUM';

            $scope.hostReady = !!pid;
            if (!pid) {
                $scope.statusLine = 'Waiting for persona ID — authenticate via shell first.';
            } else if (!$scope.statusLine || $scope.statusLine.indexOf('Sent ') !== 0) {
                $scope.statusLine = 'Host ready · PID ' + pid;
            }

            if (!$scope.selectedSlot || $scope.selectedSlot.isLocal) {
                $scope.selectedSlot = host;
                $scope.selectedTeam = 1;
            }
        }

        function firstEmptySlot(team) {
            for (var i = 0; i < team.length; i++) {
                if (!team[i].occupied) {
                    return team[i];
                }
            }
            return null;
        }

        function fillAiSlot(slot, aiPid) {
            slot.occupied = true;
            slot.isAi = true;
            slot.isLocal = false;
            slot.pid = aiPid;
            slot.displayName = 'AI_1';
            slot.codename = codenameForFaction('APA');
            slot.faction = 'APA';
            slot.startpoint = 2;
            slot.teamNum = 2;
            slot.difficulty = 'MEDIUM';
            slot.ready = true;
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
                fillAiSlot(slot, aiPid);
                if (window.CncProbe) {
                    CncProbe._lobbyAiPid = aiPid;
                }
            } catch (e) { /* empty */ }
        }

        function sendAttr(key, value, slot) {
            if (!window.CncProbe || !CncProbe.sendLobbyAttr) {
                $scope.statusLine = 'Blaze bridge unavailable — open from in-game shell.';
                return;
            }
            var pid = slot && slot.pid ? slot.pid : (CncProbe.resolveHostPid && CncProbe.resolveHostPid());
            if (!pid) {
                $scope.statusLine = 'No playerID for Blaze attribute.';
                return;
            }
            CncProbe.sendLobbyAttr(key, value, { gameId: $scope.gameId, playerId: pid });
            $scope.statusLine = 'Sent ' + key + '=' + value + ' (pid ' + pid + ')';
        }

        $scope.applySelectedSlot = function () {
            var slot = $scope.selectedSlot;
            if (!slot || !slot.occupied) {
                return;
            }
            slot.codename = codenameForFaction(slot.faction);
            sendAttr('_faction', slot.faction, slot);
            $timeout(function () {
                sendAttr('_startpoint', String(slot.startpoint), slot);
            }, 250);
            $timeout(function () {
                sendAttr('_team', String(slot.teamNum), slot);
            }, 500);
            $timeout(function () {
                sendAttr('_isai', slot.isAi ? '1' : '0', slot);
            }, 750);
        };

        $scope.setFaction = function (code) {
            if (!$scope.selectedSlot || !$scope.selectedSlot.occupied) {
                return;
            }
            $scope.selectedSlot.faction = code;
            $scope.selectedSlot.codename = codenameForFaction(code);
            $scope.statusLine = 'Faction ' + code + ' (UI only — use Test getFullGameData to hit Blaze)';
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
            slot.difficulty = diff;
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
            $scope.statusLine = 'Invite friend — not wired in test lobby.';
        };

        $scope.addAi = function ($event) {
            if ($event && $event.stopPropagation) {
                $event.stopPropagation();
            }
            if (window.CncProbe && CncProbe.probeAddAi) {
                CncProbe.probeAddAi($scope.gameId);
                $scope.statusLine = 'Add AI via Refracted probe…';
                return;
            }
            $scope.statusLine = 'CncProbe unavailable — use debug probe Add AI.';
        };

        $scope.startBattle = function () {
            $scope.statusLine = 'START BATTLE — wire to finalizeGameCreation when GMGR flow is ready.';
            if (window.CncProbe) {
                CncProbe.log('Lobby: START BATTLE clicked (not wired to match start yet).');
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
        loadAiFromStorage();

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
