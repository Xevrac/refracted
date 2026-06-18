/**
 * CNC debug probe — shellaccesslayer (EAWebKit) + gameclient (Frostbite) hooks.
 * Patterns: https://github.com/Xevrac/cnc_backend/wiki
 */
(function (window) {
    'use strict';

    var CncProbe = {
        outEl: function () {
            return document.getElementById('cnc-probe-modal-out') || document.getElementById('debugOutput');
        },
        log: function (msg) {
            var t = (typeof msg === 'string' ? msg : String(msg));
            var el = CncProbe.outEl();
            if (el) {
                el.innerText = t;
            }
            if (window.console && console.log) { console.log('[CncProbe]', t); }
        },
        /** For shell / HTTP JSON results */
        shellResult: function (res) {
            try {
                var msg = 'RESPONSE: ' + JSON.stringify(res, null, 2);
                if (res && typeof res === 'object' && ('status' in res || 'success' in res || 'error' in res)) {
                    msg += '\n\nShell status: ' + (res.status != null ? res.status : '—')
                        + ' | success: ' + (res.success != null ? res.success : '—')
                        + (res.error ? (' | error: ' + res.error) : '');
                }
                CncProbe.log(msg);
            } catch (e) {
                CncProbe.log('RESPONSE: ' + String(res));
            }
            if (res && typeof res === 'object') {
                var blob = JSON.stringify(res);
                if (blob.indexOf('ALREADY_GAME_MEMBER') >= 0) {
                    CncProbe._inBlazeGame = true;
                }
                if (res.success === true || res.status === 0) {
                    if (CncProbe._pendingBlazeCreate) {
                        CncProbe._inBlazeGame = true;
                        CncProbe._pendingBlazeCreate = false;
                    }
                }
            }
            if (window.CncBlazeState) CncBlazeState.onShellResult(res);
            if (window.CncProbe && CncProbe.lobbyRefreshPidDisplay) CncProbe.lobbyRefreshPidDisplay();
        },
        hasShell: function () {
            return typeof shellaccesslayer !== 'undefined' && shellaccesslayer && typeof shellaccesslayer.execute === 'function';
        },
        hasGame: function () {
            return typeof gameclient !== 'undefined' && gameclient && typeof gameclient.execute === 'function';
        },
        logAvailability: function () {
            CncProbe.log('hasShell: ' + CncProbe.hasShell() + '\nhasGame: ' + CncProbe.hasGame());
        },
        runShell: function (req) {
            if (!CncProbe.hasShell()) {
                CncProbe.log('shellaccesslayer not available (not in EAWebKit game shell).');
                return;
            }
            req = req || {};
            var originalResponse = (typeof req._response === 'function') ? req._response : null;
            var wrapped = {};
            for (var k in req) {
                if (Object.prototype.hasOwnProperty.call(req, k)) {
                    wrapped[k] = req[k];
                }
            }
            wrapped._response = function (res) {
                CncProbe.shellResult(res);
                if (originalResponse && originalResponse !== CncProbe.shellResult) {
                    try { originalResponse(res); } catch (e) { /* empty */ }
                }
            };
            try {
                CncProbe.log('Request: ' + JSON.stringify(req));
            } catch (e) {
                CncProbe.log('Request sent.');
            }
            shellaccesslayer.execute(wrapped);
        },
        runGame: function (line) {
            if (!CncProbe.hasGame()) {
                CncProbe.log('gameclient not available.');
                return;
            }
            CncProbe.log('gameclient: ' + line);
            try {
                gameclient.execute(line);
            } catch (e) {
                CncProbe.log('Error: ' + (e && e.message ? e.message : e));
            }
        },
        toggleDock: function () {
            var d = document.getElementById('cnc-probe-dock');
            if (d) { d.classList.toggle('cnc-probe-open'); }
        },
        initOutputModal: function () {
            var modal = document.getElementById('cnc-probe-output-modal');
            var drag = document.getElementById('cnc-probe-output-drag');
            if (!modal || !drag || modal.getAttribute('data-init-drag') === '1') {
                return;
            }
            modal.setAttribute('data-init-drag', '1');

            var startX = 0;
            var startY = 0;
            var originLeft = 0;
            var originTop = 0;
            var dragging = false;

            var onMove = function (ev) {
                if (!dragging) {
                    return;
                }
                var dx = ev.clientX - startX;
                var dy = ev.clientY - startY;
                modal.style.left = (originLeft + dx) + 'px';
                modal.style.top = (originTop + dy) + 'px';
                modal.style.right = 'auto';
                modal.style.bottom = 'auto';
            };
            var onUp = function () {
                dragging = false;
                document.removeEventListener('mousemove', onMove);
                document.removeEventListener('mouseup', onUp);
            };
            drag.addEventListener('mousedown', function (ev) {
                dragging = true;
                startX = ev.clientX;
                startY = ev.clientY;
                var rect = modal.getBoundingClientRect();
                originLeft = rect.left;
                originTop = rect.top;
                document.addEventListener('mousemove', onMove);
                document.addEventListener('mouseup', onUp);
                ev.preventDefault();
            });
        },
        onGameLine: function () {
            var ta = document.getElementById('cnc-probe-gameclient-line');
            if (ta) { CncProbe.runGame(ta.value); }
        },
        onShellUrlLine: function () {
            var ta = document.getElementById('cnc-probe-shell-url-line');
            if (!ta) { return; }
            var url = (ta.value || '').trim();
            if (!url) {
                CncProbe.log('No shell URL entered.');
                return;
            }
            CncProbe.runShell({ _response: CncProbe.shellResult, url: url });
        },
        onShellResourceLine: function () {
            var ta = document.getElementById('cnc-probe-shell-resource-line');
            if (!ta) { return; }
            var resource = (ta.value || '').trim();
            if (!resource) {
                CncProbe.log('No shell resource entered.');
                return;
            }
            CncProbe.runShell({ _response: CncProbe.shellResult, _resource: resource });
        },
        onGameGetValue: function () {
            if (!CncProbe.hasGame() || typeof gameclient.getValue !== 'function') {
                CncProbe.log('gameclient.getValue not available.');
                return;
            }
            var input = document.getElementById('cnc-probe-getvalue-key');
            var key = input ? (input.value || '').trim() : '';
            if (!key) {
                CncProbe.log('No key for getValue.');
                return;
            }
            try {
                var value = gameclient.getValue(key);
                CncProbe.log('getValue ' + key + ' = ' + String(value));
            } catch (e) {
                CncProbe.log('getValue error: ' + (e && e.message ? e.message : e));
            }
        },
        onGameSetValue: function () {
            if (!CncProbe.hasGame() || typeof gameclient.setValue !== 'function') {
                CncProbe.log('gameclient.setValue not available.');
                return;
            }
            var keyInput = document.getElementById('cnc-probe-setvalue-key');
            var valueInput = document.getElementById('cnc-probe-setvalue-value');
            var key = keyInput ? (keyInput.value || '').trim() : '';
            var value = valueInput ? (valueInput.value || '').trim() : '';
            if (!key) {
                CncProbe.log('No key for setValue.');
                return;
            }
            try {
                gameclient.setValue(key, value);
                CncProbe.log('setValue ' + key + ' = ' + value);
            } catch (e) {
                CncProbe.log('setValue error: ' + (e && e.message ? e.message : e));
            }
        },
        setGameLine: function (line) {
            var ta = document.getElementById('cnc-probe-gameclient-line');
            if (ta) { ta.value = line; }
            CncProbe.runGame(line);
        },
        setShellUrlAndRun: function (url) {
            var ta = document.getElementById('cnc-probe-shell-url-line');
            if (ta) { ta.value = url; }
            CncProbe.runShell({ _response: CncProbe.shellResult, url: url });
        },
        dumpOutputToLog: function () {
            var el = CncProbe.outEl();
            var text = el ? (el.innerText || el.textContent || '') : '';
            if (!text) {
                CncProbe.log('No output to save yet.');
                return;
            }

            var now = new Date();
            var pad2 = function (n) { return n < 10 ? '0' + n : String(n); };
            var stamp = now.getFullYear() +
                pad2(now.getMonth() + 1) +
                pad2(now.getDate()) + '-' +
                pad2(now.getHours()) +
                pad2(now.getMinutes()) +
                pad2(now.getSeconds());
            var filename = 'cnc-probe-log-' + stamp + '.txt';
            var payload = '[CncProbe dump ' + now.toISOString() + ']\n\n' + text + '\n';

            // In-memory fallback for inspection from devtools
            try {
                if (!window.__CNC_PROBE_LOG_DUMPS) {
                    window.__CNC_PROBE_LOG_DUMPS = [];
                }
                window.__CNC_PROBE_LOG_DUMPS.push({
                    filename: filename,
                    createdAt: now.toISOString(),
                    text: payload
                });
            } catch (e0) { /* empty */ }
            try {
                if (window.localStorage) {
                    var k = 'cncProbeLogDumps';
                    var parsed = [];
                    try { parsed = JSON.parse(localStorage.getItem(k) || '[]'); } catch (e1) { parsed = []; }
                    parsed.push({ filename: filename, createdAt: now.toISOString(), text: payload });
                    if (parsed.length > 20) {
                        parsed = parsed.slice(parsed.length - 20);
                    }
                    localStorage.setItem(k, JSON.stringify(parsed));
                }
            } catch (e2) { /* empty */ }

            var url = '/cnc/probe-dump?filename=' + encodeURIComponent(filename);
            var finishOk = function (msg) { CncProbe.log(msg || ('Saved: ' + filename)); };
            var tryBlob = function () {
                try {
                    var blob = new Blob([payload], { type: 'text/plain;charset=utf-8' });
                    var u = (window.URL || window.webkitURL).createObjectURL(blob);
                    var a = document.createElement('a');
                    a.href = u;
                    a.download = filename;
                    a.style.display = 'none';
                    document.body.appendChild(a);
                    a.click();
                    setTimeout(function () {
                        try {
                            if (document.body.contains(a)) {
                                document.body.removeChild(a);
                            }
                            (window.URL || window.webkitURL).revokeObjectURL(u);
                        } catch (e3) { /* empty */ }
                    }, 0);
                    finishOk(
                        'Download triggered (client blob): ' + filename + '\n' +
                        'If nothing appears, the shell blocked it — check window.__CNC_PROBE_LOG_DUMPS or localStorage "cncProbeLogDumps".'
                    );
                } catch (e4) {
                    CncProbe.log(
                        'Client-side save failed. Last dump: window.__CNC_PROBE_LOG_DUMPS / localStorage "cncProbeLogDumps".\n' + (e4 && e4.message ? e4.message : e4)
                    );
                }
            };

            if (window.fetch) {
                fetch(url, {
                    method: 'POST',
                    body: payload,
                    headers: { 'Content-Type': 'text/plain;charset=utf-8' }
                })
                    .then(function (r) {
                        if (!r.ok) {
                            throw new Error('HTTP ' + r.status);
                        }
                        return r.blob();
                    })
                    .then(function (blob) {
                        var objectUrl = (window.URL || window.webkitURL).createObjectURL(blob);
                        var a = document.createElement('a');
                        a.href = objectUrl;
                        a.download = filename;
                        a.style.display = 'none';
                        document.body.appendChild(a);
                        a.click();
                        setTimeout(function () {
                            try {
                                if (document.body.contains(a)) {
                                    document.body.removeChild(a);
                                }
                                (window.URL || window.webkitURL).revokeObjectURL(objectUrl);
                            } catch (e) { /* empty */ }
                        }, 0);
                        finishOk('Saved via server (POST ' + url.split('?')[0] + '): ' + filename);
                    })
                    .catch(function () { tryBlob(); });
            } else {
                tryBlob();
            }
        }
    };

    // ---- Blaze shell — CNC routes GMGR via url: '/blaze/…' (not _module short names) ----
    CncProbe.blazeUrlFromResource = function (resource, params) {
        params = params || {};
        var res = String(resource || '').trim();
        var key = res.toLowerCase();
        var pathByResource = {
            authenticate: '/blaze/authenticate',
            tokenauthenticate: '/blaze/tokenauthenticate',
            games: '/blaze/games',
            creategame: '/blaze/createGame',
            joingame: '/blaze/joinGame',
            attribute: '/blaze/attribute'
        };
        var base;
        if (pathByResource[key]) {
            base = pathByResource[key];
        } else if (res.indexOf('/') >= 0) {
            base = '/blaze/' + res.replace(/^\/+/, '');
        } else {
            base = '/blaze/' + res;
        }
        var parts = [];
        for (var k in params) {
            if (Object.prototype.hasOwnProperty.call(params, k)) {
                parts.push(encodeURIComponent(k) + '=' + encodeURIComponent(String(params[k])));
            }
        }
        return parts.length ? base + '?' + parts.join('&') : base;
    };
    CncProbe.runBlazeUrl = function (urlPath) {
        CncProbe.runShell({ _response: CncProbe.shellResult, url: urlPath });
    };
    /** Primary Blaze shell path — builds url: '/blaze/…' (matches shell.js / src.js). */
    CncProbe.runBlaze = function (resource, params) {
        CncProbe.runBlazeUrl(CncProbe.blazeUrlFromResource(resource, params));
    };
    /** Experimental: _module:'blaze' + short _resource — does not dispatch on cnc.server.exe today. */
    CncProbe.runBlazeModule = function (resource, params) {
        params = params || {};
        var req = {
            _module: 'blaze',
            _resource: resource,
            _response: CncProbe.shellResult
        };
        for (var k in params) {
            if (Object.prototype.hasOwnProperty.call(params, k)) {
                req[k] = params[k];
            }
        }
        CncProbe.runShell(req);
    };
    CncProbe.blazeShellInput = function (id, fallback) {
        var el = document.getElementById(id);
        var v = el ? String(el.value || '').trim() : '';
        return v !== '' ? v : (fallback !== undefined ? String(fallback) : '');
    };
    CncProbe.blazeShellAuthenticate = function () {
        CncProbe.runBlaze('authenticate', {
            email: CncProbe.blazeShellInput('cnc-probe-blaze-email', 'user@example.com'),
            password: CncProbe.blazeShellInput('cnc-probe-blaze-pass', 'test')
        });
    };
    CncProbe.blazeShellTokenAuthenticate = function () {
        CncProbe.runBlaze('tokenauthenticate');
    };
    CncProbe.blazeShellGames = function () {
        CncProbe.runBlaze('games');
    };
    CncProbe._inBlazeGame = false;
    CncProbe._pendingBlazeCreate = false;

    CncProbe.markBlazeCreatePending = function () {
        CncProbe._pendingBlazeCreate = true;
    };

    CncProbe.blazeShellCreateGame = function () {
        CncProbe.markBlazeCreatePending();
        CncProbe.runBlaze('creategame', {
            gameName: CncProbe.blazeShellInput('cnc-probe-blaze-gname', 'XEVRAC'),
            players: CncProbe.blazeShellInput('cnc-probe-blaze-players', '2')
        });
    };
    CncProbe.blazeShellJoinGame = function () {
        if (CncProbe._inBlazeGame) {
            CncProbe.log(
                'joinGame skipped — createGame / resetDedicatedServer already put you in the game.\n' +
                '(Shell error GAMEMANAGER_ERR_ALREADY_GAME_MEMBER is expected if you join again.)'
            );
            return;
        }
        CncProbe.runBlaze('joinGame', {
            gameID: CncProbe.blazeShellInput('cnc-probe-blaze-gameid', '1')
        });
    };

    // ---- Blaze shell URL route (explicit /blaze/… — same wire as runBlaze above) ----
    CncProbe.blazeUrlAuthenticate = function () {
        CncProbe.runBlazeUrl(CncProbe.blazeUrlFromResource('authenticate', {
            email: CncProbe.blazeShellInput('cnc-probe-blaze-email', 'user@example.com'),
            password: CncProbe.blazeShellInput('cnc-probe-blaze-pass', 'test')
        }));
    };
    CncProbe.blazeUrlTokenAuthenticate = function () {
        CncProbe.runBlazeUrl('/blaze/tokenauthenticate');
    };
    CncProbe.blazeUrlGames = function () {
        CncProbe.runBlazeUrl('/blaze/games');
    };
    CncProbe.blazeUrlCreateGame = function () {
        CncProbe.markBlazeCreatePending();
        CncProbe.runBlazeUrl(CncProbe.blazeUrlFromResource('creategame', {
            gameName: CncProbe.blazeShellInput('cnc-probe-blaze-gname', 'XEVRAC'),
            players: CncProbe.blazeShellInput('cnc-probe-blaze-players', '2')
        }));
    };
    CncProbe.blazeUrlJoinGame = function () {
        CncProbe.blazeShellJoinGame();
    };
    CncProbe.blazeModuleCreateGame = function () {
        CncProbe.runBlazeModule('creategame', {
            gameName: CncProbe.blazeShellInput('cnc-probe-blaze-gname', 'XEVRAC'),
            players: CncProbe.blazeShellInput('cnc-probe-blaze-players', '2')
        });
    };

    CncProbe.logAddAiHelp = function () {
        CncProbe.log(
            'Add AI (retail only):\n' +
            '  • RtsClient.AddRemotePlayer <team> <startpoint> queues the slot locally.\n' +
            '  • Client flushes to GMGR addQueuedPlayerToGame (RPC 38) once [RtsGameClient+0xE5] is set.\n' +
            '  • That flag is set by vtable slot 3 after ClientLevel_spawnEntities (post level bind).\n' +
            '  • After add, use blazeGetPlayers or CncProbe.setLobbyAiPid(ai_pid).\n\n' +
            'Create game first via createGame / resetDedicatedServer — do not joinGame after (already a member).'
        );
    };
    /** Retail skirmish path: queue remote/AI slot by Blaze team + startpoint (message +44/+48). */
    CncProbe.runAddRemotePlayer = function (team, startpoint, opts) {
        opts = opts || {};
        if (!CncProbe.hasGame()) {
            CncProbe.log('runAddRemotePlayer: gameclient not available.');
            return false;
        }
        team = team != null ? String(team) : '2';
        startpoint = startpoint != null ? String(startpoint) : '2';
        CncProbe.runGame('RtsClient.AddRemotePlayer ' + team + ' ' + startpoint);
        if (opts.blazeGetPlayers !== false) {
            var gid = opts.gameId != null ? String(opts.gameId) : '1';
            var delay = opts.pollDelayMs != null ? opts.pollDelayMs : 800;
            setTimeout(function () {
                CncProbe.runGame('RtsClient.blazeGetPlayers ' + gid);
            }, delay);
            if (opts.pollAgainMs) {
                setTimeout(function () {
                    CncProbe.runGame('RtsClient.blazeGetPlayers ' + gid);
                }, opts.pollAgainMs);
            }
        }
        return true;
    };
    CncProbe.promptInGameAddAi = function () {
        CncProbe.runAddRemotePlayer('2', '2', { blazeGetPlayers: true });
    };
    CncProbe.syncAiFromGetPlayers = function () {
        CncProbe.log('blazeGetPlayers via gameclient — check Frostbite log / console for AI persona ids.');
        CncProbe.rtBlazeGetPlayersFromInput();
    };
    /** Dev escape hatch after in-game Add AI: paste ai_pid from Refracted console log. */
    CncProbe.setLobbyAiPid = function (pid) {
        if (pid == null || String(pid).trim() === '') {
            return;
        }
        CncProbe._lobbyAiPid = String(pid).trim();
        try {
            sessionStorage.setItem('cnc_lobby_ai_pid', CncProbe._lobbyAiPid);
        } catch (e) { /* empty */ }
        CncProbe.lobbyTargetAi();
        CncProbe.lobbyRefreshPidDisplay();
        CncProbe.log('AI slot pid set to ' + CncProbe._lobbyAiPid);
    };

    CncProbe.openLobbyTest = function () {
        window.location.href = 'lobby.html';
    };

    /** Blaze setPlayerAttributes without debug-probe form fields (lobby test page). */
    CncProbe.sendLobbyAttr = function (key, value, opts) {
        opts = opts || {};
        var gid = opts.gameId != null ? String(opts.gameId) : '1';
        var pid = opts.playerId != null ? String(opts.playerId) : CncProbe.resolveHostPid();
        if (!pid) {
            CncProbe.log('sendLobbyAttr: no playerID — set Refracted profile or authenticate first.');
            return false;
        }
        CncProbe.runBlaze('attribute', {
            gameID: gid,
            playerID: pid,
            key: String(key),
            value: String(value)
        });
        return true;
    };
    /** cmd 7 — setPlayerAttributes (gameID + playerID + key + value) */
    CncProbe.lobbyTargetMode = 'host';
    CncProbe._lobbyAiPid = null;

    CncProbe.resolveHostPid = function () {
        if (window.CncBlazeState) {
            CncBlazeState.applyExternalHints();
            if (CncBlazeState.personaId != null && String(CncBlazeState.personaId) !== '') {
                return String(CncBlazeState.personaId);
            }
        }
        var prof = window.__CNC_PROFILE;
        if (prof && prof.personaId != null) {
            return String(prof.personaId);
        }
        try {
            var ss = sessionStorage.getItem('cnc_blaze_session');
            if (ss) {
                var o = JSON.parse(ss);
                if (o && o.personaId != null) {
                    return String(o.personaId);
                }
            }
            var pidOnly = sessionStorage.getItem('cnc_blaze_persona_id');
            if (pidOnly) {
                return String(pidOnly);
            }
        } catch (e) { /* empty */ }
        return '';
    };

    CncProbe.resolveHostName = function () {
        if (window.CncBlazeState) {
            CncBlazeState.applyExternalHints();
            var n = CncBlazeState.getPlayerName();
            if (n && n !== CncBlazeState.UNKNOWN_PLAYER) {
                return n;
            }
        }
        var prof = window.__CNC_PROFILE;
        if (prof && prof.displayName) {
            return String(prof.displayName);
        }
        return '';
    };

    CncProbe.lobbyRefreshPidDisplay = function () {
        var el = document.getElementById('cnc-probe-lobby-pid-display');
        if (el) {
            var host = CncProbe.resolveHostPid();
            var name = CncProbe.resolveHostName();
            var ai = CncProbe._lobbyAiPid;
            var parts = [];
            if (host) {
                parts.push('host: ' + (name ? name + ' · ' : '') + host);
            } else {
                parts.push('host: (set profile in Refracted Accounts)');
            }
            parts.push(ai ? ('AI: ' + ai) : 'AI: — (in-game Add AI)');
            el.textContent = parts.join('  |  ');
        }
        CncProbe.lobbyUpdateTargetLabel();
    };

    CncProbe.initLobbyPids = function () {
        if (window.CncBlazeState) {
            CncBlazeState.applyExternalHints();
            CncBlazeState.subscribe(function () {
                CncProbe.lobbyRefreshPidDisplay();
            });
        }
        CncProbe.lobbyRefreshPidDisplay();
    };

    CncProbe.lobbyUpdateTargetLabel = function () {
        var el = document.getElementById('cnc-probe-lobby-target-label');
        if (!el) {
            return;
        }
        var pid = CncProbe.lobbyActivePlayerId();
        el.textContent = CncProbe.lobbyTargetMode + (pid ? ' · pid ' + pid : '');
    };
    CncProbe.lobbyActivePlayerId = function () {
        if (CncProbe.lobbyTargetMode === 'ai') {
            return CncProbe._lobbyAiPid ? String(CncProbe._lobbyAiPid) : '';
        }
        return CncProbe.resolveHostPid();
    };
    CncProbe.lobbyTargetHost = function () {
        CncProbe.lobbyTargetMode = 'host';
        CncProbe.lobbyUpdateTargetLabel();
    };
    CncProbe.lobbyTargetAi = function () {
        CncProbe.lobbyTargetMode = 'ai';
        CncProbe.lobbyUpdateTargetLabel();
        if (!CncProbe.lobbyActivePlayerId()) {
            CncProbe.log('No AI yet — use in-game lobby Add AI (GMGR 0x26), then AI slot.');
        }
    };
    CncProbe.blazeShellSetPlayerAttribute = function () {
        var pid = CncProbe.lobbyActivePlayerId();
        if (!pid) {
            CncProbe.log('playerID required — select host slot (profile) or AI slot (Add AI first).');
            return;
        }
        CncProbe.runBlaze('attribute', {
            gameID: CncProbe.blazeShellInput('cnc-probe-blaze-attr-gid', '1'),
            playerID: pid,
            key: CncProbe.blazeShellInput('cnc-probe-blaze-attr-key', 'testKey'),
            value: CncProbe.blazeShellInput('cnc-probe-blaze-attr-val', 'testVal')
        });
    };
    /** cmd 10 — setGameAttributes (gameID + key + value; omit playerID) */
    CncProbe.blazeShellSetGameAttribute = function () {
        CncProbe.runBlaze('attribute', {
            gameID: CncProbe.blazeShellInput('cnc-probe-blaze-attr-gid', '1'),
            key: CncProbe.blazeShellInput('cnc-probe-blaze-attr-key', 'testKey'),
            value: CncProbe.blazeShellInput('cnc-probe-blaze-attr-val', 'testVal')
        });
    };

    // ---- Lobby: faction · house color · spawn (player-slot attrs + engine notes) ----
    CncProbe._lobbyAttrDebounceMs = 400;
    CncProbe._lobbyAttrTimer = null;
    CncProbe._lobbyAttrPending = null;
    CncProbe._flushLobbyAttr = function () {
        CncProbe._lobbyAttrTimer = null;
        var p = CncProbe._lobbyAttrPending;
        CncProbe._lobbyAttrPending = null;
        if (!p) {
            return;
        }
        var pid = CncProbe.lobbyActivePlayerId();
        if (!pid) {
            CncProbe.log('No playerID — host slot uses profile; AI slot needs Add AI first.');
            return;
        }
        CncProbe.runBlaze('attribute', {
            gameID: CncProbe.blazeShellInput('cnc-probe-blaze-attr-gid', '1'),
            playerID: pid,
            key: p.key,
            value: p.value
        });
    };
    CncProbe.blazeShellSetLobbyAttr = function (key, value) {
        CncProbe._lobbyAttrPending = { key: key, value: String(value) };
        if (CncProbe._lobbyAttrTimer) {
            clearTimeout(CncProbe._lobbyAttrTimer);
        }
        CncProbe._lobbyAttrTimer = setTimeout(CncProbe._flushLobbyAttr, CncProbe._lobbyAttrDebounceMs);
    };
    CncProbe.lobbyInput = function (id, fallback) {
        var el = document.getElementById(id);
        var v = el ? String(el.value || '').trim() : '';
        return v !== '' ? v : (fallback !== undefined ? String(fallback) : '');
    };
    CncProbe.lobbySetFaction = function (faction) {
        CncProbe.blazeShellSetLobbyAttr('_faction', faction);
    };
    CncProbe.lobbySetStartpoint = function (n) {
        CncProbe.blazeShellSetLobbyAttr('_startpoint', n);
    };
    CncProbe.lobbySetIsAi = function (on) {
        CncProbe.blazeShellSetLobbyAttr('_isai', on ? '1' : '0');
    };
    CncProbe.lobbySetTeam = function (team) {
        CncProbe.blazeShellSetLobbyAttr('_team', team);
    };
    CncProbe.lobbyApplyFromForm = function () {
        CncProbe.lobbySetFaction(CncProbe.lobbyInput('cnc-probe-lobby-faction', 'USA'));
        setTimeout(function () {
            CncProbe.lobbySetStartpoint(CncProbe.lobbyInput('cnc-probe-lobby-start', '1'));
        }, 200);
        setTimeout(function () {
            CncProbe.lobbySetTeam(CncProbe.lobbyInput('cnc-probe-lobby-team', '1'));
        }, 400);
        setTimeout(function () {
            CncProbe.lobbySetIsAi(CncProbe.lobbyInput('cnc-probe-lobby-isai', '0') === '1');
        }, 600);
    };
    CncProbe.lobbyPresetHumanUsa = function () {
        CncProbe.log('Lobby preset: human USA, team 1, startpoint 1 (host slot)');
        CncProbe.lobbyTargetHost();
        CncProbe.lobbySetFaction('USA');
        setTimeout(function () { CncProbe.lobbySetStartpoint('1'); }, 200);
        setTimeout(function () { CncProbe.lobbySetTeam('1'); }, 400);
        setTimeout(function () { CncProbe.lobbySetIsAi(false); }, 600);
    };
    CncProbe.lobbyPresetAiEsc = function () {
        CncProbe.log('Lobby preset: AI ESC, team 2, startpoint 2 (AI slot — Add AI first)');
        CncProbe.lobbyTargetAi();
        if (!CncProbe.lobbyActivePlayerId()) {
            return;
        }
        CncProbe.lobbySetFaction('ESC');
        setTimeout(function () { CncProbe.lobbySetStartpoint('2'); }, 200);
        setTimeout(function () { CncProbe.lobbySetTeam('2'); }, 400);
        setTimeout(function () { CncProbe.lobbySetIsAi(true); }, 600);
    };
    /** gameclient: blazeSetPlayerAttribute <gameId> <playerId> <key> <value> */
    CncProbe.rtBlazeSetPlayerAttrFull = function (key, value) {
        if (CncProbe._lobbyAttrTimer) {
            clearTimeout(CncProbe._lobbyAttrTimer);
            CncProbe._lobbyAttrTimer = null;
            CncProbe._lobbyAttrPending = null;
        }
        var gid = CncProbe.blazeShellInput('cnc-probe-blaze-attr-gid', '1');
        var pid = CncProbe.lobbyActivePlayerId();
        if (!pid) {
            CncProbe.log('playerID required — host slot uses profile; AI slot needs Add AI first.');
            return;
        }
        CncProbe.runGame(
            'RtsClient.blazeSetPlayerAttribute ' + gid + ' ' + pid + ' ' + key + ' ' + value
        );
    };
    CncProbe.lobbySetFactionConsole = function (faction) {
        CncProbe.rtBlazeSetPlayerAttrFull('_faction', faction);
    };
    CncProbe.lobbyTryStartpointCli = function () {
        var n = CncProbe.lobbyInput('cnc-probe-lobby-start', '1');
        CncProbe.log('Experimental CLI startpoint (cfg/Level option, not Blaze): startpoint ' + n);
        CncProbe.runGame('startpoint ' + n);
    };
    CncProbe.lobbyLogSpawnEngineMessages = function () {
        CncProbe.log(
            'In-match spawn selection (engine messages — no Blaze shell route):\n' +
            '  ServerPlayer.ManuallySelectedSpawnPoint — player picked a start on the map\n' +
            '  Network.SelectSpawnGroup — UI/network spawn group selection\n' +
            '  Network.SpawnOnSelected — confirm spawn at selected group\n\n' +
            'Pre-game lobby slot index: use _startpoint via Blaze attribute above (1-based).\n' +
            'HouseColorSelectorWinProc fires RtsBlaze.AddHouseColor internally; shell has no addHouseColor route.'
        );
    };

    /** event/game — internal Blaze game event hook (cmd 12 when payload present) */
    CncProbe.blazeShellEventGame = function () {
        CncProbe.runBlaze('event/game');
    };
    /** event/player — internal Blaze player event hook (cmd 13 when payload present) */
    CncProbe.blazeShellEventPlayer = function () {
        CncProbe.runBlaze('event/player');
    };

    // ---- shellaccesslayer (non-Blaze routes) ----
    CncProbe.getConfig = function () { CncProbe.runShell({ _response: CncProbe.shellResult, url: '/config/' }); };
    CncProbe.getOptions = function () { CncProbe.runShell({ _response: CncProbe.shellResult, url: '/options/graphics/get' }); };
    CncProbe.frontEndFullscreen = function (on) {
        CncProbe.runShell({ _resource: '/usersettings/apply', shellfullscreen: !!on });
    };
    CncProbe.gameFullscreen = function (on) {
        CncProbe.runShell({ _resource: '/usersettings/apply', gamefullscreen: !!on });
    };
    CncProbe.setFullscreenDimensions = function (w, h) {
        CncProbe.runShell({
            _resource: '/usersettings/apply',
            fullscreenwidth: w || 1920,
            fullscreenheight: h || 1080
        });
    };
    CncProbe.setWindowedDimensions = function (w, h) {
        CncProbe.runShell({
            _resource: '/usersettings/apply',
            windowedwidth: w || 1920,
            windowedheight: h || 1080
        });
    };
    CncProbe.applyAudio = function () { CncProbe.runShell({ _resource: '/usersettings/applyAudio' }); };
    CncProbe.saveSettings = function () { CncProbe.runShell({ _resource: '/usersettings/save' }); };
    CncProbe.discardSettings = function () { CncProbe.runShell({ _resource: '/usersettings/discard' }); };
    CncProbe.sessionQuit = function () { CncProbe.runShell({ _resource: '/session/quit' }); };
    CncProbe.sessionSurrender = function () { CncProbe.runShell({ _resource: '/session/surrender' }); };

    // ---- Salamander (_module: salamander / _resource: attribute) ----
    CncProbe.salamanderInput = function (id, fallback) {
        var el = document.getElementById(id);
        var v = el ? String(el.value || '').trim() : '';
        return v !== '' ? v : (fallback !== undefined ? String(fallback) : '');
    };
    CncProbe.runSalamanderAttribute = function (playerID, key, value) {
        CncProbe.runShell({
            _module: 'salamander',
            _resource: 'attribute',
            _response: CncProbe.shellResult,
            playerID: playerID,
            key: key,
            value: value
        });
    };
    CncProbe.salamanderAttribute = function () {
        CncProbe.runSalamanderAttribute(
            CncProbe.salamanderInput('cnc-probe-salamander-pid', '999'),
            CncProbe.salamanderInput('cnc-probe-salamander-key', 'GameReady'),
            CncProbe.salamanderInput('cnc-probe-salamander-val', 'localhost')
        );
    };
    CncProbe.salamanderGameReady = function () {
        var host = CncProbe.salamanderInput('cnc-probe-salamander-val', 'localhost');
        CncProbe.runSalamanderAttribute(
            CncProbe.salamanderInput('cnc-probe-salamander-pid', '999'),
            'GameReady',
            host
        );
    };
    CncProbe.salamanderGameReadyAndCreate = function () {
        var host = CncProbe.salamanderInput('cnc-probe-salamander-val', 'localhost');
        CncProbe.log(
            'GameReady + createGame (no join)\n' +
            'joinGame is omitted — it requires a listening game server, not just Blaze/shell.'
        );
        CncProbe.runSalamanderAttribute(
            CncProbe.salamanderInput('cnc-probe-salamander-pid', '999'),
            'GameReady',
            host
        );
        setTimeout(function () { CncProbe.rtCreateLocal(); }, 350);
    };
    CncProbe.salamanderGameReadyCreateAndJoin = function () {
        var host = CncProbe.salamanderInput('cnc-probe-salamander-val', 'localhost');
        CncProbe.log(
            'GameReady → createGame → joinGame ' + host + '\n' +
            'If nothing listens for that join, the client will fatal ("Unable to connect to the server").'
        );
        CncProbe.runSalamanderAttribute(
            CncProbe.salamanderInput('cnc-probe-salamander-pid', '999'),
            'GameReady',
            host
        );
        setTimeout(function () { CncProbe.rtCreateLocal(); }, 350);
        setTimeout(function () { CncProbe.rtJoinLocal(host); }, 800);
    };

    CncProbe.startWs = function (port) {
        port = port || 9000;
        if (!CncProbe.hasShell() || !shellaccesslayer.startWebsocketListener) {
            CncProbe.log('startWebsocketListener not available.');
            return;
        }
        try {
            shellaccesslayer.startWebsocketListener(function (ev) { CncProbe.log('WS: ' + (ev && ev.data !== undefined ? ev.data : String(ev))); }, port);
            CncProbe.log('WebSocketListener started on ' + port);
        } catch (e) {
            CncProbe.log('WS error: ' + e);
        }
    };
    CncProbe.toggleInspector = function () {
        if (CncProbe.hasShell() && shellaccesslayer.toggleInspector) { shellaccesslayer.toggleInspector(); }
    };
    CncProbe.inspectBridgeObjects = function () {
        var safeKeys = function (obj) {
            if (!obj) { return []; }
            try { return Object.getOwnPropertyNames(obj).sort(); } catch (e) { return []; }
        };
        var ctorNames = [];
        try {
            var globals = Object.getOwnPropertyNames(window);
            for (var i = 0; i < globals.length; i++) {
                var n = globals[i];
                if (typeof window[n] === 'function' && /^[A-Z]/.test(n)) {
                    ctorNames.push(n);
                }
            }
            ctorNames.sort();
        } catch (e2) { ctorNames = []; }
        CncProbe.log(
            'BRIDGE INSPECT\n\n' +
            'shellaccesslayer methods:\n' + safeKeys(window.shellaccesslayer).join(', ') + '\n\n' +
            'gameclient methods:\n' + safeKeys(window.gameclient).join(', ') + '\n\n' +
            'constructor/class-like globals (sample):\n' + ctorNames.slice(0, 200).join(', ')
        );
    };
    CncProbe.inspectShellCallbackContract = function () {
        if (!CncProbe.hasShell()) {
            CncProbe.log('shellaccesslayer not available.');
            return;
        }
        CncProbe.runShell({
            url: '/blaze/games',
            _response: function (res) {
                var typeName = Object.prototype.toString.call(res);
                var keys = [];
                try { keys = Object.keys(res || {}); } catch (e) { keys = []; }
                CncProbe.log(
                    'CALLBACK CONTRACT\n\n' +
                    'typeof response: ' + (typeof res) + '\n' +
                    'toString tag: ' + typeName + '\n' +
                    'keys: ' + keys.join(', ') + '\n\n' +
                    'payload:\n' + (function () { try { return JSON.stringify(res, null, 2); } catch (x) { return String(res); } })()
                );
            }
        });
    };

    // ---- gameclient: RtsClient (Blaze bridge from wiki + FB list) ----
    CncProbe.probeGid = function () {
        var el = document.getElementById('cnc-probe-rt-gid');
        var v = el ? parseInt(el.value, 10) : 1;
        return (isNaN(v) || v < 1) ? 1 : v;
    };
    CncProbe.rtQuit = function () { CncProbe.runGame('RtsClient.quit'); };
    CncProbe.rtVersion = function () { CncProbe.runGame('RtsClient.version'); };
    CncProbe.rtIsConnected = function () { CncProbe.runGame('RtsClient.isConnectedToServer'); };
    CncProbe.rtTestEvaluatorPerformance = function () { CncProbe.runGame('RtsClient.testEvaluatorPerformance'); };
    CncProbe.rtSetCameraLookAt = function () {
        var x = document.getElementById('cnc-probe-cam-x');
        var y = document.getElementById('cnc-probe-cam-y');
        var z = document.getElementById('cnc-probe-cam-z');
        var sx = x ? (x.value || '0').trim() : '0';
        var sy = y ? (y.value || '0').trim() : '0';
        var sz = z ? (z.value || '0').trim() : '0';
        CncProbe.runGame('RtsClient.setCameraLookAt ' + sx + ' ' + sy + ' ' + sz);
    };
    CncProbe.rtSaveProfileSettings = function () { CncProbe.runGame('RtsClient.saveProfileSettings'); };
    CncProbe.rtEnableAutoCam = function () { CncProbe.runGame('RtsClient.enableAutoCam'); };
    CncProbe.rtJoinGameFromInputs = function () {
        var gid = CncProbe.probeGid();
        var h = document.getElementById('cnc-probe-join-host');
        var host = h ? (h.value || 'localhost').trim() : 'localhost';
        CncProbe.runGame('RtsClient.joinGame ' + gid + ' ' + host);
    };
    CncProbe.rtBlazeCreate = function (gname, n, level) {
        CncProbe.markBlazeCreatePending();
        var elG = document.getElementById('cnc-probe-blaze-gname');
        var elN = document.getElementById('cnc-probe-blaze-players');
        var elL = document.getElementById('cnc-probe-create-level');
        if (gname === undefined || gname === null) {
            gname = elG ? String(elG.value || '').trim() : '';
        }
        if (!gname) {
            gname = 'XEVRAC';
        }
        if (n === undefined || n === null) {
            n = elN ? String(elN.value || '').trim() : '';
        }
        if (n === '' || n === undefined) {
            n = '1';
        }
        if (level === undefined || level === null) {
            var elBlazeLevel = document.getElementById('cnc-probe-blaze-level');
            if (elBlazeLevel && String(elBlazeLevel.value || '').trim()) {
                level = String(elBlazeLevel.value || '').trim();
            } else {
                level = elL ? String(elL.value || '').trim() : '';
            }
        }
        if (!level) {
            level = 'levels/SP/Alpha_Tutorial/Alpha_Tutorial';
        }
        CncProbe.runGame('RtsClient.blazeCreateGame ' + gname + ' ' + n + ' ' + level);
    };
    CncProbe.rtBlazeJoin = function (gid) {
        if (gid === undefined || gid === null) {
            gid = CncProbe.probeGid();
        }
        CncProbe.runGame('RtsClient.blazeJoinGame ' + gid);
    };
    CncProbe.rtBlazeJoinFromInput = function () { CncProbe.rtBlazeJoin(); };
    CncProbe.rtBlazeGetGames = function () { CncProbe.runGame('RtsClient.blazeGetGames'); };
    CncProbe.rtBlazeGetPlayers = function (gid) {
        gid = (gid !== undefined && gid !== null) ? gid : CncProbe.probeGid();
        CncProbe.runGame('RtsClient.blazeGetPlayers ' + gid);
    };
    CncProbe.rtBlazeGetPlayersFromInput = function () { CncProbe.rtBlazeGetPlayers(); };
    CncProbe.rtBlazeLogin = function () { CncProbe.runGame('RtsClient.blazeLogin'); };
    CncProbe.rtBlazeSetProto = function (v) {
        v = v || '3.19.4.0';
        CncProbe.runGame('RtsClient.blazeSetProtocolVersion ' + v);
    };
    CncProbe.rtBlazeSetProtoFromInput = function () {
        var el = document.getElementById('cnc-probe-blaze-proto');
        var v = el ? (el.value || '').trim() : '';
        CncProbe.rtBlazeSetProto(v || '3.19.4.0');
    };
    CncProbe.rtBlazeSetAttr = function (k, v) {
        k = k || 'testKey';
        v = v || 'testVal';
        CncProbe.runGame('RtsClient.blazeSetPlayerAttribute ' + k + ' ' + v);
    };
    CncProbe.rtGetHouseColors = function () { CncProbe.runGame('RtsClient.getSelectableHouseColors'); };
    CncProbe.rtEndGame = function () {
        if (window.CncPreLanding && CncPreLanding.scheduleReturnFromMatch) {
            CncPreLanding.scheduleReturnFromMatch();
        }
        CncProbe.runGame('RtsClient.endGame');
    };
    CncProbe.rtSurrender = function () { CncProbe.runGame('RtsClient.surrenderGame'); };
    /** Matches Frostbite console: RtsClient.createGame <fullConfigPath> <levelName> [playerId] */
    CncProbe.rtCreateGameLine = function () {
        var cfgEl = document.getElementById('cnc-probe-create-cfg');
        var lvlEl = document.getElementById('cnc-probe-create-level');
        var pidEl = document.getElementById('cnc-probe-create-playerid');
        var cfgPath = cfgEl ? (cfgEl.value || '').trim() : '';
        var levelName = lvlEl ? (lvlEl.value || '').trim() : '';
        var playerId = pidEl ? (pidEl.value || '').trim() : '1';
        if (!cfgPath || !levelName) {
            return null;
        }
        return 'RtsClient.createGame ' + cfgPath + ' ' + levelName + (playerId !== '' ? ' ' + playerId : '');
    };
    CncProbe.rtCreateLocal = function () {
        var line = CncProbe.rtCreateGameLine();
        if (!line) {
            CncProbe.log(
                'createGame: set config path + level name.\n' +
                'Example: C:\\CNCO\\level.cfg  levels/SP/Alpha_Tutorial/Alpha_Tutorial  1'
            );
            return;
        }
        CncProbe.runGame(line);
        CncProbe.log(
            'createGame sent. Black screen + WaitingForLevel is normal until something completes the session.\n' +
            'Use Salamander → GameReady + createGame if the level does not start.'
        );
    };
    CncProbe.rtJoinLocal = function (host) {
        host = host || CncProbe.salamanderInput('cnc-probe-salamander-val', 'localhost');
        CncProbe.runGame('RtsClient.joinGame 1 ' + host);
    };

    CncProbe.joinMultiplayer = function () { CncProbe.runGame('client.joinMultiplayer'); };
    CncProbe.clientDisconnect = function () { CncProbe.runGame('client.disconnect'); };
    CncProbe.testClientJoinMultiplayer = function () {
        CncProbe.log('test → client.joinMultiplayer');
        CncProbe.joinMultiplayer();
    };
    CncProbe.testClientDisconnect = function () {
        CncProbe.log('test → client.disconnect');
        CncProbe.clientDisconnect();
    };

    // ---- Render / debug (low risk) ----
    CncProbe.renderFps = function (on) { CncProbe.runGame('Render.DrawFps ' + (on ? 'true' : 'false')); };
    CncProbe.renderInfo = function (on) { CncProbe.runGame('Render.DrawInfo ' + (on ? 'true' : 'false')); };
    CncProbe._drawScreenInfoEnabled = false;
    CncProbe.toggleDrawScreenInfo = function () {
        CncProbe._drawScreenInfoEnabled = !CncProbe._drawScreenInfoEnabled;
        var on = CncProbe._drawScreenInfoEnabled;
        CncProbe.runGame('Render.DrawInfo ' + (on ? 'true' : 'false'));
        CncProbe.runGame('Render.DrawFpsHistogram ' + (on ? 'true' : 'false'));
        CncProbe.runGame('Render.DrawScreenInfo ' + (on ? 'true' : 'false'));
    };
    CncProbe._fpsIncreaseEnabled = false;
    CncProbe.toggleFpsIncrease = function () {
        CncProbe._fpsIncreaseEnabled = !CncProbe._fpsIncreaseEnabled;
        if (CncProbe._fpsIncreaseEnabled) {
            CncProbe.runGame('GameTime.MaxSimFps 60');
            CncProbe.runGame('GameTime.MaxVariableFps 60');
            CncProbe.runGame('GameTime.MaxInactiveVariableFps 60');
        } else {
            CncProbe.runGame('GameTime.MaxSimFps 30');
            CncProbe.runGame('GameTime.MaxVariableFps 30');
            CncProbe.runGame('GameTime.MaxInactiveVariableFps 30');
        }
    };
    CncProbe.coreLog = function (lvl) { lvl = lvl || 'Debug'; CncProbe.runGame('Core.LogLevel ' + lvl); };
    CncProbe.onlineBackend = function () { CncProbe.runGame('Online.Name'); };
    CncProbe.rtsUseBlaze = function () { CncProbe.runGame('RtsClientSettings.UseBlaze'); };

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function () {
            CncProbe.initOutputModal();
            CncProbe.initLobbyPids();
        });
    } else {
        CncProbe.initOutputModal();
        CncProbe.initLobbyPids();
    }

    window.CncProbe = CncProbe;
})(window);
