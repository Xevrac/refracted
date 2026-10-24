/**
 * Lobby presets: named sets of match tweaks the host picks in Lobby Options.
 * Aurora ships two (Generals 2, Generals Classic); players save their own, and one preset is the
 * default applied to every lobby they set up. A preset holds gameplay tweaks only: the password
 * and the spectator seat belong to the lobby, not the rules.
 * Custom presets persist in the shell WebKit profile localStorage (CNCO_DL\0\webkit on retail).
 */
(function (global) {
    'use strict';

    var STORAGE_KEY = 'cnc_lobby_presets';
    var MAX_CUSTOM = 12;
    var NAME_MAX = 24;
    var FACTORY_DEFAULT = 'generals2';

    /** lobbyOptions keys a preset sets, with the Generals 2 value. */
    var TWEAKS = [
        { key: 'enableInstantSelling', label: 'Instant Selling', def: false,
          tip: 'Structures sell instantly instead of after 10 seconds.' },
        { key: 'enableRebuildableDerricks', label: 'Rebuildable Derricks', def: true,
          tip: 'A destroyed oil derrick leaves its well, so you can build a new one.' },
        { key: 'enableGeneralsArmor', label: 'Combat Tuning (Classic)', def: false,
          tip: 'Generals-style damage: each weapon type is strong or weak against each armor type.' },
        { key: 'enableShowResourcesInFog', label: 'Resources Through Fog', def: true,
          tip: 'Supply stashes and oil wells are drawn under the fog and on the minimap from the start. The fog itself stays closed.' },
        { key: 'enableChinookName', label: 'Chinook (Zero Hour name)', def: false,
          tip: 'The EU Harvester is called the Chinook, as in Zero Hour. Name only: the unit plays the same.' },
        { key: 'enableTechTree', label: 'Tech Tree', def: true, soon: true },
        { key: 'enableSpecialAbilities', label: 'Special Abilities', def: true, soon: true },
        { key: 'enableUnlockFullFactionRoster', label: 'Factions Only', def: false, soon: true }
    ];

    function factoryValues() {
        var v = {};
        var i;
        for (i = 0; i < TWEAKS.length; i++) {
            v[TWEAKS[i].key] = TWEAKS[i].def;
        }
        return v;
    }

    function withValues(overrides) {
        var v = factoryValues();
        var k;
        for (k in overrides) {
            if (overrides.hasOwnProperty(k)) {
                v[k] = !!overrides[k];
            }
        }
        return v;
    }

    var BUILTIN = [
        { id: 'generals2', label: 'Generals 2', builtin: true,
          blurb: 'The Command & Conquer 2013 experience, or \'Generals 2\' as it\'s also called. Faithfully restored and preserved.',
          values: factoryValues() },
        { id: 'classic', label: 'Generals Classic', builtin: true,
          blurb: 'All game rules and gameplay changes inspired by Generals and Zero Hour are turned ON, get the best of both worlds.',
          values: withValues({ enableRebuildableDerricks: false, enableGeneralsArmor: true, enableChinookName: true }) }
    ];

    var state = null;
    var listCache = null;

    function normalizeCustom(raw) {
        if (!raw || typeof raw.id !== 'string' || typeof raw.label !== 'string') {
            return null;
        }
        var label = raw.label.replace(/\s+/g, ' ').replace(/^\s+|\s+$/g, '').substring(0, NAME_MAX);
        if (!label) {
            return null;
        }
        return { id: raw.id, label: label, builtin: false, blurb: '', values: withValues(raw.values || {}) };
    }

    function read() {
        if (state) {
            return state;
        }
        state = { custom: [], defaultId: FACTORY_DEFAULT };
        try {
            var raw = global.localStorage && global.localStorage.getItem(STORAGE_KEY);
            var parsed = raw ? global.JSON.parse(raw) : null;
            var i;
            if (parsed && parsed.custom && parsed.custom.length) {
                for (i = 0; i < parsed.custom.length && state.custom.length < MAX_CUSTOM; i++) {
                    var p = normalizeCustom(parsed.custom[i]);
                    if (p) {
                        state.custom.push(p);
                    }
                }
            }
            if (parsed && typeof parsed.defaultId === 'string') {
                state.defaultId = parsed.defaultId;
            }
        } catch (e) { /* factory state */ }
        if (!find(state.defaultId)) {
            state.defaultId = FACTORY_DEFAULT;
        }
        return state;
    }

    function write() {
        listCache = null;
        try {
            if (global.localStorage && global.JSON) {
                var custom = [];
                var i;
                for (i = 0; i < state.custom.length; i++) {
                    custom.push({ id: state.custom[i].id, label: state.custom[i].label, values: state.custom[i].values });
                }
                global.localStorage.setItem(STORAGE_KEY, global.JSON.stringify({
                    custom: custom,
                    defaultId: state.defaultId
                }));
            }
        } catch (e) { /* in-memory only */ }
    }

    /** Built-ins then custom presets; the same array until a preset is saved or deleted. */
    function list() {
        if (!listCache) {
            listCache = BUILTIN.concat(read().custom);
        }
        return listCache;
    }

    function find(id) {
        var all = BUILTIN.concat(state ? state.custom : []);
        var i;
        for (i = 0; i < all.length; i++) {
            if (all[i].id === id) {
                return all[i];
            }
        }
        return null;
    }

    function optionValue(options, t) {
        var v = options ? options[t.key] : undefined;
        return (v === undefined || v === null) ? t.def : !!v;
    }

    function matches(preset, options) {
        var i;
        for (i = 0; i < TWEAKS.length; i++) {
            if (optionValue(options, TWEAKS[i]) !== preset.values[TWEAKS[i].key]) {
                return false;
            }
        }
        return true;
    }

    /** The preset whose tweaks equal options (built-ins first), or null for a one-off mix. */
    function match(options) {
        var all = list();
        var i;
        for (i = 0; i < all.length; i++) {
            if (matches(all[i], options)) {
                return all[i];
            }
        }
        return null;
    }

    function apply(preset, options) {
        var i;
        if (!preset || !options) {
            return;
        }
        for (i = 0; i < TWEAKS.length; i++) {
            options[TWEAKS[i].key] = preset.values[TWEAKS[i].key];
        }
    }

    /** Saves options under name; a custom preset of the same name is replaced. */
    function save(name, options) {
        var s = read();
        var label = String(name || '').replace(/\s+/g, ' ').replace(/^\s+|\s+$/g, '').substring(0, NAME_MAX);
        var values = {};
        var i;
        if (!label) {
            return null;
        }
        for (i = 0; i < TWEAKS.length; i++) {
            values[TWEAKS[i].key] = optionValue(options, TWEAKS[i]);
        }
        for (i = 0; i < s.custom.length; i++) {
            if (s.custom[i].label.toLowerCase() === label.toLowerCase()) {
                s.custom[i] = { id: s.custom[i].id, label: label, builtin: false, blurb: '', values: values };
                write();
                return s.custom[i];
            }
        }
        if (s.custom.length >= MAX_CUSTOM) {
            return null;
        }
        var preset = { id: 'custom-' + new Date().getTime().toString(36), label: label, builtin: false, blurb: '', values: values };
        s.custom.push(preset);
        write();
        return preset;
    }

    function remove(id) {
        var s = read();
        var i;
        for (i = 0; i < s.custom.length; i++) {
            if (s.custom[i].id === id) {
                s.custom.splice(i, 1);
                if (s.defaultId === id) {
                    s.defaultId = FACTORY_DEFAULT;
                }
                write();
                return true;
            }
        }
        return false;
    }

    function getDefault() {
        return find(read().defaultId) || BUILTIN[0];
    }

    function setDefault(id) {
        if (find(id)) {
            read().defaultId = id;
            write();
        }
    }

    global.CncLobbyPresets = {
        TWEAKS: TWEAKS,
        MAX_CUSTOM: MAX_CUSTOM,
        NAME_MAX: NAME_MAX,
        list: list,
        find: function (id) { read(); return find(id); },
        match: match,
        apply: apply,
        save: save,
        remove: remove,
        getDefault: getDefault,
        setDefault: setDefault,
        canSaveMore: function () { return read().custom.length < MAX_CUSTOM; }
    };
})(window);
