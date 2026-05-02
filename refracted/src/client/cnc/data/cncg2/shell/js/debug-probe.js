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
                CncProbe.log('RESPONSE: ' + JSON.stringify(res, null, 2));
            } catch (e) {
                CncProbe.log('RESPONSE: ' + String(res));
            }
            if (window.CncBlazeState) CncBlazeState.onShellResult(res);
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

    // ---- HTTP / shellaccesslayer (Blaze & session) ----
    CncProbe.blazeBase = function () { CncProbe.runShell({ _response: CncProbe.shellResult, url: '/blaze' }); };
    CncProbe.blazeAuth = function (email, password) {
        email = email || 'user@example.com';
        password = password || 'test';
        CncProbe.runShell({ _response: CncProbe.shellResult, url: '/blaze/authenticate?email=' + encodeURIComponent(email) + '&password=' + encodeURIComponent(password) });
    };
    CncProbe.blazeLogout = function () {
        var completed = false;
        CncProbe.log('Attempting logout via /blaze/logout (url style)...');
        CncProbe.runShell({
            url: '/blaze/logout',
            _response: function (res) {
                completed = true;
                CncProbe.shellResult(res);
            }
        });
        setTimeout(function () {
            if (completed) {
                return;
            }
            CncProbe.log('No callback yet. Retrying logout via _resource style...');
            CncProbe.runShell({
                _resource: '/blaze/logout',
                _response: function (res) {
                    completed = true;
                    CncProbe.shellResult(res);
                }
            });
            setTimeout(function () {
                if (!completed) {
                    CncProbe.log(
                        'Logout probe did not return a callback in this shell context.\n' +
                        'Likely causes: route not implemented or callback suppressed by EAWebKit bridge.'
                    );
                }
            }, 1400);
        }, 1400);
    };
    CncProbe.blazeCreate = function (name, players) {
        name = name || 'XEVRAC';
        players = players || 4;
        CncProbe.runShell({ _response: CncProbe.shellResult, url: '/blaze/createGame?gameName=' + encodeURIComponent(name) + '&players=' + players });
    };
    CncProbe.blazeJoin = function (gid) {
        gid = gid || 1;
        CncProbe.runShell({ _response: CncProbe.shellResult, url: '/blaze/joinGame?gameID=' + encodeURIComponent(gid) });
    };
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
    CncProbe.salamanderGameReady = function (host) {
        host = host || 'localhost';
        CncProbe.runShell({
            _response: CncProbe.shellResult,
            _resource: '/salamander/attribute',
            playerID: '999',
            key: 'GameReady',
            value: host
        });
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
            _resource: '/blaze',
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
    CncProbe.rtBlazeCreateUrl = function (gname, n) {
        var elG = document.getElementById('cnc-probe-blaze-gname');
        var elN = document.getElementById('cnc-probe-blaze-players');
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
            n = '4';
        }
        var url = '/blaze/createGame?gameName=' + encodeURIComponent(gname) + '&players=' + encodeURIComponent(n);
        CncProbe.log('blazeCreateGame (URL) -> ' + url);
        CncProbe.runShell({ url: url });
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
    CncProbe.rtEndGame = function () { CncProbe.runGame('RtsClient.endGame'); };
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
            'Experimental: use GameReady + createGame (no TCP join). Do not use joinGame unless a real game server is listening.'
        );
    };
    CncProbe.rtJoinLocal = function (host) {
        host = host || 'localhost';
        CncProbe.runGame('RtsClient.joinGame 1 ' + host);
    };
    /**
     * Salamander GameReady + createGame only. Safe with Refracted: does not call RtsClient.joinGame
     * (joinGame opens a TCP game session; the emulator is not that host — it fatals with "Unable to connect to the server").
     */
    CncProbe.localSalamanderGameReadyAndCreate = function (host) {
        host = host || 'localhost';
        CncProbe.log(
            'GameReady + createGame (no join)\n' +
            'joinGame is omitted — it requires a listening game server, not just Blaze/shell.'
        );
        CncProbe.runShell({
            _response: CncProbe.shellResult,
            _resource: '/salamander/attribute',
            playerID: '999',
            key: 'GameReady',
            value: host
        });
        setTimeout(function () { CncProbe.rtCreateLocal(); }, 350);
    };
    /** Same as above then RtsClient.joinGame — only if a real game host is running or you accept a fatal disconnect dialog. */
    CncProbe.localHostAndJoinWithGameServer = function (host) {
        host = host || 'localhost';
        CncProbe.log(
            'FULL: GameReady → createGame → joinGame ' + host + '\n' +
            'If nothing listens for that join, the client will fatal ("Unable to connect to the server").'
        );
        CncProbe.runShell({
            _response: CncProbe.shellResult,
            _resource: '/salamander/attribute',
            playerID: '999',
            key: 'GameReady',
            value: host
        });
        setTimeout(function () { CncProbe.rtCreateLocal(); }, 350);
        setTimeout(function () { CncProbe.rtJoinLocal(host); }, 800);
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
        document.addEventListener('DOMContentLoaded', function () { CncProbe.initOutputModal(); });
    } else {
        CncProbe.initOutputModal();
    }

    window.CncProbe = CncProbe;
})(window);
