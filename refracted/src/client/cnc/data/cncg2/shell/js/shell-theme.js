/**
 * Shell UI theme — Classic, Aurora (Default), Aurora (Alternate).
 *
 * Stylesheets: every <link data-shell-css="..."> lists the themes it belongs to. Only the
 * active theme's links stay in the document, so the classic stylesheets (Bootstrap included)
 * and the Aurora ones never mix. Each Aurora theme is one generated stylesheet:
 * css/themes/aurora.css and css/themes/aurora-alt.css (tools/cnc-shell-webdev/aurora).
 * Both Aurora themes share the Aurora templates; the alternate also carries
 * body.cc-theme--aurora-alt.
 *
 * Scale: the Aurora stylesheets are written in rem against a reference window
 * (1920x1080, or <html data-au-ref="WxH">). fitRoot() sets the root font size to
 * 16px x min(width/refW, height/refH), so the layout holds its proportions at any size.
 *
 * Requires the EAWebKit 16.4 engine (flexbox); there is no 13.x fallback.
 *
 * Active theme: applied now. Default theme: used on next shell boot.
 * Persists in the shell WebKit profile localStorage (CNCO_DL\0\webkit on retail).
 */
(function (global) {
    var STORAGE_KEY = 'cnc_shell_ui_theme';
    var THEMES = {
        classic: {
            id: 'classic',
            label: 'Classic',
            bodyClass: 'cc-theme--classic',
            family: 'classic',
            rootTemplate: 'view/roots/classic.html',
            homeTemplate: 'view/home.html',
            lobbyTemplate: 'view/lobby/lobby-classic.html',
            hint: 'Classic is the original Generals 2 shell layout built for the alpha client.'
        },
        aurora: {
            id: 'aurora',
            label: 'Aurora (Default)',
            bodyClass: 'cc-theme--aurora',
            family: 'aurora',
            rootTemplate: 'view/roots/aurora.html',
            homeTemplate: 'view/home-aurora.html',
            lobbyTemplate: 'view/lobby/lobby-aurora.html',
            hint: 'Aurora (Default) puts your general over the battlefield, in gunmetal with light blue accents.'
        },
        'aurora-alt': {
            id: 'aurora-alt',
            label: 'Aurora (Alternate)',
            bodyClass: 'cc-theme--aurora cc-theme--aurora-alt',
            family: 'aurora',
            rootTemplate: 'view/roots/aurora.html',
            homeTemplate: 'view/home-aurora.html',
            lobbyTemplate: 'view/lobby/lobby-aurora.html',
            hint: 'Aurora (Alternate) keeps the classic backdrop, with cyan readouts and a wide feature banner.'
        }
    };
    THEMES['cnc-alpha'] = THEMES.classic;
    var FACTORY_DEFAULT = 'aurora';
    var listeners = [];
    var sheetSlots = null;

    /** Closed beta playtest (DebugShell, devWrapper.html): Aurora (Default) only. */
    function playtestLocked() {
        return global.__CNC_PLAYTEST === true;
    }

    function normalize(id) {
        if (playtestLocked()) {
            return FACTORY_DEFAULT;
        }
        if (id && THEMES[id]) {
            return THEMES[id].id;
        }
        return FACTORY_DEFAULT;
    }

    /** The theme that is drawn: the chosen one. */
    function effective(id) {
        return normalize(id);
    }

    /**
     * Keep only the active theme's <link data-shell-css> elements in the document.
     * A link that is out is replaced by a comment node holding its place.
     */
    function syncStylesheets(themeId) {
        var doc = global.document;
        var i;
        if (!doc || !doc.getElementsByTagName) {
            return;
        }
        if (!sheetSlots) {
            sheetSlots = [];
        }
        var links = doc.getElementsByTagName('link');
        for (i = 0; i < links.length; i++) {
            if (links[i].getAttribute('data-shell-css') && !links[i]._cncSlot) {
                links[i]._cncSlot = true;
                sheetSlots.push({ link: links[i], mark: doc.createComment('shell-css'), on: true });
            }
        }
        for (i = 0; i < sheetSlots.length; i++) {
            var slot = sheetSlots[i];
            var want = (' ' + slot.link.getAttribute('data-shell-css') + ' ').indexOf(' ' + themeId + ' ') !== -1;
            if (want && !slot.on && slot.mark.parentNode) {
                slot.mark.parentNode.replaceChild(slot.link, slot.mark);
                slot.on = true;
            } else if (!want && slot.on && slot.link.parentNode) {
                slot.link.parentNode.replaceChild(slot.mark, slot.link);
                slot.on = false;
            }
        }
    }

    /** Root font size for the rem-based Aurora stylesheets. */
    function fitRoot() {
        var doc = global.document;
        var root = doc && doc.documentElement;
        if (!root) {
            return;
        }
        var ref = /^(\d+)x(\d+)$/.exec(root.getAttribute('data-au-ref') || '') || [0, 1920, 1080];
        var w = global.innerWidth || root.clientWidth || 0;
        var h = global.innerHeight || root.clientHeight || 0;
        if (!w || !h) {
            return;
        }
        var k = Math.min(w / Number(ref[1]), h / Number(ref[2]));
        root.style.fontSize = (Math.round(k * 16 * 1000) / 1000) + 'px';
    }

    function meta(id) {
        return THEMES[effective(id)];
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
            .replace(/\bcc-theme--aurora-alt\b/g, '')
            .replace(/\bcc-theme--aurora\b/g, '')
            .replace(/\s+/g, ' ')
            .replace(/^\s+|\s+$/g, '');
    }

    function apply(id) {
        var themeId = effective(id);
        var m = meta(themeId);
        var body = global.document && global.document.body;
        syncStylesheets(themeId);
        fitRoot();
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

    function get() {
        return effective(readLocalPrefs().theme);
    }

    function getDefault() {
        return readLocalPrefs().defaultTheme;
    }

    function set(id) {
        var prefs = readLocalPrefs();
        prefs.theme = normalize(id);
        writeLocalPrefs(prefs);
        return apply(prefs.theme);
    }

    function setDefault(id) {
        var prefs = readLocalPrefs();
        prefs.defaultTheme = normalize(id);
        writeLocalPrefs(prefs);
        return prefs.defaultTheme;
    }

    function restore(themeId, defaultThemeId) {
        var prefs = {
            theme: normalize(themeId),
            defaultTheme: normalize(defaultThemeId || themeId)
        };
        writeLocalPrefs(prefs);
        apply(prefs.theme);
        return prefs.theme;
    }

    /** Themes for the OPTIONS picker; `locked` ones are shown greyed out (playtest: all but Default). */
    function list() {
        var lock = playtestLocked();
        return [
            { id: 'classic', label: THEMES.classic.label, locked: lock },
            { id: 'aurora', label: THEMES.aurora.label, locked: false },
            { id: 'aurora-alt', label: THEMES['aurora-alt'].label, locked: lock }
        ];
    }

    function hint(id) {
        return meta(id || get()).hint;
    }

    /** 'aurora' for both Aurora themes, 'classic' otherwise. */
    function family(id) {
        return meta(id || get()).family;
    }

    function bodyClass(id) {
        return meta(id || get()).bodyClass;
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
        var prefs = readLocalPrefs();
        var bootId = normalize(prefs.defaultTheme || prefs.theme);
        prefs.theme = bootId;
        prefs.defaultTheme = normalize(prefs.defaultTheme || bootId);
        writeLocalPrefs(prefs);
        var drawn = apply(bootId);
        if (typeof done === 'function') {
            done(drawn);
        }
    }

    if (global.addEventListener) {
        global.addEventListener('resize', fitRoot, false);
        global.addEventListener('load', fitRoot, false);
    }

    if (global.document) {
        // Runs in <head>, after the stylesheet links: drop the other themes' sheets before first paint.
        syncStylesheets(effective(readLocalPrefs().theme));
        fitRoot();
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
        restore: restore,
        apply: apply,
        list: list,
        hint: hint,
        family: family,
        bodyClass: bodyClass,
        boot: boot,
        rootTemplate: rootTemplate,
        homeTemplate: homeTemplate,
        lobbyTemplate: lobbyTemplate,
        onChange: onChange,
        normalize: normalize,
        fitRoot: fitRoot
    };
})(window);
