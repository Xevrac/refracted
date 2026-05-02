/**
 * Persists Blaze / shell auth hints from shellaccesslayer JSON responses and syncs to Angular $rootScope.
 * Also accepts: sessionStorage, window.__CNC_BLAZE, postMessage (cnc:persona), game-injected fields (DSNM, etc.).
 */
(function (window) {
    'use strict';

    var KEY = 'cnc_blaze_session';
    var SS_DSNM = 'cnc_blaze_dsnm';
    var SS_MAIL = 'cnc_blaze_email';
    var SS_PID = 'cnc_blaze_persona_id';

    var UNKNOWN_PLAYER = 'Guest';

    function nonEmptyString(x) {
        if (x == null) {
            return null;
        }
        var s = String(x).trim();
        return s ? s : null;
    }

    function emailLocalPart(mail) {
        if (mail == null) {
            return null;
        }
        var s = String(mail).trim();
        var at = s.indexOf('@');
        if (at <= 0) {
            return null;
        }
        var local = s.substring(0, at);
        return nonEmptyString(local);
    }

    function digName(u) {
        if (!u || typeof u !== 'object') {
            return null;
        }
        return nonEmptyString(
            u.DSNM || u.dsnm || u.displayName || u.display_name || u.personaName || u.name || u.nickname
                || u.persona || u.PersonaName
        );
    }

    function pickNameFromRes(res, depth) {
        if (!res || typeof res !== 'object') {
            return null;
        }
        depth = depth || 0;
        if (depth > 3) {
            return null;
        }
        var u = res.user || res.User || res.account || res.Account || res.profile || res.USER || null;
        var direct =
            nonEmptyString(
                res.DSNM || res.dsnm || res.displayName || res.display_name || res.personaName || res.persona
                    || res.name || res.nickname || res.localName || res.LocalName
            ) ||
            (res.user && res.user.DSNM && nonEmptyString(res.user.DSNM)) ||
            (res.PDTL && (nonEmptyString(res.PDTL.DSNM) || digName(res.PDTL))) ||
            (res.SESSION && digName(res.SESSION)) ||
            (res.SessionInfo && digName(res.SessionInfo)) ||
            (res.persona && typeof res.persona === 'object' && digName(res.persona)) ||
            (res.Persona && typeof res.Persona === 'object' && digName(res.Persona)) ||
            digName(u);
        if (direct) {
            return direct;
        }
        var nest = res.data || res.result || res.payload || res.body;
        if (nest && typeof nest === 'object') {
            var n2 = pickNameFromRes(nest, depth + 1);
            if (n2) {
                return n2;
            }
        }
        if (res.success) {
            var u2 = res.user || res.User;
            if (u2) {
                var n3 = digName(u2);
                if (n3) {
                    return n3;
                }
            }
        }
        var mail = nonEmptyString(res.MAIL || res.mail || res.email || (u && (u.MAIL || u.mail || u.email)));
        if (mail) {
            var lp = emailLocalPart(mail);
            if (lp) {
                return lp;
            }
        }
        return null;
    }

    var CncBlazeState = {
        UNKNOWN_PLAYER: UNKNOWN_PLAYER,
        email: null,
        displayName: null,
        personaId: null,
        _listeners: [],

        getPlayerName: function () {
            var n = CncBlazeState.displayName;
            if (n == null) {
                return UNKNOWN_PLAYER;
            }
            n = String(n).trim();
            return n ? n : UNKNOWN_PLAYER;
        },

        applyExternalHints: function () {
            var changed = false;
            // Refracted-injected profile (window.__CNC_PROFILE) — primary source when
            // the user has set a profile in the Refracted Accounts dialog. Lower priority
            // than live Blaze responses but higher than email-localpart fallback.
            var prof = window.__CNC_PROFILE;
            if (prof && typeof prof === 'object') {
                if (!CncBlazeState.displayName && prof.displayName) {
                    CncBlazeState.displayName = String(prof.displayName);
                    changed = true;
                }
                if (!CncBlazeState.email && prof.email) {
                    CncBlazeState.email = String(prof.email);
                    changed = true;
                }
                if (CncBlazeState.personaId == null && prof.personaId != null) {
                    CncBlazeState.personaId = String(prof.personaId);
                    changed = true;
                }
            }
            // Native bridge object (legacy fallback)
            var g = window.__CNC_BLAZE;
            if (g && typeof g === 'object') {
                if (g.displayName) {
                    CncBlazeState.displayName = String(g.displayName);
                    changed = true;
                }
                if (g.email) {
                    CncBlazeState.email = String(g.email);
                    changed = true;
                }
                if (g.personaId != null) {
                    CncBlazeState.personaId = String(g.personaId);
                    changed = true;
                }
            }
            if (!CncBlazeState.displayName && CncBlazeState.email) {
                var lp2 = emailLocalPart(CncBlazeState.email);
                if (lp2) {
                    CncBlazeState.displayName = lp2;
                    changed = true;
                }
            }
            if (changed) {
                CncBlazeState.persist();
                CncBlazeState.notifyListeners();
            }
        },

        subscribe: function (fn) {
            if (typeof fn !== 'function') {
                return;
            }
            CncBlazeState._listeners.push(fn);
        },

        notifyListeners: function () {
            for (var i = 0; i < CncBlazeState._listeners.length; i++) {
                try {
                    CncBlazeState._listeners[i]();
                } catch (e) { /* empty */ }
            }
        },

        load: function () {
            try {
                var s = sessionStorage.getItem(KEY);
                if (!s) {
                    return;
                }
                var o = JSON.parse(s);
                CncBlazeState.email = o.email || null;
                CncBlazeState.displayName = o.displayName || null;
                CncBlazeState.personaId = o.personaId != null ? String(o.personaId) : null;
            } catch (e) { /* empty */ }
        },

        persist: function () {
            try {
                sessionStorage.setItem(KEY, JSON.stringify({
                    email: CncBlazeState.email,
                    displayName: CncBlazeState.displayName,
                    personaId: CncBlazeState.personaId
                }));
            } catch (e) { /* empty */ }
        },

        pickUser: function (res) {
            if (!res || typeof res !== 'object') {
                return null;
            }
            return res.user || res.User || res.account || res.Account || res.profile
                || res.USER || null;
        },

        onShellResult: function (res) {
            if (!res || typeof res !== 'object') {
                CncBlazeState.notifyListeners();
                return;
            }

            var u = CncBlazeState.pickUser(res);
            var email = nonEmptyString(
                res.MAIL || res.mail || res.email
                    || (u && (u.MAIL || u.mail || u.email))
                    || res.login
            );
            var name = pickNameFromRes(res);
            if (!name && email) {
                name = emailLocalPart(email);
            }
            var pid = res.personaId != null ? res.personaId
                : (res.PID != null ? res.PID
                    : (u && (u.personaId != null ? u.personaId : (u.PID != null ? u.PID : u.id))));

            if (res.error && (email == null) && (name == null) && (pid == null)) {
                CncBlazeState.notifyListeners();
                return;
            }

            if (email) {
                CncBlazeState.email = email;
            }
            if (name) {
                CncBlazeState.displayName = name;
            }
            if (pid != null) {
                CncBlazeState.personaId = String(pid);
            }

            CncBlazeState.persist();
            CncBlazeState.notifyListeners();
        }
    };

    CncBlazeState.load();
    CncBlazeState.applyExternalHints();

    if (typeof ShellResult === 'function') {
        var _inner = ShellResult;
        window.ShellResult = function (res) {
            CncBlazeState.onShellResult(res);
            _inner(res);
        };
    }

    window.CncBlazeState = CncBlazeState;
})(window);
