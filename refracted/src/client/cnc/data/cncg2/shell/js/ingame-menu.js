/**

 * In-game pause menu (WebPathIngameMenu).

 *   SetPauseMenuVisibility false  — close pause UI

 *   RtsClient.surrenderGame       — surrender → return to menu

 *   quit true                     — Quit Game → close process (desktop)

 */

var CCApp = angular.module('CCApp', []);



CCApp.controller('IngameMenuController', function ($scope, $rootScope) {

    /* Options UI mirrors shell theme; pause chrome stays Aurora-styled via CSS. */
    $rootScope.allowShellThemeSelect = false;

    (function syncIngameOptionsTheme() {
        var id = 'aurora';
        try {
            if (window.CncShellTheme && typeof CncShellTheme.get === 'function') {
                id = CncShellTheme.get() || 'aurora';
            }
        } catch (e) { /* ignore */ }
        if (id !== 'classic' && id !== 'aurora') {
            id = 'aurora';
        }
        var body = document.body;
        if (!body) {
            return;
        }
        body.className = String(body.className || '')
            .replace(/\bcc-theme--classic\b/g, '')
            .replace(/\bcc-theme--cnc-alpha\b/g, '')
            .replace(/\bcc-theme--aurora\b/g, '')
            .replace(/\bingame-aurora\b/g, '')
            .replace(/\bingame-classic\b/g, '')
            .replace(/\s+/g, ' ')
            .trim();
        if ((' ' + body.className + ' ').indexOf(' cc-theme ') === -1) {
            body.className = (body.className ? body.className + ' ' : '') + 'cc-theme';
        }
        body.className += ' cc-theme--' + id + ' ingame-' + id;
    })();

    $scope.optionsOpen = false;
    $scope.confirmQuitOpen = false;
    $scope.confirmSurrenderOpen = false;
    $scope.quittingOpen = false;



    var QUIT_BLAZE_STEP_MS = 2500;

    var QUIT_POST_BLAZE_BUFFER_MS = 350;



    function runGame(line) {

        try {

            if (typeof gameclient !== 'undefined' && gameclient && typeof gameclient.execute === 'function') {

                gameclient.execute(line);

            }

        } catch (e) { /* ignore */ }

    }



    function clearConfirms() {

        $scope.confirmQuitOpen = false;

        $scope.confirmSurrenderOpen = false;

    }



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

        if (window.CncProbe) {

            CncProbe._inBlazeGame = false;

        }

    }



    function hasShell() {

        return typeof shellaccesslayer !== 'undefined' && shellaccesslayer

            && typeof shellaccesslayer.execute === 'function';

    }



    function resolveLocalPersonaId() {

        var pid = 0;

        if (window.CncBlazeState && CncBlazeState.getPersonaId) {

            try {

                pid = Number(CncBlazeState.getPersonaId()) || 0;

            } catch (e) { /* ignore */ }

        }

        if (!pid) {

            pid = Number(readSession('cnc_match_pid')) || 0;

        }

        if (!pid && window.__CNC_PROFILE && window.__CNC_PROFILE.personaId) {

            pid = Number(window.__CNC_PROFILE.personaId) || 0;

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



    /** Quit to desktop after removePlayer + logout. */

    function quitToDesktopAfterBlazeCleanup() {

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



    $scope.openOptions = function () {

        clearConfirms();

        $scope.optionsOpen = true;

    };



    $scope.closeOptions = function () {

        $scope.optionsOpen = false;

    };



    $scope.returnToGame = function () {

        clearConfirms();

        $scope.optionsOpen = false;

        runGame('SetPauseMenuVisibility false');

    };



    $scope.openConfirmSurrender = function () {

        $scope.confirmQuitOpen = false;

        $scope.confirmSurrenderOpen = true;

    };



    $scope.openConfirmQuit = function () {

        $scope.confirmSurrenderOpen = false;

        $scope.confirmQuitOpen = true;

    };



    $scope.cancelConfirm = function () {

        clearConfirms();

    };



    $scope.confirmSurrenderYes = function () {

        clearConfirms();

        runGame('RtsClient.surrenderGame');

        runGame('SetPauseMenuVisibility false');

    };



    $scope.confirmQuitYes = function () {
        $scope.confirmQuitOpen = false;
        $scope.confirmSurrenderOpen = false;
        $scope.optionsOpen = false;
        $scope.quittingOpen = true;

        // removePlayer + logout must finish before quit true.
        quitToDesktopAfterBlazeCleanup();
    };

});


