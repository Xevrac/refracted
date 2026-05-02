/**
 * Profile Controller
 * Manages the specific views and data rendering for the Profile Tab
 */
CCApp.controller('ProfileController', function($scope, $rootScope) {
    
    // ==========================================
    // PROFILE OVERVIEW DATA (name: use playerName in templates / $root)
    // ==========================================
    $scope.profileOverview = {
        wins: 51, generalsUnlocked: 4, generalsTotal: 18,
        recentGames: [ { date: "Oct 16, 2013", result: "DEFEAT" }, { date: "Oct 16, 2013", result: "DEFEAT" } ]
    };

    // ==========================================
    // STATISTICS DATA
    // ==========================================
    $scope.statsActiveFaction = 'apa'; 
    
    $scope.setStatsFaction = function(factionId) { 
        $scope.statsActiveFaction = factionId; 
    };

    $scope.factionStats = {
        apa: { wins: 46, losses: 22, income: "4,463", xp: "21,683", resources: "1,991,372", unitsBuilt: "2,695", unitsDestroyed: "1,475", unitsLost: "490", structBuilt: "696", structDestroyed: "300", structLost: "127" },
        gla: { wins: 12, losses: 8, income: "3,100", xp: "8,450", resources: "850,120", unitsBuilt: "1,450", unitsDestroyed: "890", unitsLost: "310", structBuilt: "210", structDestroyed: "150", structLost: "85" },
        eu: { wins: 30, losses: 15, income: "5,200", xp: "15,800", resources: "1,500,000", unitsBuilt: "2,100", unitsDestroyed: "1,200", unitsLost: "400", structBuilt: "500", structDestroyed: "220", structLost: "90" }
    };

    $scope.getWinPct = function(factionId) {
        var data = $scope.factionStats[factionId];
        var total = data.wins + data.losses;
        if (total === 0) return 50; 
        return (data.wins / total) * 100;
    };

    // ==========================================
    // MATCH HISTORY DATA
    // ==========================================
    $scope.matchHistoryData = [
        { date: "OCT 8, 2013", defeats: 0, victories: 2, defPct: 0, vicPct: 100 },
        { date: "OCT 6, 2013", defeats: 0, victories: 1, defPct: 0, vicPct: 100 },
        { date: "OCT 4, 2013", defeats: 0, victories: 2, defPct: 0, vicPct: 100 },
        { date: "OCT 3, 2013", defeats: 0, victories: 5, defPct: 0, vicPct: 100 },
        { date: "SEP 25, 2013", defeats: 1, victories: 5, defPct: 20, vicPct: 80 },
        { date: "SEP 24, 2013", defeats: 0, victories: 1, defPct: 0, vicPct: 100 },
        { date: "SEP 22, 2013", defeats: 2, victories: 1, defPct: 65, vicPct: 35 }
    ];

    // ==========================================
    // ACHIEVEMENTS DATA
    // ==========================================
    $scope.achievementsList = [
        { name: "No Contest", icon: "view/image/debugPinkRect.png", locked: false, multiplier: "10:00\nX" },
        { name: "Demolitionist", icon: "view/image/debugPinkRect.png", locked: false, multiplier: "x15" },
        { name: "Peoples Hero", icon: "view/image/debugPinkRect.png", locked: false, multiplier: "x20" },
        { name: "Napoleon Redux", icon: "view/image/debugPinkRect.png", locked: true, multiplier: "x20", progress: 25 },
        { name: "Welcome to the Watch", icon: "view/image/debugPinkRect.png", locked: true, multiplier: "", progress: 50 }
    ];

    // ==========================================
    // LEADERBOARDS DATA
    // ==========================================
    $scope.leaderboardNearMe = [
        { rank: 240, name: "lamnothere2", level: 2, wins: 3, losses: 0, rating: "1,259", isMe: false },
        { rank: 241, name: "FunkerVoigt", level: 3, wins: 12, losses: 1, rating: "1,259", isMe: false },
        { rank: 242, name: "UnknownPlayer", level: 19, wins: 51, losses: 25, rating: "1,258", isMe: true },
        { rank: 243, name: "Prinzip1914", level: 16, wins: 26, losses: 0, rating: "1,258", isMe: false },
        { rank: 244, name: "LittleJohn", level: 8, wins: 4, losses: 2, rating: "1,258", isMe: false }
    ];

    function syncLocalPlayerOnLeaderboard(name) {
        var n = (name != null && String(name) !== "") ? name : "UnknownPlayer";
        for (var j = 0; j < $scope.leaderboardNearMe.length; j++) {
            if ($scope.leaderboardNearMe[j].isMe) {
                $scope.leaderboardNearMe[j].name = n;
                break;
            }
        }
    }
    $rootScope.$watch("playerName", syncLocalPlayerOnLeaderboard);
    syncLocalPlayerOnLeaderboard($rootScope.playerName);

    $scope.leaderboardTop = [];
    var mockTopNames = ["edi211188", "Dabrifa", "ZxGanon_the_Boss", "Brossea", "ORBIT_Hexis", "CYC_FuryDE"];
    for (var i = 1; i <= 50; i++) {
        $scope.leaderboardTop.push({
            rank: i, name: mockTopNames[i-1] || ("Commander_" + (900 + i)), level: Math.floor(Math.random() * 40) + 10,
            wins: Math.floor(Math.random() * 300) + 100, losses: Math.floor(Math.random() * 50) + 1,
            rating: (2255 - (i * 12)).toLocaleString(), isMe: false
        });
    }

});