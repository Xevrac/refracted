/**
 * In-game chat panel. Empty Enter hides. Esc hides.
 */
var CCApp = angular.module('CCApp', []);

CCApp.controller('IngameChatController', function ($scope, $timeout) {
    $scope.channel = 'all';
    $scope.draft = '';
    $scope.messages = [];
    $scope.playerName = 'You';

    var chatUiOpen = false;
    var ignoreEnterUntil = 0;

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

    function chatInputEl() {
        return document.getElementById('chat-input');
    }

    function focusChatInput() {
        var input = chatInputEl();
        if (!input) {
            return;
        }
        try {
            if (input.setActive) {
                input.setActive();
            }
            input.focus();
        } catch (e) { /* ignore */ }
    }

    function releaseChatFocus() {
        try {
            var input = chatInputEl();
            if (input) {
                input.blur();
            }
            if (document.activeElement && document.activeElement.blur) {
                document.activeElement.blur();
            }
        } catch (e) { /* ignore */ }
    }

    function markOpened() {
        chatUiOpen = true;
        ignoreEnterUntil = Date.now() + 400;
    }

    function scheduleFocus() {
        markOpened();
        $timeout(focusChatInput, 0);
        $timeout(focusChatInput, 40);
        $timeout(focusChatInput, 120);
    }

    $scope.playerName = resolvePlayerName();

    $scope.setChannel = function (ch) {
        if (ch !== 'all' && ch !== 'team') {
            return;
        }
        $scope.channel = ch;
        runGame('ServerPlayer.ChangeChatChannel ' + (ch === 'team' ? '1' : '0'));
        $timeout(focusChatInput, 0);
    };

    $scope.closeChat = function () {
        chatUiOpen = false;
        releaseChatFocus();
        runGame('SetChatVisibility false');
    };

    $scope.openChat = function () {
        markOpened();
        runGame('SetChatVisibility true');
        scheduleFocus();
    };

    $scope.send = function () {
        if (Date.now() < ignoreEnterUntil) {
            return;
        }
        var input = chatInputEl();
        var raw = (input && typeof input.value === 'string') ? input.value : ($scope.draft || '');
        var text = String(raw).replace(/^\s+|\s+$/g, '');
        if (!text) {
            $scope.closeChat();
            return;
        }
        text = text.replace(/[\r\n\t]/g, ' ').replace(/"/g, "'");
        if (text.length > 180) {
            text = text.substring(0, 180);
        }

        $scope.playerName = resolvePlayerName();
        pushLine($scope.playerName, text, 'self');
        $scope.draft = '';
        if (input) {
            input.value = '';
        }

        runGame('ServerPlayer.Chat ' + text);
        ignoreEnterUntil = Date.now() + 400;
        $timeout(focusChatInput, 0);
    };

    try {
        document.addEventListener('keydown', function (ev) {
            var key = ev.keyCode || ev.which;
            if (key === 27) {
                if (ev.preventDefault) {
                    ev.preventDefault();
                }
                $scope.$apply(function () {
                    $scope.closeChat();
                });
                return;
            }
            if (key === 13) {
                if (Date.now() < ignoreEnterUntil) {
                    if (ev.preventDefault) {
                        ev.preventDefault();
                    }
                    return;
                }
                if (!chatUiOpen) {
                    if (ev.preventDefault) {
                        ev.preventDefault();
                    }
                    if (ev.stopPropagation) {
                        ev.stopPropagation();
                    }
                    $scope.$apply(function () {
                        $scope.openChat();
                    });
                    return;
                }
            }
        }, true);
        window.addEventListener('focus', function () {
            scheduleFocus();
        }, false);
    } catch (e) { /* ignore */ }

    $timeout(function () {
        var input = chatInputEl();
        if (!input) {
            return;
        }
        input.addEventListener('focus', function () {
            if (!chatUiOpen) {
                markOpened();
            }
        });
    }, 0);
});
