/**
 * In-game pause menu (WebPathIngameMenu).
 * Commands (IDA / probe):
 *   SetPauseMenuVisibility false  — close pause UI
 *   RtsClient.surrenderGame       — surrender
 *   RtsClient.quit                — exit client
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
        runGame('RtsClient.quit');
    };
});
