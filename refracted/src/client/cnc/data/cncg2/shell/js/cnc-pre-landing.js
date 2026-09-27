/**
 * Pre-landing
 */
(function (window) {
    'use strict';

    var DEFAULT_EMAIL = 'user@example.com';
    var DEFAULT_PASSWORD = 'test';
    var AUTH_STEP_MS = 8000;
    var MIN_PRE_LANDING_MS = 8000;
    var STATUS_DOT_FRAMES = ['...', '..', '.', '..', '...'];
    var STATUS_DOT_MS = 340;

    function scheduleDone(onDone, tStart) {
        var now = (typeof Date !== 'undefined' && Date.now) ? Date.now() : 0;
        var elapsed = tStart != null ? (now - tStart) : 0;
        var wait = Math.max(0, MIN_PRE_LANDING_MS - elapsed);
        if (wait <= 0) {
            onDone();
        } else {
            setTimeout(onDone, wait);
        }
    }

    function hasShell() {
        return (
            typeof shellaccesslayer !== 'undefined' &&
            shellaccesslayer &&
            typeof shellaccesslayer.execute === 'function'
        );
    }

    function getTagline() {
        return 'Powered by Refracted';
    }

    function getInitialStatus() {
        return 'LOGGING YOU IN, PLEASE WAIT';
    }

    function getBootstrapCredentials() {
        if (typeof location !== 'undefined' && location.search && location.search.indexOf('cncEmail=') >= 0) {
            var m = location.search.match(/[?&]cncEmail=([^&]+)/);
            if (m) {
                try {
                    return { email: decodeURIComponent(m[1].replace(/\+/g, ' ')) };
                } catch (e) { /* empty */ }
            }
        }
        var e = null;
        var p = null;
        try {
            e = sessionStorage.getItem('cnc_bootstrap_email');
            p = sessionStorage.getItem('cnc_bootstrap_password');
        } catch (err) { /* empty */ }
        return { email: e, password: p };
    }

    function getProfileCredentials() {
        try {
            if (window.__CNC_PROFILE && typeof window.__CNC_PROFILE.email === 'string') {
                return { email: window.__CNC_PROFILE.email };
            }
        } catch (err) { /* empty */ }
        return { email: null };
    }

    function getCredentials() {
        var c = getBootstrapCredentials();
        var prof = getProfileCredentials();
        return {
            email: c.email || prof.email || DEFAULT_EMAIL,
            password: c.password != null && c.password !== '' ? c.password : DEFAULT_PASSWORD
        };
    }

    function shouldSkip() {
        if (typeof location === 'undefined' || !location.search) {
            return false;
        }
        return /[?&]skipLogin=1(?!\d)/.test(location.search) || /[?&]skipPreLanding=1/.test(location.search);
    }

    var SESSION_DONE_KEY = 'cnc_shell_prelanding_done';

    function isSessionDone() {
        try {
            return sessionStorage.getItem(SESSION_DONE_KEY) === '1';
        } catch (e) {
            return false;
        }
    }

    function markSessionDone() {
        try {
            sessionStorage.setItem(SESSION_DONE_KEY, '1');
        } catch (e) { /* empty */ }
    }

    function scheduleReturnFromMatch() {
        markSessionDone();
    }

    /** True when the full login splash + authenticate chain should run. */
    function shouldRunPreLanding() {
        if (shouldSkip()) {
            return false;
        }
        return !isSessionDone();
    }

    function runShellStep(st, onDone, timeoutMs) {
        if (!hasShell() || !window.CncBlazeState) {
            onDone();
            return;
        }
        var done = false;
        var t = setTimeout(function () {
            if (done) {
                return;
            }
            done = true;
            CncBlazeState.onShellResult({ error: 'timeout', step: st.url });
            onDone();
        }, timeoutMs);
        function onResp(res) {
            if (done) {
                return;
            }
            done = true;
            clearTimeout(t);
            if (res && window.CncBlazeState) {
                CncBlazeState.onShellResult(res);
            }
            onDone();
        }
        try {
            shellaccesslayer.execute({ _response: onResp, url: st.url });
        } catch (e) {
            onResp({ error: String(e) });
        }
    }

    var LOGIN_KEY_STORAGE = 'cnc_login_key';

    /** Per-launch key: rides the login email to Prism (TOKN suffix) and scopes /cnc/auth-refusal. */
    function getLoginKey() {
        var key = null;
        try {
            key = sessionStorage.getItem(LOGIN_KEY_STORAGE);
        } catch (e) { /* empty */ }
        if (key) {
            return key;
        }
        key = '';
        for (var i = 0; i < 24; i++) {
            key += Math.floor(Math.random() * 16).toString(16);
        }
        try {
            sessionStorage.setItem(LOGIN_KEY_STORAGE, key);
        } catch (e2) { /* empty */ }
        return key;
    }

    function emailWithLoginKey(email, key) {
        var s = String(email || '');
        var at = s.indexOf('@');
        return at > 0 ? s.substring(0, at) + '~rk~' + key + s.substring(at) : s;
    }

    function chainShellSteps(ctx, statusFn, onComplete) {
        if (!hasShell() || !window.CncBlazeState) {
            onComplete();
            return;
        }

        var key = getLoginKey();
        var main = { text: 'Communicating with Refracted...', url: '/blaze/authenticate?email=' + emailWithLoginKey(ctx.email, key) + '&password=' + ctx.password };
        statusFn(main.text);
        runShellStep(main, function () {
            checkAuthRefusal(key, statusFn, onComplete);
        }, AUTH_STEP_MS);
    }

    function formatBanDuration(secs) {
        var n;
        var unit;
        if (secs >= 86400) {
            n = Math.ceil(secs / 86400);
            unit = 'day';
        } else if (secs >= 3600) {
            n = Math.ceil(secs / 3600);
            unit = 'hour';
        } else {
            n = Math.max(1, Math.ceil(secs / 60));
            unit = 'minute';
        }
        return n + ' ' + unit + (n === 1 ? '' : 's');
    }

    function authRefusalInfo(r) {
        if (r.reason === 'banned') {
            return {
                title: 'Account Banned',
                message: (r.permanent || r.remainingSecs == null)
                    ? 'Your account has been banned permanently.'
                    : 'Your account has been banned for ' + formatBanDuration(r.remainingSecs) + '.'
            };
        }
        return {
            title: 'Sign-in Required',
            message: 'Missing or invalid token, close the game and retry via the launcher.'
        };
    }

    /** Refracted records a refused Blaze login under this launch's key; a refusal blocks the shell behind a Quit-only modal. */
    function checkAuthRefusal(key, statusFn, onComplete) {
        var done = false;
        var proceed = function () {
            if (!done) {
                done = true;
                onComplete();
            }
        };
        try {
            var xhr = new XMLHttpRequest();
            xhr.open('GET', '/cnc/auth-refusal?key=' + encodeURIComponent(key), true);
            xhr.onload = function () {
                var r = null;
                try {
                    r = JSON.parse(xhr.responseText);
                } catch (e) { /* ignore */ }
                if (r && r.refused && window.CncShowAuthRefusal) {
                    done = true;
                    statusFn('');
                    window.CncShowAuthRefusal(authRefusalInfo(r));
                    return;
                }
                proceed();
            };
            xhr.onerror = xhr.onabort = xhr.ontimeout = proceed;
            xhr.timeout = 3000;
            xhr.send(null);
        } catch (e) {
            proceed();
        }
    }

    function shouldAnimateStatus(line) {
        return typeof line === 'string' && /communicating with refracted/i.test(line);
    }

    function stripTrailingDots(line) {
        if (typeof line !== 'string') {
            return '';
        }
        return line.replace(/\s*\.+\s*$/, '');
    }

    function createStatusAnimator(setStatus) {
        var timer = null;
        var base = '';
        var frame = 0;
        var animating = false;

        function render() {
            if (!animating) {
                setStatus(base);
                return;
            }
            setStatus(base + STATUS_DOT_FRAMES[frame]);
            frame = (frame + 1) % STATUS_DOT_FRAMES.length;
        }

        return {
            setLine: function (line) {
                if (shouldAnimateStatus(line)) {
                    base = stripTrailingDots(line);
                    frame = 0;
                    animating = true;
                    render();
                    if (timer !== null) {
                        clearInterval(timer);
                    }
                    timer = setInterval(render, STATUS_DOT_MS);
                } else {
                    animating = false;
                    base = line || '';
                    if (timer !== null) {
                        clearInterval(timer);
                        timer = null;
                    }
                    setStatus(base);
                }
            },
            stop: function () {
                if (timer !== null) {
                    clearInterval(timer);
                    timer = null;
                }
                animating = false;
            }
        };
    }

    window.CncPreLanding = {
        getTagline: getTagline,
        getInitialStatus: getInitialStatus,
        hasShell: hasShell,
        shouldSkip: shouldSkip,
        getCredentials: getCredentials,

        run: function (o) {
            o = o || {};
            var setStatus = o.setStatus || function () {};
            var onDone = o.onDone || function () {};
            var tStart = (typeof Date !== 'undefined' && Date.now) ? Date.now() : 0;
            var statusAnimator = createStatusAnimator(setStatus);
            function done() {
                statusAnimator.stop();
                markSessionDone();
                scheduleDone(onDone, tStart);
            }

            if (!shouldRunPreLanding()) {
                statusAnimator.stop();
                onDone();
                return;
            }
            if (shouldSkip()) {
                statusAnimator.stop();
                markSessionDone();
                onDone();
                return;
            }
            if (!hasShell()) {
                statusAnimator.setLine(getInitialStatus());
                done();
                return;
            }
            var cred = getCredentials();
            var ctx = {
                email: o.email != null ? o.email : cred.email,
                password: o.password != null ? o.password : cred.password
            };
            statusAnimator.setLine(getInitialStatus());
            setTimeout(function () {
                chainShellSteps(
                    ctx,
                    function (line) {
                        statusAnimator.setLine(line);
                    },
                    done
                );
            }, 100);
        },

        isSessionDone: isSessionDone,
        shouldRunPreLanding: shouldRunPreLanding,
        scheduleReturnFromMatch: scheduleReturnFromMatch
    };
})(window);
