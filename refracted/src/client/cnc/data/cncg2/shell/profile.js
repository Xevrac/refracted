/**
 * Profile Controller
 * Profile pages read GET /cnc/profile (Refracted, Nexus mysql): wins, losses, streaks, every
 * reported match (date + result) and the wins leaderboard. Nothing else is recorded yet, so
 * per-faction statistics, levels, ratings and achievements show as not recorded ('—' / empty).
 */
CCApp.controller('ProfileController', function($scope, $rootScope, $timeout) {
    var NONE = '—';
    var MONTHS = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];

    function dayLabel(d) {
        return MONTHS[d.getMonth()] + ' ' + d.getDate() + ', ' + d.getFullYear();
    }
    /** 754 -> "12:34", 4000 -> "1:06:40", unknown -> '—'. */
    function clock(secs) {
        if (secs === null || secs === undefined || secs === '' || isNaN(secs)) {
            return NONE;
        }
        secs = Math.max(0, Math.round(Number(secs)));
        var h = Math.floor(secs / 3600);
        var m = Math.floor(secs / 60) % 60;
        var s = secs % 60;
        var mm = (h && m < 10 ? '0' : '') + m;
        return (h ? h + ':' : '') + mm + ':' + (s < 10 ? '0' : '') + s;
    }
    $scope.clock = clock;

    /** "APA Classic" for a general id, '' when unknown. */
    function generalName(id) {
        var factions = ['APA', 'EU', 'GLA'];
        var i, j, list;
        if (!id || !window.CncLobbyDefaults || !CncLobbyDefaults.generals) {
            return '';
        }
        for (i = 0; i < factions.length; i++) {
            list = CncLobbyDefaults.generals(factions[i], true);
            for (j = 0; j < list.length; j++) {
                if (Number(list[j].id) === Number(id)) {
                    return factions[i] + ' ' + list[j].label;
                }
            }
        }
        return '';
    }

    function dayKey(d) {
        return d.getFullYear() * 10000 + (d.getMonth() + 1) * 100 + d.getDate();
    }

    // ==========================================
    // OVERVIEW
    // ==========================================
    $scope.profileLoaded = false;
    // True when Refracted answered but has no match records to give (no Nexus database: 503).
    $scope.profileUnavailable = false;
    $scope.profileOverview = {
        wins: 0, losses: 0, winRate: 0, winStreak: 0, lossStreak: 0,
        shortestMatch: NONE, bestWin: NONE, lastGeneral: NONE,
        generalsUnlocked: NONE, generalsTotal: NONE,
        recentGames: [], achievements: []
    };

    // ==========================================
    // STATISTICS (per faction is not recorded)
    // ==========================================
    $scope.statsActiveFaction = 'apa';
    $scope.setStatsFaction = function(factionId) {
        $scope.statsActiveFaction = factionId;
    };
    function emptyFactionRow() {
        return { wins: NONE, losses: NONE, income: NONE, xp: NONE, resources: NONE, unitsBuilt: NONE,
                 unitsDestroyed: NONE, unitsLost: NONE, structBuilt: NONE, structDestroyed: NONE, structLost: NONE };
    }
    $scope.factionStats = { apa: emptyFactionRow(), gla: emptyFactionRow(), eu: emptyFactionRow() };
    $scope.factionStatsRecorded = false;
    $scope.getWinPct = function() {
        var o = $scope.profileOverview;
        var total = o.wins + o.losses;
        return total ? (o.wins / total) * 100 : 50;
    };

    // ==========================================
    // MATCH HISTORY (per local day)
    // ==========================================
    $scope.matchHistoryData = [];

    // ==========================================
    // ACHIEVEMENTS (not recorded)
    // ==========================================
    $scope.achievementsList = [];

    // ==========================================
    // LEADERBOARDS (by wins, then fewest losses)
    // ==========================================
    $scope.leaderboardNearMe = [];
    $scope.leaderboardTop = [];
    $scope.leaderboardRanked = 0;

    function boardRows(rows, me) {
        var out = [];
        for (var i = 0; rows && i < rows.length; i++) {
            out.push({
                rank: rows[i].rank,
                name: rows[i].name,
                level: NONE,
                wins: rows[i].wins,
                losses: rows[i].losses,
                rating: NONE,
                isMe: rows[i].personaId === me
            });
        }
        return out;
    }

    /** Record totals: from /cnc/profile, or from /cnc/match-record (matchRecord) when that is all the
        server offers (Refracted builds before the profile endpoint). */
    function applyRecord(data) {
        var o = $scope.profileOverview;
        o.wins = data.wins || 0;
        o.losses = data.losses || 0;
        o.winStreak = data.winStreak || 0;
        o.lossStreak = data.lossStreak || 0;
        o.winRate = (o.wins + o.losses) ? Math.round(o.wins / (o.wins + o.losses) * 100) : 0;
        o.shortestMatch = data.shortestMatchSecs ? clock(data.shortestMatchSecs) : NONE;
        o.bestWin = data.bestWinSecs ? clock(data.bestWinSecs) : NONE;
        o.lastGeneral = generalName(data.lastGeneralId) || NONE;
    }

    function applyProfile(data) {
        var o = $scope.profileOverview;
        applyRecord(data);

        var history = data.history || [];
        var recent = [];
        var days = [];
        var byDay = {};
        for (var i = 0; i < history.length; i++) {
            var d = new Date(history[i].at * 1000);
            var win = !!history[i].victory;
            if (recent.length < 10) {
                recent.push({
                    date: dayLabel(d),
                    result: win ? 'VICTORY' : 'DEFEAT',
                    general: generalName(history[i].generalId),
                    length: history[i].secs === null || history[i].secs === undefined ? '' : clock(history[i].secs)
                });
            }
            var k = dayKey(d);
            var row = byDay[k];
            if (!row) {
                row = byDay[k] = { date: dayLabel(d), defeats: 0, victories: 0, defPct: 0, vicPct: 0 };
                days.push(row);
            }
            if (win) { row.victories++; } else { row.defeats++; }
        }
        for (var j = 0; j < days.length; j++) {
            var n = days[j].victories + days[j].defeats;
            days[j].vicPct = Math.round(days[j].victories / n * 100);
            days[j].defPct = 100 - days[j].vicPct;
        }
        o.recentGames = recent;
        $scope.matchHistoryData = days;

        $scope.leaderboardTop = boardRows(data.top, data.personaId);
        $scope.leaderboardNearMe = boardRows(data.near, data.personaId);
        $scope.leaderboardRanked = data.ranked || 0;
        $scope.profileLoaded = true;
        $scope.profileUnavailable = false;
    }

    function loadProfile() {
        var key = '';
        try {
            if (window.CncPreLanding && CncPreLanding.getLoginKey) {
                key = CncPreLanding.getLoginKey() || '';
            }
        } catch (e) { /* ignore */ }
        var pid = 0;
        try {
            if (window.CncBlazeState && CncBlazeState.getPersonaId) {
                pid = CncBlazeState.getPersonaId() || 0;
            }
        } catch (e2) { /* ignore */ }
        var url = '/cnc/profile?key=' + encodeURIComponent(key) +
            '&pid=' + encodeURIComponent(String(pid || ''));
        try {
            var xhr = new XMLHttpRequest();
            xhr.open('GET', url, true);
            xhr.onreadystatechange = function () {
                if (xhr.readyState !== 4) {
                    return;
                }
                if (xhr.status !== 200) {
                    if (xhr.status === 503) {
                        $timeout(function () { $scope.profileUnavailable = true; }, 0);
                    }
                    return;
                }
                var data = null;
                try {
                    data = JSON.parse(xhr.responseText);
                } catch (pe) { /* ignore */ }
                if (!data || !data.ok) {
                    return;
                }
                if ($rootScope.$$phase) {
                    applyProfile(data);
                } else {
                    $scope.$apply(function () { applyProfile(data); });
                }
            };
            xhr.send(null);
        } catch (e2) { /* ignore */ }
    }

    // Reload when the record changes (a reported match) and whenever Profile is opened.
    $rootScope.$watch('matchRecord', function (rec) {
        if (rec) {
            applyRecord(rec);
        }
        $timeout(loadProfile, 0);
    }, true);
    $scope.$watch('activeTab', function (tab) {
        if (tab === 'PROFILE') {
            loadProfile();
        }
    });
});
