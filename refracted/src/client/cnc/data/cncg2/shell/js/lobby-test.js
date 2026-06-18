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
        $scope.subTab = 'GENERALS';
        $scope.mapLabel = BENCHMARK_MAP.label;
        $scope.mapPath = BENCHMARK_MAP.path;
        $scope.matchLabel = '1v1 Skirmish';
        $scope.gameId = '1';
        $scope.onlineCount = 0;
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

        $scope.startpointLabel = function (n) {
            if (n == null || n === '') {
                return '—';
            }
            return 'Slot ' + n;
        };

        function refreshOnlineCount() {
            if (!window.fetch) {
                return;
            }
            fetch('/cnc/online-count', { method: 'GET', credentials: 'same-origin' })
                .then(function (r) { return r.json(); })
                .then(function (body) {
                    if (!body || !body.ok) {
                        return;
                    }
                    var n = body.count != null ? body.count : body.active;
                    if (n != null && !$scope.$$phase) {
                        $scope.$apply(function () {
                            $scope.onlineCount = n;
                        });
                    } else if (n != null) {
                        $scope.onlineCount = n;
                    }
                })
                .catch(function () { /* Refracted offline */ });
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
            if (pid && host.faction) {
                $timeout(function () {
                    sendAttr('_faction', host.faction, host);
                }, 400);
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
            slot.codename = codenameForFaction('APA');
            slot.faction = 'APA';
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
                $scope.statusLine = 'AI slot open — waiting for persona ID from blazeGetPlayers / server log.';
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
                sendAttr('_startpoint', String(slot.startpoint), slot);
            }, 500);
            $timeout(function () {
                sendAttr('_team', String(slot.teamNum), slot);
            }, 750);
            $timeout(function () {
                sendDifficulty(slot, slot.difficulty || 'MEDIUM');
            }, 1000);
            $scope.statusLine = 'AI ready · PID ' + pid;
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
            var host = $scope.team1[0];
            if (!host || !host.occupied || !host.isLocal) {
                return;
            }
            host.faction = code;
            host.codename = codenameForFaction(code);
            $scope.selectedSlot = host;
            $scope.selectedTeam = 1;
            sendAttr('_faction', code, host);
            $scope.statusLine = 'Faction ' + code + ' · PID ' + (host.pid || '?');
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
            sendDifficulty(slot, diff);
            $scope.statusLine = 'AI difficulty ' + diff + ' · PID ' + (slot.pid || 'pending');
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
            if (!window.CncProbe || !CncProbe.runGame) {
                $scope.statusLine = 'gameclient unavailable — open lobby from in-game shell.';
                return;
            }
            var slot = firstEmptySlot($scope.team2);
            if (!slot) {
                $scope.statusLine = 'Team 2 has no open slot.';
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
            $scope.statusLine = 'AddRemotePlayer team ' + teamNum + ' start ' + startpoint + '…';
            if (CncProbe.runAddRemotePlayer) {
                CncProbe.runAddRemotePlayer(teamNum, startpoint, {
                    gameId: $scope.gameId,
                    pollDelayMs: 1200
                });
            } else {
                CncProbe.runGame('RtsClient.AddRemotePlayer ' + teamNum + ' ' + startpoint);
                $timeout(function () {
                    CncProbe.runGame('RtsClient.blazeGetPlayers ' + $scope.gameId);
                }, 1200);
            }
            fillAiSlot(slot, '', teamNum, startpoint);
            $scope.selectedSlot = slot;
            $scope.selectedTeam = teamNum;
            $timeout(function () {
                loadAiFromStorage();
                applyAiAttrs(slot);
            }, 3000);
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
