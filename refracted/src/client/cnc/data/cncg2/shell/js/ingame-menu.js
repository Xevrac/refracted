/**
 * In-game pause menu.
 */

var CCApp = angular.module('CCApp', []);



CCApp.controller('IngameMenuController', function ($scope, $rootScope) {

    /* Options modal matches shell (tabs, buttons, INTERFACE); pause chrome stays ingame CSS. */
    $rootScope.allowShellThemeSelect = true;

    function syncIngameBodyTheme(id) {
        var family = (id === 'classic') ? 'classic' : 'aurora';
        var themeClass = 'cc-theme--' + family;
        if (window.CncShellTheme && typeof CncShellTheme.family === 'function') {
            family = CncShellTheme.family(id);
            themeClass = CncShellTheme.bodyClass(id);
        }
        var body = document.body;
        if (!body) {
            return;
        }
        body.className = String(body.className || '')
            .replace(/\bcc-theme--classic\b/g, '')
            .replace(/\bcc-theme--cnc-alpha\b/g, '')
            .replace(/\bcc-theme--aurora-alt\b/g, '')
            .replace(/\bcc-theme--aurora\b/g, '')
            .replace(/\bingame-aurora\b/g, '')
            .replace(/\bingame-classic\b/g, '')
            .replace(/\s+/g, ' ')
            .trim();
        if ((' ' + body.className + ' ').indexOf(' cc-theme ') === -1) {
            body.className = (body.className ? body.className + ' ' : '') + 'cc-theme';
        }
        body.className += ' ' + themeClass + ' ingame-' + family;
    }

    (function syncIngameOptionsTheme() {
        var id = 'aurora';
        try {
            if (window.CncShellTheme && typeof CncShellTheme.get === 'function') {
                id = CncShellTheme.get() || 'aurora';
            }
        } catch (e) { /* ignore */ }
        syncIngameBodyTheme(id);
        if (window.CncShellTheme && typeof CncShellTheme.onChange === 'function') {
            CncShellTheme.onChange(syncIngameBodyTheme);
        }
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
            CncProbe._matchWatchArmed = false;

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



    var MENU_LEAVE_MS = 300;
    var menuLeaving = false;
    var menuNeedsReplay = false;

    function pausePanel() {
        var nodes = document.getElementsByClassName('pause-menu');
        return (nodes && nodes.length) ? nodes[0] : null;
    }

    function hasClass(el, name) {
        return !!(el && (' ' + el.className + ' ').indexOf(' ' + name + ' ') !== -1);
    }

    function addClass(el, name) {
        if (!el || hasClass(el, name)) {
            return;
        }
        el.className = (el.className ? el.className + ' ' : '') + name;
    }

    function stripClass(el, name) {
        if (!el || !el.className) {
            return;
        }
        el.className = String(el.className)
            .replace(new RegExp('\\b' + name + '\\b', 'g'), ' ')
            .replace(/\s+/g, ' ')
            .replace(/^\s+|\s+$/g, '');
    }

    function motionNodes() {
        var panel = pausePanel();
        var nodes = [];
        var i;
        if (!panel) {
            return nodes;
        }
        nodes.push(panel);
        var titles = panel.getElementsByTagName('h2');
        var buttons = panel.getElementsByTagName('button');
        for (i = 0; i < titles.length; i++) {
            nodes.push(titles[i]);
        }
        for (i = 0; i < buttons.length; i++) {
            nodes.push(buttons[i]);
        }
        return nodes;
    }

    // While the view is hidden the menu waits in the first frame of its entrance: panel off the
    // right edge, title and buttons transparent, veil faded out. The game shows the view before
    // the page hears about it, so whatever is on screen at that moment is what gets painted
    // first. Parked in the finished position, that was a fully open menu for a frame or two,
    // then the slide-in from off-screen: the menu appeared twice.
    function parkMotion() {
        var nodes = motionNodes();
        var i;
        for (i = 0; i < nodes.length; i++) {
            nodes[i].style.webkitAnimation = 'none';
            nodes[i].style.animation = 'none';
            if (i === 0) {
                nodes[i].style.webkitTransform = 'translateX(100%)';
                nodes[i].style.transform = 'translateX(100%)';
            } else {
                nodes[i].style.opacity = '0';
            }
        }
        addClass(document.body, 'pause-leaving');
    }

    function replayPauseEntrance() {
        if (!menuNeedsReplay) {
            return;
        }
        menuNeedsReplay = false;
        menuLeaving = false;
        var body = document.body;
        // Changing the veil's animation name restarts its fade with the panel.
        addClass(body, 'pause-leaving');
        if (body) {
            body.offsetWidth;
        }
        stripClass(body, 'pause-leaving');
        stripClass(pausePanel(), 'is-leaving');
        var nodes = motionNodes();
        var i;
        for (i = 0; i < nodes.length; i++) {
            nodes[i].style.webkitAnimation = 'none';
            nodes[i].style.animation = 'none';
        }
        if (nodes.length) {
            nodes[0].offsetWidth;
        }
        for (i = 0; i < nodes.length; i++) {
            nodes[i].style.webkitTransform = '';
            nodes[i].style.transform = '';
            nodes[i].style.opacity = '';
            nodes[i].style.webkitAnimation = '';
            nodes[i].style.animation = '';
        }
    }

    function hidePauseMenu() {
        if (menuLeaving || $scope.quittingOpen) {
            return;
        }
        menuLeaving = true;
        clearConfirms();
        $scope.optionsOpen = false;
        addClass(pausePanel(), 'is-leaving');
        addClass(document.body, 'pause-leaving');
        setTimeout(function () {
            runGame('SetPauseMenuVisibility false');
            // Park at the start of the entrance (the body keeps .pause-leaving: veil stays out).
            parkMotion();
            stripClass(pausePanel(), 'is-leaving');
            menuLeaving = false;
            menuNeedsReplay = true;
        }, MENU_LEAVE_MS);
    }

    function onPauseShown() {
        replayPauseEntrance();
    }

    document.addEventListener('visibilitychange', function () {
        if (document.visibilityState === 'hidden') {
            // Hidden by the game itself (not through Return to game): park for the next show.
            if (!menuLeaving) {
                parkMotion();
                menuNeedsReplay = true;
            }
            return;
        }
        onPauseShown();
    });
    window.addEventListener('focus', onPauseShown);
    window.addEventListener('pageshow', onPauseShown);
    // A parked menu is off-screen, so it must never miss its entrance: any input reaching the
    // view also plays it (a no-op unless parked).
    document.addEventListener('mousemove', onPauseShown, true);
    document.addEventListener('mousedown', onPauseShown, true);
    document.addEventListener('keydown', onPauseShown, true);

    $scope.returnToGame = function () {
        hidePauseMenu();
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

        hidePauseMenu();

    };



    $scope.confirmQuitYes = function () {
        $scope.confirmQuitOpen = false;
        $scope.confirmSurrenderOpen = false;
        $scope.optionsOpen = false;
        $scope.quittingOpen = true;

        // removePlayer + logout must finish before quit true.
        quitToDesktopAfterBlazeCleanup();
    };

    function isTextField(el) {
        if (!el || !el.tagName) {
            return false;
        }
        var tag = el.tagName.toUpperCase();
        if (tag === 'TEXTAREA') {
            return true;
        }
        if (tag !== 'INPUT') {
            return false;
        }
        var type = (el.type || 'text').toLowerCase();
        return type === 'text' || type === 'password' || type === 'search' || type === 'email' || type === '';
    }

    // Retail Esc toggles the pause window. While it is up, Esc steps out of a
    // dialog first, then closes the menu.
    function handleIngameEscape() {
        if ($scope.quittingOpen) {
            return;
        }
        var el = document.activeElement;
        if (isTextField(el)) {
            el.blur();
            return;
        }
        if ($scope.optionsOpen) {
            $scope.closeOptions();
            return;
        }
        if ($scope.confirmQuitOpen || $scope.confirmSurrenderOpen) {
            $scope.cancelConfirm();
            return;
        }
        $scope.returnToGame();
    }

    var escHeld = false;
    document.addEventListener('keydown', function (ev) {
        var key = ev.keyCode || ev.which;
        if (key !== 27) {
            return;
        }
        if (escHeld) {
            return;
        }
        escHeld = true;
        if (ev.preventDefault) {
            ev.preventDefault();
        }
        $scope.$apply(handleIngameEscape);
    });
    document.addEventListener('keyup', function (ev) {
        var key = ev.keyCode || ev.which;
        if (key === 27) {
            escHeld = false;
        }
    });

});


