/**
 * In-game chat (WebPathIngameChat → ChatWinProc).
 *
 *   ServerPlayer.Chat              → ServerPlayerChatMessage (GameServer net msg)
 *   ServerPlayer.ChangeChatChannel → channel switch before send
 *   UI.HudChat                    → UIHudChatMessage (HUD display event)
 *   shell _module:origin /chat    → Origin friends chat only (UseOrigin gate)
 *
 * MsgSys Client channel has no Chat type (GeneralTaunt only).
 * Send path: gameclient.execute('ServerPlayer.Chat …'); local echo for UI.
 */
var CCApp = angular.module('CCApp', []);

CCApp.controller('IngameChatController', function ($scope, $timeout) {
    $scope.channel = 'all';
    $scope.draft = '';
    $scope.messages = [];
    $scope.playerName = 'You';

    function runGame(line) {
        try {
            if (typeof gameclient !== 'undefined' && gameclient && typeof gameclient.execute === 'function') {
                gameclient.execute(line);
            }
        } catch (e) { /* ignore */ }
    }

    function scrollHistory() {
        $timeout(function () {
            var el = document.getElementById('chat-history');
            if (el) {
                el.scrollTop = el.scrollHeight;
            }
        }, 0);
    }

    function pushLine(from, text, kind) {
        $scope.messages.push({
            from: from || '',
            text: text || '',
            kind: kind || 'msg',
            channel: $scope.channel
        });
        if ($scope.messages.length > 80) {
            $scope.messages.shift();
        }
        scrollHistory();
    }

    function resolvePlayerName() {
        try {
            if (window.__CNC_PROFILE && window.__CNC_PROFILE.persona) {
                return String(window.__CNC_PROFILE.persona);
            }
            if (window.__CNC_BLAZE && window.__CNC_BLAZE.persona) {
                return String(window.__CNC_BLAZE.persona);
            }
        } catch (e) { /* ignore */ }
        return 'You';
    }

    $scope.playerName = resolvePlayerName();

    $scope.setChannel = function (ch) {
        if (ch !== 'all' && ch !== 'team') {
            return;
        }
        $scope.channel = ch;
        // 0 = all / say-all, 1 = team (BF-style; verify in-game)
        runGame('ServerPlayer.ChangeChatChannel ' + (ch === 'team' ? '1' : '0'));
    };

    $scope.closeChat = function () {
        runGame('SetChatVisibility false');
    };

    $scope.send = function () {
        var text = ($scope.draft || '').replace(/^\s+|\s+$/g, '');
        if (!text) {
            return;
        }
        // Strip control chars; keep console-safe single line
        text = text.replace(/[\r\n\t]/g, ' ').replace(/"/g, "'");
        if (text.length > 180) {
            text = text.substring(0, 180);
        }

        $scope.playerName = resolvePlayerName();
        pushLine($scope.playerName, text, 'self');
        $scope.draft = '';

        // Primary RTS send: Frostbite GameServer networkable message
        runGame('ServerPlayer.Chat ' + text);
    };

    // Esc closes chat (mirrors pause menu return)
    try {
        document.addEventListener('keydown', function (ev) {
            var key = ev.keyCode || ev.which;
            if (key === 27) {
                $scope.$apply(function () {
                    $scope.closeChat();
                });
            }
        }, false);
    } catch (e) { /* ignore */ }

    // Autofocus input when ChatWinProc shows this page
    $timeout(function () {
        var input = document.getElementById('chat-input');
        if (input) {
            try { input.focus(); } catch (e) { /* ignore */ }
        }
    }, 50);
});
