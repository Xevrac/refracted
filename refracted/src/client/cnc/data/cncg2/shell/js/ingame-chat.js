/**
 * In-game chat panel. Empty Enter hides. Esc hides.
 * Lines go to the match server (shell route /prism/chat/send -> RequestRPC)
 * (always / active / never).
 */
var CCApp = angular.module('CCApp', []);

CCApp.controller('IngameChatController', function ($scope, $timeout) {
    $scope.channel = 'all';
    $scope.draft = '';
    $scope.messages = [];
    $scope.playerName = '';

    var DEV_NAMES = { 'nemo': true, 'xevrac': true };
    var chatUiOpen = false;
    var ignoreEnterUntil = 0;
    var VISIBILITY_KEY = 'cnc_ingame_chat_visibility';
    var ACTIVE_SHOW_MS = 3000;
    // In-process shell route; a 1 s poll added up to a second to every line.
    var POLL_MS = 100;
    var POLL_STALL_MS = 2000;
    var pollSerial = 0;
    var lastSeq = -1;
    var activeHideTimer = null;
    var shownByActivity = false;

    function visibilityMode() {
        var v = null;
        try {
            v = localStorage.getItem(VISIBILITY_KEY);
        } catch (e) { /* ignore */ }
        return (v === 'always' || v === 'never') ? v : 'active';
    }

    function executeShell(resource, payload, done) {
        try {
            if (typeof shellaccesslayer === 'undefined' || !shellaccesslayer
                || typeof shellaccesslayer.execute !== 'function') {
                return false;
            }
            var req = payload || {};
            req._resource = resource;
            if (done) {
                req._response = function (res) {
                    var data = res;
                    if (typeof data === 'string') {
                        try {
                            data = JSON.parse(data);
                        } catch (e) {
                            data = null;
                        }
                    }
                    $timeout(function () {
                        done(data);
                    }, 0);
                };
            }
            shellaccesslayer.execute(req);
            return true;
        } catch (e) {
            return false;
        }
    }

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

    function isDevName(name) {
        return !!DEV_NAMES[String(name || '').toLowerCase()];
    }

    // All chat: purple for developers (lobby parity). Otherwise viewer-relative
    // ally blue / enemy red / self gold. Team channel keeps its green tint.
    // A spectator's line is grey for every reader, whatever the channel.
    $scope.fromClass = function (m) {
        if (!m || m.kind === 'system') {
            return '';
        }
        if (m.relation === 'o') {
            return 'spectator';
        }
        if (m.channel === 'team') {
            return 'team';
        }
        if (isDevName(m.from)) {
            return 'dev';
        }
        if (m.relation === 's' || m.kind === 'self') {
            return 'self';
        }
        if (m.relation === 'e') {
            return 'enemy';
        }
        return 'ally';
    };

    function pushLine(from, text, kind) {
        $scope.messages.push({
            from: from || '',
            text: text || '',
            kind: kind || 'msg',
            channel: $scope.channel,
            relation: kind === 'self' ? 's' : ''
        });
        if ($scope.messages.length > 80) {
            $scope.messages.shift();
        }
        scrollHistory();
    }

    function nonEmpty(value) {
        if (value == null) {
            return '';
        }
        var s = String(value).replace(/^\s+|\s+$/g, '');
        return s;
    }

    function readLoginKey() {
        try {
            var stored = sessionStorage.getItem('cnc_login_key');
            if (stored) {
                return stored;
            }
        } catch (e) { /* ignore */ }
        try {
            var match = /(?:^|;\s*)cnc_login_key=([0-9a-f]{24})/.exec(document.cookie || '');
            return match ? match[1] : '';
        } catch (e2) {
            return '';
        }
    }

    // DSNM lives on the Refracted roster (Blaze persona), not in this page.
    function resolvePlayerName() {
        if (nonEmpty($scope.playerName) && $scope.playerName !== 'You') {
            return $scope.playerName;
        }
        try {
            var prof = window.__CNC_PROFILE;
            var fromProf = prof && (nonEmpty(prof.displayName) || nonEmpty(prof.DSNM));
            if (fromProf) {
                return fromProf;
            }
            var blaze = window.__CNC_BLAZE;
            var fromBlaze = blaze && (nonEmpty(blaze.displayName) || nonEmpty(blaze.DSNM));
            if (fromBlaze) {
                return fromBlaze;
            }
            var cached = localStorage.getItem('cnc_blaze_dsnm');
            if (nonEmpty(cached)) {
                return nonEmpty(cached);
            }
            var raw = sessionStorage.getItem('cnc_blaze_session');
            if (raw) {
                var parsed = JSON.parse(raw);
                if (parsed && nonEmpty(parsed.displayName)) {
                    return nonEmpty(parsed.displayName);
                }
            }
        } catch (e) { /* ignore */ }
        return '';
    }

    function identityNameSync() {
        var key = readLoginKey();
        if (!key || typeof XMLHttpRequest === 'undefined') {
            return '';
        }
        try {
            var xhr = new XMLHttpRequest();
            xhr.open('GET', '/cnc/identity?key=' + encodeURIComponent(key), false);
            xhr.send();
            if (xhr.status !== 200) {
                return '';
            }
            var data = JSON.parse(xhr.responseText);
            return data && nonEmpty(data.displayName);
        } catch (e) {
            return '';
        }
    }

    function refreshIdentity() {
        var key = readLoginKey();
        if (!key || typeof XMLHttpRequest === 'undefined') {
            $scope.playerName = resolvePlayerName();
            return;
        }
        try {
            var xhr = new XMLHttpRequest();
            xhr.open('GET', '/cnc/identity?key=' + encodeURIComponent(key), true);
            xhr.onreadystatechange = function () {
                if (xhr.readyState !== 4 || xhr.status !== 200) {
                    return;
                }
                var data = null;
                try {
                    data = JSON.parse(xhr.responseText);
                } catch (e) {
                    return;
                }
                var name = data && nonEmpty(data.displayName);
                if (!name) {
                    return;
                }
                $timeout(function () {
                    $scope.playerName = name;
                });
            };
            xhr.send();
        } catch (e) { /* ignore */ }
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
    refreshIdentity();

    // All = messages sent to every player. Team = this team only.
    $scope.messageVisible = function (m) {
        if (!m) {
            return false;
        }
        if (m.kind === 'system') {
            return true;
        }
        if ($scope.channel === 'team') {
            return m.channel === 'team';
        }
        return m.channel !== 'team';
    };

    $scope.hasVisible = function () {
        var i;
        for (i = 0; i < $scope.messages.length; i++) {
            if ($scope.messageVisible($scope.messages[i])) {
                return true;
            }
        }
        return false;
    };

    function hideAfterActivity() {
        if (activeHideTimer) {
            $timeout.cancel(activeHideTimer);
        }
        activeHideTimer = $timeout(function () {
            activeHideTimer = null;
            if (shownByActivity && !chatUiOpen && visibilityMode() === 'active') {
                shownByActivity = false;
                runGame('SetChatVisibility false');
            }
        }, ACTIVE_SHOW_MS);
    }

    // Shown without focus: the game keeps the keyboard until the player opens the chat.
    function showForActivity() {
        var mode = visibilityMode();
        if (mode === 'never' || chatUiOpen) {
            return;
        }
        runGame('SetChatVisibility true');
        if (mode === 'active') {
            shownByActivity = true;
            hideAfterActivity();
        }
    }

    // Lines are comma-joined, each percent-encoded (H372):
    // "seq|channel|sender|relation|name|text" once decoded (text last, may contain '|').
    // relation: s=self, a=ally, e=enemy (viewer-relative), o=sender is a spectator.
    // Legacy 5-field lines omit relation.
    function applyLines(data) {
        if (!data || data.status !== 0 || typeof data.lines !== 'string') {
            return;
        }
        var firstPoll = lastSeq < 0;
        var maxSeq = lastSeq;
        var fresh = 0;
        var mine = String(resolvePlayerName()).toLowerCase();
        var rows = data.lines ? data.lines.split(',') : [];
        for (var i = 0; i < rows.length; i++) {
            var row;
            try {
                row = decodeURIComponent(rows[i]);
            } catch (e) {
                continue;
            }
            var parts = row.split('|');
            if (parts.length < 5) {
                continue;
            }
            var seq = Number(parts[0]) || 0;
            if (seq <= maxSeq) {
                continue;
            }
            maxSeq = seq;
            var relation = '';
            var from;
            var text;
            var rel = parts[3];
            if (parts.length >= 6 && (rel === 's' || rel === 'a' || rel === 'e' || rel === 'o')) {
                relation = rel;
                from = parts[4];
                text = parts.slice(5).join('|');
            } else {
                from = parts[3];
                text = parts.slice(4).join('|');
            }
            var kind = (relation === 's' || from.toLowerCase() === mine) ? 'self' : 'msg';
            if (!relation && kind === 'self') {
                relation = 's';
            }
            $scope.messages.push({
                from: from,
                text: text,
                kind: kind,
                channel: parts[1] === '1' ? 'team' : 'all',
                relation: relation
            });
            if ($scope.messages.length > 80) {
                $scope.messages.shift();
            }
            ++fresh;
        }
        lastSeq = maxSeq < 0 ? 0 : maxSeq;
        if (fresh > 0) {
            scrollHistory();
            if (!firstPoll) {
                showForActivity();
            }
        }
    }

    // One poll in flight; the next starts after the reply (or after POLL_STALL_MS without one).
    function pollServer() {
        var serial = ++pollSerial;
        function again() {
            if (serial === pollSerial) {
                ++pollSerial;
                setTimeout(pollServer, POLL_MS);
            }
        }
        if (!executeShell('/prism/chat/poll', { since: lastSeq < 0 ? 0 : lastSeq }, function (data) {
            applyLines(data);
            again();
        })) {
            again();
            return;
        }
        setTimeout(again, POLL_STALL_MS);
    }

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
        shownByActivity = false;
        releaseChatFocus();
        if (visibilityMode() !== 'always') {
            runGame('SetChatVisibility false');
        }
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

        var named = identityNameSync() || resolvePlayerName();
        if (named) {
            $scope.playerName = named;
        }
        $scope.draft = '';
        if (input) {
            input.value = '';
        }

        // The server relays the line back to this player too, so it is not pushed here.
        var sent = executeShell('/prism/chat/send',
            { channel: $scope.channel, user: $scope.playerName, text: text },
            function (data) {
                if (!data || data.status !== 0) {
                    pushLine('', 'Message not sent (no match server connection).', 'system');
                }
            });
        if (!sent) {
            pushLine('', 'Message not sent (chat unavailable).', 'system');
        }
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

    if (visibilityMode() === 'always') {
        runGame('SetChatVisibility true');
    }
    pollServer();

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
