/**
 * In-game pause menu (WebPathIngameMenu).
 *   SetPauseMenuVisibility false  — close pause UI
 *   RtsClient.surrenderGame       — surrender
 */
var CCApp = angular.module('CCApp', []);

CCApp.controller('IngameMenuController', function ($scope, $rootScope) {
    /* Main-shell Aurora theme must never style pause / in-game options. */
    $rootScope.allowShellThemeSelect = false;
    $scope.optionsOpen = false;
    $scope.confirmQuitOpen = false;
    $scope.confirmSurrenderOpen = false;

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

    function postLeaveBeforeQuit() {
        if (window.CncPreLanding && CncPreLanding.scheduleReturnFromMatch) {
            CncPreLanding.scheduleReturnFromMatch();
        }
        var gid = readSession('cnc_match_gid') || '1';
        var pid = 0;
        if (window.CncBlazeState && CncBlazeState.getPersonaId) {
            try {
                pid = Number(CncBlazeState.getPersonaId()) || 0;
            } catch (e) { /* ignore */ }
        }
        if (!pid) {
            pid = Number(readSession('cnc_match_pid')) || 0;
        }
        var leaveUrl = '/cnc/leave-game?gid=' + encodeURIComponent(gid) +
            '&pid=' + encodeURIComponent(pid) +
            '&force=1';
        try {
            if (typeof XMLHttpRequest !== 'undefined') {
                var xhr = new XMLHttpRequest();
                xhr.open('POST', leaveUrl, true);
                xhr.send(null);
            }
        } catch (e) { /* best-effort */ }
        if (window.CncProbe && CncProbe.runBlazeUrl && CncProbe.blazeUrlFromResource) {
            try {
                CncProbe.runBlazeUrl(CncProbe.blazeUrlFromResource('removePlayer', {
                    gameID: gid,
                    playerID: pid
                }));
            } catch (e2) { /* shell route may not exist */ }
        }
        clearMatchSession();
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
        clearConfirms();
        if (window.CncPreLanding && CncPreLanding.scheduleReturnFromMatch) {
            CncPreLanding.scheduleReturnFromMatch();
        }
        runGame('RtsClient.EndGame');
        setTimeout(function () {
            postLeaveBeforeQuit();
            runGame('RtsClient.quit');
        }, 400);
    };
});
