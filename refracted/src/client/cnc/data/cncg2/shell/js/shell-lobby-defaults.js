/**
 * Preferred lobby roster defaults (faction, general, colour).
 * Persists in the shell WebKit profile localStorage (CNCO_DL\0\webkit on retail).
 */
(function (global) {
    'use strict';

    var STORAGE_KEY = 'cnc_lobby_defaults';

    var COLORS = [
        { value: '#3a7bd5', label: 'Blue' },
        { value: '#2aa8a0', label: 'Teal' },
        { value: '#3fbf4a', label: 'Green' },
        { value: '#e6b322', label: 'Gold' },
        { value: '#e67e22', label: 'Orange' },
        { value: '#c0392b', label: 'Red' },
        { value: '#ececec', label: 'White' },
        { value: '#8e44ad', label: 'Purple' }
    ];

    var FACTIONS = [
        { code: 'APA', label: 'APA' },
        { code: 'EU', label: 'EU' },
        { code: 'GLA', label: 'GLA' }
    ];

    var GENERALS_BY_FACTION = {
        APA: [
            { id: 2914080600, key: 'APA_ClassicGeneral', label: 'Classic' },
            { id: 3919700239, key: 'APA_AtomicGeneral', label: 'Atomic' },
            { id: 1259669531, key: 'APA_EspionageGeneral', label: 'Espionage' },
            { id: 73570905, key: 'APA_FlameGeneral', label: 'Flame' },
            { id: 1113071128, key: 'APA_RocketGeneral', label: 'Rocket' },
            { id: 3042903044, key: 'APA_UrbanGeneral', label: 'Urban' },
            { id: 497011786, key: 'APA_TutorialGeneral', label: 'Tutorial' }
        ],
        EU: [
            { id: 232716472, key: 'EU_ClassicGeneral', label: 'Classic' },
            { id: 76852445, key: 'EU_AirforceGeneral', label: 'Airforce' },
            { id: 162564633, key: 'EU_LaserGeneral', label: 'Laser' },
            { id: 1523019091, key: 'EU_RapidAssaultGeneral', label: 'Rapid Assault' },
            { id: 2373292237, key: 'EU_ReconGeneral', label: 'Recon' },
            { id: 3943761922, key: 'EU_ScienceGeneral', label: 'Science' },
            { id: 3463861546, key: 'EU_TutorialGeneral', label: 'Tutorial' }
        ],
        GLA: [
            { id: 580378690, key: 'GLA_ClassicGeneral', label: 'Classic' },
            { id: 1632710820, key: 'GLA_DemolitionGeneral', label: 'Demolition' },
            { id: 1975634889, key: 'GLA_EngineerGeneral', label: 'Engineer' },
            { id: 2905920036, key: 'GLA_JunkyardGeneral', label: 'Junkyard' },
            { id: 2837579969, key: 'GLA_MarauderGeneral', label: 'Marauder' },
            { id: 901813698, key: 'GLA_ToxinGeneral', label: 'Toxin' },
            { id: 145977592, key: 'GLA_TutorialGeneral', label: 'Tutorial' }
        ]
    };

    var FACTORY = {
        faction: 'APA',
        general: 2914080600,
        color: '#3a7bd5'
    };

    function normalizeFaction(code) {
        var c = String(code || 'APA').toUpperCase();
        if (c === 'ESC') {
            return 'EU';
        }
        if (c === 'EU' || c === 'GLA' || c === 'APA') {
            return c;
        }
        return 'APA';
    }

    function generalsForFaction(code, includeTutorial) {
        var list = GENERALS_BY_FACTION[normalizeFaction(code)] || [];
        if (includeTutorial) {
            return list.slice();
        }
        var out = [];
        var i;
        for (i = 0; i < list.length; i++) {
            if (!/Tutorial/i.test(list[i].key)) {
                out.push(list[i]);
            }
        }
        return out;
    }

    function classicGeneralId(code) {
        var list = generalsForFaction(code, true);
        var i;
        for (i = 0; i < list.length; i++) {
            if (/Classic/i.test(list[i].key) && list[i].id) {
                return list[i].id;
            }
        }
        return list.length ? list[0].id : 0;
    }

    function tutorialGeneralId(code) {
        var list = generalsForFaction(code, true);
        var i;
        for (i = 0; i < list.length; i++) {
            if (/Tutorial/i.test(list[i].key) && list[i].id) {
                return list[i].id;
            }
        }
        return 0;
    }

    function normalizeColor(value) {
        var raw = String(value || '').toLowerCase();
        var i;
        for (i = 0; i < COLORS.length; i++) {
            if (String(COLORS[i].value).toLowerCase() === raw) {
                return COLORS[i].value;
            }
        }
        return FACTORY.color;
    }

    function normalizeGeneral(faction, generalId) {
        var id = Number(generalId) || 0;
        var list = generalsForFaction(faction, true);
        var i;
        for (i = 0; i < list.length; i++) {
            if (list[i].id === id) {
                return id;
            }
        }
        return classicGeneralId(faction);
    }

    function normalizePrefs(raw) {
        var prefs = {
            random: !!(raw && raw.random),
            faction: normalizeFaction(raw && raw.faction),
            general: normalizeGeneral(raw && raw.faction, raw && raw.general),
            color: normalizeColor(raw && raw.color)
        };
        prefs.general = normalizeGeneral(prefs.faction, prefs.general);
        return prefs;
    }

    function pickRandomIndex(length) {
        if (!length) {
            return 0;
        }
        return Math.floor(Math.random() * length);
    }

    function rollRandomPrefs(map) {
        var forcedFaction = defaultFactionForMap(map);
        var faction;
        if (forcedFaction) {
            faction = forcedFaction;
        } else {
            faction = FACTIONS[pickRandomIndex(FACTIONS.length)].code;
        }

        var generals = generalsForFaction(faction, false);
        if (!generals.length) {
            generals = generalsForFaction(faction, true);
        }
        var general = generals.length
            ? generals[pickRandomIndex(generals.length)].id
            : classicGeneralId(faction);
        if (map && map.forceTutorialGeneral) {
            general = defaultGeneralForMap(faction, map);
        }

        return {
            random: true,
            faction: faction,
            general: general,
            color: COLORS[pickRandomIndex(COLORS.length)].value
        };
    }

    function resolvePrefsForLobby(map) {
        var stored = readLocalPrefs();
        if (stored.random) {
            return rollRandomPrefs(map);
        }

        var prefs = {
            random: false,
            faction: stored.faction,
            general: stored.general,
            color: stored.color
        };
        var forcedFaction = defaultFactionForMap(map);
        if (forcedFaction) {
            prefs.faction = forcedFaction;
            prefs.general = defaultGeneralForMap(prefs.faction, map);
        }
        return prefs;
    }

    function emptyPrefs() {
        return normalizePrefs(FACTORY);
    }

    function readLocalPrefs() {
        try {
            if (!global.localStorage || !global.JSON) {
                return emptyPrefs();
            }
            var raw = global.localStorage.getItem(STORAGE_KEY);
            if (!raw) {
                return emptyPrefs();
            }
            return normalizePrefs(global.JSON.parse(raw));
        } catch (e) {
            return emptyPrefs();
        }
    }

    function writeLocalPrefs(prefs) {
        var payload = normalizePrefs(prefs);
        try {
            if (global.localStorage && global.JSON) {
                global.localStorage.setItem(STORAGE_KEY, global.JSON.stringify(payload));
            }
        } catch (e) { /* ignore */ }
        return payload;
    }

    function defaultFactionForMap(map) {
        if (map && map.forceFaction) {
            return normalizeFaction(map.forceFaction);
        }
        if (map && map.forceTutorialGeneral) {
            return 'EU';
        }
        return null;
    }

    function defaultGeneralForMap(faction, map) {
        if (map && map.forceTutorialGeneral) {
            return tutorialGeneralId(faction);
        }
        return classicGeneralId(faction);
    }

    function applyToSlot(slot, map) {
        if (!slot) {
            return slot;
        }
        var prefs = resolvePrefsForLobby(map);
        slot.faction = prefs.faction;
        slot.general = prefs.general;
        slot.color = prefs.color;
        return slot;
    }

    function get() {
        return readLocalPrefs();
    }

    function set(partial) {
        var merged = normalizePrefs(global.jQuery
            ? global.jQuery.extend({}, readLocalPrefs(), partial || {})
            : (function () {
                var base = readLocalPrefs();
                var key;
                for (key in partial) {
                    if (Object.prototype.hasOwnProperty.call(partial, key)) {
                        base[key] = partial[key];
                    }
                }
                return base;
            }()));
        writeLocalPrefs(merged);
        return merged;
    }

    function save(prefs) {
        return writeLocalPrefs(prefs || readLocalPrefs());
    }

    function restore() {
        return save(FACTORY);
    }

    function coerceForFaction(prefs) {
        var next = normalizePrefs(prefs || readLocalPrefs());
        next.general = normalizeGeneral(next.faction, next.general);
        return next;
    }

    global.CncLobbyDefaults = {
        STORAGE_KEY: STORAGE_KEY,
        FACTORY: emptyPrefs(),
        COLORS: COLORS,
        FACTIONS: FACTIONS,
        get: get,
        set: set,
        save: save,
        restore: restore,
        colors: function () { return COLORS.slice(); },
        factions: function () { return FACTIONS.slice(); },
        generals: function (faction, includeTutorial) {
            return generalsForFaction(faction, !!includeTutorial);
        },
        coerceForFaction: coerceForFaction,
        applyToSlot: applyToSlot,
        resolveForLobby: resolvePrefsForLobby,
        rollRandom: rollRandomPrefs,
        normalize: normalizePrefs,
        isRandom: function () {
            return !!readLocalPrefs().random;
        }
    };
}(window));
