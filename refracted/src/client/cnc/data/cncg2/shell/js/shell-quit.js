/**
 * Shared shell quit-to-desktop flow (Blaze cleanup + gameclient quit).
 */
(function (global) {
    'use strict';

    var QUIT_BLAZE_STEP_MS = 2500;
    var QUIT_POST_BLAZE_BUFFER_MS = 350;

    function readSession(key) {
        try {
            return sessionStorage.getItem(key);
        } catch (e) {
            return null;
        }
    }

    function clearMatchSession() {
        try {
            sessionStorage.removeItem('cnc_match_gid');
            sessionStorage.removeItem('cnc_match_pid');
        } catch (e) { /* ignore */ }
        if (global.CncProbe) {
            CncProbe._inBlazeGame = false;
            CncProbe._matchWatchArmed = false;
        }
    }

    function hasShell() {
        return typeof shellaccesslayer !== 'undefined' && shellaccesslayer
            && typeof shellaccesslayer.execute === 'function';
    }

    function resolveLocalPersonaId() {
        var pid = 0;
        if (global.CncBlazeState && CncBlazeState.getPersonaId) {
            try {
                pid = Number(CncBlazeState.getPersonaId()) || 0;
            } catch (e) { /* ignore */ }
        }
        if (!pid) {
            pid = Number(readSession('cnc_match_pid')) || 0;
        }
        if (!pid && global.__CNC_PROFILE && global.__CNC_PROFILE.personaId) {
            pid = Number(global.__CNC_PROFILE.personaId) || 0;
        }
        return pid;
    }

    function blazeUrl(resource, params) {
        params = params || {};
        var key = String(resource || '').trim().toLowerCase();
        var pathByResource = {
            removeplayer: '/blaze/removePlayer',
            logout: '/blaze/logout'
        };
        var base = pathByResource[key] || ('/blaze/' + String(resource || '').replace(/^\/+/, ''));
        var parts = [];
        var k;
        for (k in params) {
            if (Object.prototype.hasOwnProperty.call(params, k)) {
                parts.push(encodeURIComponent(k) + '=' + encodeURIComponent(String(params[k])));
            }
        }
        return parts.length ? (base + '?' + parts.join('&')) : base;
    }

    function runGame(line) {
        try {
            if (typeof gameclient !== 'undefined' && gameclient && typeof gameclient.execute === 'function') {
                gameclient.execute(line);
            }
        } catch (e) { /* ignore */ }
    }

    function runShellUrl(url, cb) {
        if (!hasShell()) {
            if (cb) {
                cb(null);
            }
            return;
        }
        var done = false;
        var finish = function (res) {
            if (done) {
                return;
            }
            done = true;
            if (cb) {
                cb(res);
            }
        };
        var timer = setTimeout(function () { finish(null); }, QUIT_BLAZE_STEP_MS);
        try {
            shellaccesslayer.execute({
                url: url,
                _response: function (res) {
                    clearTimeout(timer);
                    finish(res);
                }
            });
        } catch (e) {
            clearTimeout(timer);
            finish(null);
        }
    }

    function postLeaveGame(gid, pid, cb) {
        var leaveUrl = '/cnc/leave-game?gid=' + encodeURIComponent(gid) +
            '&pid=' + encodeURIComponent(pid || 0) +
            '&force=1';
        var done = false;
        var finish = function () {
            if (done) {
                return;
            }
            done = true;
            if (cb) {
                cb();
            }
        };
        var timer = setTimeout(finish, QUIT_BLAZE_STEP_MS);
        try {
            var xhr = new XMLHttpRequest();
            xhr.open('POST', leaveUrl, true);
            xhr.onload = xhr.onerror = xhr.onabort = xhr.ontimeout = function () {
                clearTimeout(timer);
                finish();
            };
            xhr.timeout = QUIT_BLAZE_STEP_MS - 100;
            xhr.send(null);
        } catch (e) {
            clearTimeout(timer);
            finish();
        }
    }

    function quitToDesktop() {
        var gid = readSession('cnc_match_gid') || '1';
        var pid = resolveLocalPersonaId();
        postLeaveGame(gid, pid, function () {
            runShellUrl(blazeUrl('removePlayer', { gameID: gid, playerID: pid || 0 }), function () {
                runShellUrl(blazeUrl('logout'), function () {
                    clearMatchSession();
                    setTimeout(function () {
                        runGame('quit true');
                    }, QUIT_POST_BLAZE_BUFFER_MS);
                });
            });
        });
    }

    global.CncShellQuit = {
        quitToDesktop: quitToDesktop
    };
}(window));
