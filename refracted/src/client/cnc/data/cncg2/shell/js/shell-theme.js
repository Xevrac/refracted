/**
 * Shell UI theme — Classic vs Aurora.
 * Active theme: applied now. Default theme: used on next shell boot.
 * Persists: localStorage + Refracted GET/POST /cnc/shell-theme (prefs JSON).
 * EAWebKit 13.2 / ES5 only.
 */
(function (global) {
    var STORAGE_KEY = 'cnc_shell_ui_theme';
    var PREFS_URL = '/cnc/shell-theme';
    var THEMES = {
        classic: {
            id: 'classic',
            label: 'Classic',
            bodyClass: 'cc-theme--classic',
            rootTemplate: 'view/roots/classic.html',
            homeTemplate: 'view/home.html',
            lobbyTemplate: 'view/lobby/lobby-classic.html',
            hint: 'Classic is the original Generals 2 shell layout built for the alpha client.'
        },
        aurora: {
            id: 'aurora',
            label: 'Aurora',
            bodyClass: 'cc-theme--aurora',
            rootTemplate: 'view/roots/aurora.html',
            homeTemplate: 'view/home-aurora.html',
            lobbyTemplate: 'view/lobby/lobby-aurora.html',
            hint: 'Aurora is a modernised view of the shell layout built for the game client.'
        }
    };
    THEMES['cnc-alpha'] = THEMES.classic;
    var FACTORY_DEFAULT = 'aurora';
    var listeners = [];

    function normalize(id) {
        if (!id) {
            return FACTORY_DEFAULT;
        }
        if (THEMES[id]) {
            return THEMES[id].id;
        }
        return FACTORY_DEFAULT;
    }

    function meta(id) {
        return THEMES[normalize(id)];
    }

    function emptyPrefs() {
        return {
            theme: FACTORY_DEFAULT,
            defaultTheme: FACTORY_DEFAULT
        };
    }

    function readLocalPrefs() {
        var prefs = emptyPrefs();
        try {
            if (!global.localStorage) {
                return prefs;
            }
            var raw = global.localStorage.getItem(STORAGE_KEY);
            if (!raw) {
                return prefs;
            }
            if (raw.charAt(0) === '{') {
                var data = global.JSON ? JSON.parse(raw) : null;
                if (data) {
                    prefs.theme = normalize(data.theme || data.defaultTheme);
                    prefs.defaultTheme = normalize(data.defaultTheme || data.theme);
                    return prefs;
                }
            }
            prefs.theme = normalize(raw);
            prefs.defaultTheme = prefs.theme;
        } catch (e) { /* ignore */ }
        return prefs;
    }

    function writeLocalPrefs(prefs) {
        var payload = {
            theme: normalize(prefs.theme),
            defaultTheme: normalize(prefs.defaultTheme)
        };
        try {
            if (global.localStorage && global.JSON) {
                global.localStorage.setItem(STORAGE_KEY, JSON.stringify(payload));
            }
        } catch (e) { /* ignore */ }
        return payload;
    }

    function notify(themeId) {
        var i;
        for (i = 0; i < listeners.length; i++) {
            try {
                listeners[i](themeId);
            } catch (e) { /* ignore */ }
        }
    }

    function stripThemeClasses(el) {
        if (!el || !el.className) {
            return;
        }
        el.className = String(el.className)
            .replace(/\bcc-theme--cnc-alpha\b/g, '')
            .replace(/\bcc-theme--classic\b/g, '')
            .replace(/\bcc-theme--aurora\b/g, '')
            .replace(/\s+/g, ' ')
            .replace(/^\s+|\s+$/g, '');
    }

    function apply(id) {
        var themeId = normalize(id);
        var m = meta(themeId);
        var body = global.document && global.document.body;
        if (!body) {
            return themeId;
        }
        if ((' ' + body.className + ' ').indexOf(' cc-theme ') === -1) {
            body.className = (body.className ? body.className + ' ' : '') + 'cc-theme';
        }
        stripThemeClasses(body);
        body.className = (body.className + ' ' + m.bodyClass).replace(/\s+/g, ' ').replace(/^\s+|\s+$/g, '');
        body.setAttribute('data-shell-theme', themeId);
        notify(themeId);
        return themeId;
    }

    function persistRemote(prefs) {
        var payload = JSON.stringify({
            theme: normalize(prefs.theme),
            defaultTheme: normalize(prefs.defaultTheme)
        });
        try {
            if (global.jQuery && global.jQuery.ajax) {
                global.jQuery.ajax({
                    url: PREFS_URL,
                    type: 'POST',
                    contentType: 'application/json',
                    data: payload,
                    dataType: 'text',
                    timeout: 4000
                });
                return;
            }
        } catch (e) { /* ignore */ }
        try {
            var xhr = new global.XMLHttpRequest();
            xhr.open('POST', PREFS_URL, true);
            xhr.setRequestHeader('Content-Type', 'application/json');
            xhr.send(payload);
        } catch (e2) { /* ignore */ }
    }

    function fetchRemote(done) {
        function finish(prefs) {
            if (typeof done === 'function') {
                done({
                    theme: normalize(prefs.theme),
                    defaultTheme: normalize(prefs.defaultTheme)
                });
            }
        }
        try {
            if (global.jQuery && global.jQuery.ajax) {
                global.jQuery.ajax({
                    url: PREFS_URL,
                    type: 'GET',
                    dataType: 'json',
                    timeout: 3000,
                    success: function (data) {
                        var local = readLocalPrefs();
                        finish({
                            theme: data && data.theme ? data.theme : local.theme,
                            defaultTheme: data && (data.defaultTheme || data.theme)
                                ? (data.defaultTheme || data.theme)
                                : local.defaultTheme
                        });
                    },
                    error: function () {
                        finish(readLocalPrefs());
                    }
                });
                return;
            }
        } catch (e) { /* ignore */ }
        try {
            var xhr = new global.XMLHttpRequest();
            xhr.open('GET', PREFS_URL, true);
            xhr.onreadystatechange = function () {
                if (xhr.readyState !== 4) {
                    return;
                }
                if (xhr.status >= 200 && xhr.status < 300 && xhr.responseText) {
                    try {
                        var data = global.JSON ? JSON.parse(xhr.responseText) : null;
                        var local = readLocalPrefs();
                        finish({
                            theme: data && data.theme ? data.theme : local.theme,
                            defaultTheme: data && (data.defaultTheme || data.theme)
                                ? (data.defaultTheme || data.theme)
                                : local.defaultTheme
                        });
                        return;
                    } catch (pe) { /* ignore */ }
                }
                finish(readLocalPrefs());
            };
            xhr.send(null);
            return;
        } catch (e2) { /* ignore */ }
        finish(readLocalPrefs());
    }

    function get() {
        return readLocalPrefs().theme;
    }

    function getDefault() {
        return readLocalPrefs().defaultTheme;
    }

    function set(id) {
        var prefs = readLocalPrefs();
        prefs.theme = apply(id);
        writeLocalPrefs(prefs);
        persistRemote(prefs);
        return prefs.theme;
    }

    function setDefault(id) {
        var prefs = readLocalPrefs();
        prefs.defaultTheme = normalize(id);
        writeLocalPrefs(prefs);
        persistRemote(prefs);
        return prefs.defaultTheme;
    }

    function list() {
        return [
            { id: 'classic', label: THEMES.classic.label },
            { id: 'aurora', label: THEMES.aurora.label }
        ];
    }

    function hint(id) {
        return meta(id || get()).hint;
    }

    function rootTemplate(id) {
        return meta(id || get()).rootTemplate;
    }

    function homeTemplate(id) {
        return meta(id || get()).homeTemplate;
    }

    function lobbyTemplate(id) {
        return meta(id || get()).lobbyTemplate;
    }

    function onChange(fn) {
        if (typeof fn === 'function') {
            listeners.push(fn);
        }
    }

    function boot(done) {
        fetchRemote(function (prefs) {
            /* Boot applies default theme (next-session preference). */
            prefs.theme = prefs.defaultTheme;
            writeLocalPrefs(prefs);
            apply(prefs.theme);
            if (typeof done === 'function') {
                done(prefs.theme);
            }
        });
    }

    if (global.document) {
        if (global.document.body) {
            apply(readLocalPrefs().theme);
        } else if (global.document.addEventListener) {
            global.document.addEventListener('DOMContentLoaded', function () {
                apply(readLocalPrefs().theme);
            }, false);
        } else if (global.document.attachEvent) {
            global.document.attachEvent('onreadystatechange', function () {
                if (global.document.readyState === 'complete') {
                    apply(readLocalPrefs().theme);
                }
            });
        }
    }

    global.CncShellTheme = {
        STORAGE_KEY: STORAGE_KEY,
        DEFAULT: FACTORY_DEFAULT,
        get: get,
        getDefault: getDefault,
        set: set,
        setDefault: setDefault,
        apply: apply,
        list: list,
        hint: hint,
        boot: boot,
        rootTemplate: rootTemplate,
        homeTemplate: homeTemplate,
        lobbyTemplate: lobbyTemplate,
        onChange: onChange,
        normalize: normalize
    };
})(window);
