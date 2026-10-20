/**
 * Playtest shell (devWrapper.html / DebugShell).
 */
(function (window) {
    'use strict';

    var EU_CLASSIC_ID = 232716472;
    // Playtest generals: EU Classic, plus EU Airforce and EU Experimental Weapons (Laser).
    var PLAYTEST_GENERAL_IDS = [EU_CLASSIC_ID, 76852445, 162564633];

    function allowsGeneral(id) {
        var n = Number(id);
        for (var i = 0; i < PLAYTEST_GENERAL_IDS.length; i++) {
            if (PLAYTEST_GENERAL_IDS[i] === n) {
                return true;
            }
        }
        return false;
    }

    function on() {
        return window.__CNC_PLAYTEST === true;
    }

    function isSmalltown(map) {
        if (!map) {
            return false;
        }
        var id = String(map.id || '').toLowerCase();
        var path = String(map.path || '');
        var label = String(map.label || '');
        return id === 'smalltown' ||
            path.indexOf('DM_Smalltown') !== -1 ||
            label === 'Smalltown';
    }

    window.CncPlaytest = {
        euClassicId: EU_CLASSIC_ID,
        generalIds: PLAYTEST_GENERAL_IDS,
        allowsGeneral: allowsGeneral,
        statusLabel: 'Not Available',
        on: on,
        isSmalltown: isSmalltown,
        mapBlocked: function (map) {
            return on() && !isSmalltown(map);
        },
        coerceSlot: function (slot) {
            if (!on() || !slot) {
                return false;
            }
            // A human observer plays no faction, so EU-only does not apply to it.
            if (!slot.isAi && String(slot.faction || '').toUpperCase() === 'OBS') {
                return false;
            }
            var changed = false;
            if (String(slot.faction || '').toUpperCase() !== 'EU') {
                slot.faction = 'EU';
                changed = true;
            }
            if (!allowsGeneral(slot.general)) {
                slot.general = EU_CLASSIC_ID;
                changed = true;
            }
            return changed;
        }
    };
}(window));
