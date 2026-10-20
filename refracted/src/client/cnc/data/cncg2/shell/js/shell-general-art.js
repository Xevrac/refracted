/**
 * Aurora main menu: art for the general to show (the last one played, else the lobby default).
 *
 * 1. Menu art: images/general-art/<file>, one cut-out per general (MENU_ART below). Drawn to fill
 *    the square general box on the nav bar, anchored bottom right ("fit"). Sizes in the shell
 *    README ("General art").
 * 2. A general without menu art uses the game's loading-screen art (UTFWinAssets, served by
 *    Refracted at /cnc/utfwin/images/LoadingScreen/). "edge" is the figure's right edge as a
 *    fraction of the 1920px canvas; the image is slid right by (1 - edge).
 */
(function (global) {
    'use strict';

    var MENU_ART_DIR = 'images/general-art/';
    var ART_DIR = '/cnc/utfwin/images/LoadingScreen/';
    var ICON_DIR = 'view/image/generals/';

    // File names as supplied (they follow the game's own art names, not the general keys).
    var MENU_ART = {
        APA_ClassicGeneral: 'AG_Basic_01_UP.png',
        APA_AtomicGeneral: 'AG_Atomic_01.png',
        APA_EspionageGeneral: 'AGEspionage01.png',
        APA_FlameGeneral: 'AG_Tank_01_UP.png',
        APA_RocketGeneral: 'AGRocket01.png',
        APA_UrbanGeneral: 'AGUrbanCombat01.png',
        EU_ClassicGeneral: 'EgBasic01.png',
        EU_AirforceGeneral: 'EGAirForce01.png',
        EU_LaserGeneral: 'EG_ExperimentalWeapons_01_UP.png',
        EU_RapidAssaultGeneral: 'EGRapidAssault01.png',
        EU_ReconGeneral: 'EGRecon01.png',
        EU_ScienceGeneral: 'EG_Railgun_01_UP.png',
        GLA_ClassicGeneral: 'GGBasic01.png',
        GLA_DemolitionGeneral: 'GGDemolition01.png',
        GLA_EngineerGeneral: 'GGEngineer01.png',
        GLA_JunkyardGeneral: 'GGSalvage01.png',
        GLA_ToxinGeneral: 'GGToxin01.png'
    };

    // Game loading-screen art for generals without menu art.
    var ART = {
        GLA_MarauderGeneral: { file: 'LoadingScreen_GGMarauder01.png', edge: 0.571 }
    };

    // 64x64 portrait icons (shell copies of UTFWinAssets "Generals 64x64").
    var ICON = {
        APA_ClassicGeneral: 'AG_Basic_01.png', APA_AtomicGeneral: 'AG_Atomic_01.png',
        APA_EspionageGeneral: 'AG_Espionage_01.png', APA_FlameGeneral: 'AG_Tank_01.png',
        APA_RocketGeneral: 'AG_Rocket_01.png', APA_UrbanGeneral: 'AG_UrbanCombat_01.png',
        EU_ClassicGeneral: 'EG_Basic_01.png', EU_AirforceGeneral: 'EG_Airforce_01.png',
        EU_LaserGeneral: 'EG_ExperimentalWeapons_01.png', EU_RapidAssaultGeneral: 'EG_RapidAssault_01.png',
        EU_ReconGeneral: 'EG_Recon_01.png', EU_ScienceGeneral: 'EG_Railgun_01.png',
        GLA_ClassicGeneral: 'GG_Basic_01.png', GLA_DemolitionGeneral: 'GG_Demolition_01.png',
        GLA_EngineerGeneral: 'GG_Engineer_01.png', GLA_JunkyardGeneral: 'GG_Salvage_01.png',
        GLA_MarauderGeneral: 'GG_Marauder_01.png', GLA_ToxinGeneral: 'GG_Toxin_01.png'
    };

    function normalizeFaction(f) {
        var c = String(f || '').toUpperCase();
        return c === 'ESC' ? 'EU' : c;
    }

    /** { faction, general } for a general id, searching every faction. */
    function findById(generalId) {
        var factions = ['APA', 'EU', 'GLA'];
        var i, j, list;
        if (!global.CncLobbyDefaults || !CncLobbyDefaults.generals || !generalId) {
            return null;
        }
        for (i = 0; i < factions.length; i++) {
            list = CncLobbyDefaults.generals(factions[i], true);
            for (j = 0; j < list.length; j++) {
                if (Number(list[j].id) === Number(generalId)) {
                    return { faction: factions[i], general: list[j] };
                }
            }
        }
        return null;
    }

    /**
     * The general to show: `last` ({ generalId }, the last match played, from Refracted) when it
     * resolves, else the preferred general from the lobby defaults in OPTIONS. Null when neither.
     */
    function current(last) {
        var playtest = !!(global.CncPlaytest && CncPlaytest.on && CncPlaytest.on());
        if (playtest && last && !CncPlaytest.allowsGeneral(last.generalId)) {
            last = null; // the playtest plays EU Classic / Airforce / Experimental only
        }
        var hit = last && last.generalId ? findById(last.generalId) : null;
        var random = false;
        var prefs;
        if (!hit && global.CncLobbyDefaults && CncLobbyDefaults.get) {
            prefs = CncLobbyDefaults.get();
            if (global.CncPlaytest && CncPlaytest.on && CncPlaytest.on()) {
                prefs.random = false;
                prefs.faction = 'EU';
                if (!CncPlaytest.allowsGeneral(prefs.general)) {
                    prefs.general = CncPlaytest.euClassicId;
                }
            }
            random = !!prefs.random;
            hit = findById(prefs.general);
            if (hit) {
                hit.faction = normalizeFaction(prefs.faction) || hit.faction;
            }
        }
        if (!hit) {
            return null;
        }
        var general = hit.general;
        // A tutorial general has no art of its own; it wears its faction's classic general.
        var key = String(general.key).replace('TutorialGeneral', 'ClassicGeneral');
        var out = {
            key: key,
            line: (last && last.generalId && !random)
                ? (hit.faction + ' ' + general.label)
                : (random ? 'Random general each match' : (hit.faction + ' ' + general.label + ' general')),
            icon: ICON[key] ? (ICON_DIR + ICON[key]) : '',
            src: '',
            fit: false,
            bust: false,
            style: {}
        };
        var art;
        if (MENU_ART[key]) {
            out.src = MENU_ART_DIR + MENU_ART[key];
            out.fit = true;
        } else if ((art = ART[key])) {
            // +4: the figure runs a little off the right edge, as on the concept sheet.
            var shift = Math.max(0, Math.round((1 - art.edge) * 100) + 4);
            out.src = ART_DIR + art.file;
            out.style = {
                '-webkit-transform': 'translateX(' + shift + '%)',
                'transform': 'translateX(' + shift + '%)'
            };
        }
        return out;
    }

    global.CncGeneralArt = {
        current: current
    };
}(window));
