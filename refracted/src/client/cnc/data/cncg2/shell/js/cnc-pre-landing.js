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
        var n = (navigator.language || 'en').toLowerCase();
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

    function chainShellSteps(ctx, statusFn, onComplete) {
        if (!hasShell() || !window.CncBlazeState) {
            onComplete();
            return;
        }

        var main = { text: 'Communicating with Refracted...', url: '/blaze/authenticate?email=' + ctx.email + '&password=' + ctx.password };
        statusFn(main.text);
        runShellStep(main, onComplete, AUTH_STEP_MS);
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
        getInitialStatus: getTagline,
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
                scheduleDone(onDone, tStart);
            }

            if (shouldSkip()) {
                statusAnimator.stop();
                onDone();
                return;
            }
            if (!hasShell()) {
                statusAnimator.setLine(getTagline());
                done();
                return;
            }
            var cred = getCredentials();
            var ctx = {
                email: o.email != null ? o.email : cred.email,
                password: o.password != null ? o.password : cred.password
            };
            statusAnimator.setLine(getTagline());
            setTimeout(function () {
                chainShellSteps(
                    ctx,
                    function (line) {
                        statusAnimator.setLine(line);
                    },
                    done
                );
            }, 100);
        }
    };
})(window);
