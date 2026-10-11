/**
 * CCApp - Command & Conquer UI Replica
 * Targeting late 2011 WebKit environment
 */

var ccConfig = function($routeProvider) {
    $routeProvider
        .when('/', {
            template: '<div></div>'
        })
        .otherwise({
            redirectTo: '/'
        });
};

var CCApp = angular.module('CCApp', ['ngResource']).config(ccConfig);

CCApp.run(function ($rootScope) {
    var preDone = window.CncPreLanding && CncPreLanding.isSessionDone && CncPreLanding.isSessionDone();
    $rootScope.preLandingComplete = !!preDone;
    if (window.CncPreLanding && CncPreLanding.getTagline) {
        $rootScope.preLandingTagline = CncPreLanding.getTagline();
    } else {
        $rootScope.preLandingTagline = '';
    }
    $rootScope.preLandingStatus = ' ';
    $rootScope.preLandingHasShell =
        window.CncPreLanding && CncPreLanding.hasShell && CncPreLanding.hasShell();
    $rootScope.preLandingUseArt = false;
    $rootScope.allowShellThemeSelect = true;
    $rootScope.playtest = !!(window.CncPlaytest && CncPlaytest.on && CncPlaytest.on());
    $rootScope.showDebugTools = false;
    $rootScope.lobbyOpen = false;
    $rootScope.lobbyView = 'matchmake';

    $rootScope.openLobby = function () {
        $rootScope.lobbyOpen = true;
        $rootScope.lobbyView = 'matchmake';
    };
    // Aurora "Find a match": no map pick. The lobby controller consumes quickMatchPending and joins
    // the lowest-ping populated server (an idle one when none is populated).
    $rootScope.quickMatch = function () {
        $rootScope.quickMatchPending = true;
        $rootScope.lobbyOpen = true;
        $rootScope.lobbyView = 'matchmake';
    };
    $rootScope.openGameRoom = function () {
        $rootScope.lobbyOpen = true;
        $rootScope.lobbyView = 'gameroom';
    };
    $rootScope.exitLobby = function () {
        $rootScope.lobbyOpen = false;
        $rootScope.lobbyView = 'matchmake';
    };
    $rootScope.openServerBrowser = function () {
        $rootScope.lobbyOpen = true;
        $rootScope.lobbyView = 'browser';
    };
    $rootScope.closeServerBrowser = function () {
        $rootScope.lobbyView = 'matchmake';
    };
    $rootScope.leaveGameRoom = function () {
        $rootScope.$broadcast('cnc:leaveGameRoom');
        $rootScope.lobbyView = 'matchmake';
    };

    function syncThemeTemplates(id) {
        var themeId = (window.CncShellTheme && CncShellTheme.normalize)
            ? CncShellTheme.normalize(id)
            : (id || 'aurora');
        $rootScope.shellUiTheme = themeId;
        $rootScope.shellRootTemplate = (window.CncShellTheme && CncShellTheme.rootTemplate)
            ? CncShellTheme.rootTemplate(themeId)
            : 'view/roots/aurora.html';
        $rootScope.homeTemplateUrl = (window.CncShellTheme && CncShellTheme.homeTemplate)
            ? CncShellTheme.homeTemplate(themeId)
            : 'view/home-aurora.html';
        $rootScope.lobbyTemplateUrl = (window.CncShellTheme && CncShellTheme.lobbyTemplate)
            ? CncShellTheme.lobbyTemplate(themeId)
            : 'view/lobby/lobby-aurora.html';
    }

    syncThemeTemplates(window.CncShellTheme ? CncShellTheme.get() : 'aurora');

    if (window.CncShellTheme) {
        if (CncShellTheme.onChange) {
            CncShellTheme.onChange(function (themeId) {
                if ($rootScope.$$phase) {
                    syncThemeTemplates(themeId);
                } else {
                    $rootScope.$apply(function () {
                        syncThemeTemplates(themeId);
                    });
                }
            });
        }
        if (CncShellTheme.boot) {
            CncShellTheme.boot(function (themeId) {
                if ($rootScope.$$phase) {
                    syncThemeTemplates(themeId);
                } else {
                    $rootScope.$apply(function () {
                        syncThemeTemplates(themeId);
                    });
                }
            });
        } else if (CncShellTheme.apply) {
            CncShellTheme.apply(CncShellTheme.get());
        }
    }
});

// ==========================================
// CUSTOM DIRECTIVE: Right Click Handler
// ==========================================
CCApp.directive('ngRightClick', function($parse) {
    return function(scope, element, attrs) {
        var fn = $parse(attrs.ngRightClick);
        element.bind('contextmenu', function(event) {
            scope.$apply(function() {
                event.preventDefault(); // Prevent standard browser right-click menu
                fn(scope, {$event:event});
            });
        });
    };
});

// ==========================================
// ROOT CONTROLLER (Global Modals & Context Menus)
// ==========================================
CCApp.controller('RootController', function($scope, $document, $rootScope, $timeout) {
    $rootScope.buildRfr = $rootScope.buildRfr || '—';
    $rootScope.buildPrism = $rootScope.buildPrism || '—';
    $rootScope.buildGame = $rootScope.buildGame || '150805';

    function syncBlaze() {
        if (!window.CncBlazeState) {
            $rootScope.playerName = 'UnknownPlayer';
            return;
        }
        $rootScope.blazeEmail = CncBlazeState.email;
        $rootScope.blazeDisplayName = CncBlazeState.displayName;
        $rootScope.blazePersonaId = CncBlazeState.personaId;
        $rootScope.playerName = CncBlazeState.getPlayerName();
    }
    if (window.CncBlazeState) {
        CncBlazeState.subscribe(function () {
            if ($rootScope.$$phase) {
                syncBlaze();
            } else {
                $rootScope.$apply(function () { syncBlaze(); });
            }
        });
        syncBlaze();
    }

    $timeout(function () {
        if (window.CncPreLanding && CncPreLanding.shouldRunPreLanding && !CncPreLanding.shouldRunPreLanding()) {
            $rootScope.preLandingComplete = true;
            if (window.CncBlazeState && CncBlazeState.applyExternalHints) {
                CncBlazeState.applyExternalHints();
            }
            syncBlaze();
            return;
        }
        if (typeof CncPreLanding === 'undefined' || !CncPreLanding.run) {
            $rootScope.$apply(function () {
                $rootScope.preLandingComplete = true;
            });
            return;
        }
        CncPreLanding.run({
            setStatus: function (line) {
                if ($rootScope.$$phase) {
                    $rootScope.preLandingStatus = line;
                } else {
                    $rootScope.$apply(function () {
                        $rootScope.preLandingStatus = line;
                    });
                }
            },
            onDone: function () {
                $rootScope.$apply(function () {
                    $rootScope.preLandingComplete = true;
                    if (window.CncBlazeState && CncBlazeState.applyExternalHints) {
                        CncBlazeState.applyExternalHints();
                    }
                    syncBlaze();
                });
            }
        });
    }, 0);

    if (window.CncBlazeState && CncBlazeState.applyExternalHints) {
        [100, 400, 1200, 3000, 8000, 15000, 30000].forEach(function (ms) {
            $timeout(function () {
                CncBlazeState.applyExternalHints();
                if ($rootScope.$$phase) {
                    syncBlaze();
                } else {
                    $rootScope.$apply(function () {
                        syncBlaze();
                    });
                }
            }, ms);
        });
    }

    $rootScope.openDebug = function () {
        window.location.href = 'debug.html';
    };

    $rootScope.optionsOpen = false;
    $rootScope.openOptions = function () { $rootScope.optionsOpen = true; };
    $rootScope.closeOptions = function () { $rootScope.optionsOpen = false; };

    $rootScope.creditsOpen = false;
    $rootScope.openCredits = function () { $rootScope.creditsOpen = true; };
    $rootScope.closeCredits = function () { $rootScope.creditsOpen = false; };

    // Support links open in the OS browser via retail RtsWebKitClient LinkNotification:
    // an 'external:' navigation is handed to ShellExecuteExW on this PC and cancelled.
    $rootScope.PATREON_URL = 'https://www.patreon.com/cw/refracted_';
    $rootScope.openSupportUs = function () {
        window.location.href = 'external:' + $rootScope.PATREON_URL;
    };

    $rootScope.alertPopupOpen = false;
    $rootScope.toggleAlertPopup = function () {
        $rootScope.alertPopupOpen = !$rootScope.alertPopupOpen;
    };

    $rootScope.confirmQuitOpen = false;
    $rootScope.quittingOpen = false;
    $rootScope.openConfirmQuit = function () {
        $rootScope.confirmQuitOpen = true;
    };
    $rootScope.cancelConfirmQuit = function () {
        $rootScope.confirmQuitOpen = false;
    };
    // Refused Blaze login (cnc-pre-landing -> /cnc/auth-refusal): blocking modal, Quit only.
    $rootScope.authRefusal = null;
    window.CncShowAuthRefusal = function (info) {
        var show = function () {
            $rootScope.authRefusal = info;
        };
        if ($rootScope.$$phase) {
            show();
        } else {
            $rootScope.$apply(show);
        }
    };
    $rootScope.authRefusalQuit = function () {
        if ($rootScope.authRefusal) {
            $rootScope.authRefusal.quitting = true;
        }
        $rootScope.confirmQuitYes();
    };

    $rootScope.confirmQuitYes = function () {
        $rootScope.confirmQuitOpen = false;
        $rootScope.quittingOpen = true;
        if (window.CncShellQuit && CncShellQuit.quitToDesktop) {
            CncShellQuit.quitToDesktop();
            return;
        }
        try {
            if (typeof gameclient !== 'undefined' && gameclient && typeof gameclient.execute === 'function') {
                gameclient.execute('quit true');
            }
        } catch (e) { /* ignore */ }
    };

    $rootScope.serverLostError = '';
    $rootScope.showServerLostModal = function (message) {
        $rootScope.serverLostError = message || 'Server connection lost.';
        try {
            sessionStorage.setItem('cnc_connection_lost', '1');
        } catch (e) { /* ignore */ }
    };
    function localShellPersonaId() {
        var pid = 0;
        try {
            pid = Number(sessionStorage.getItem('cnc_match_pid')) || 0;
        } catch (e) { /* ignore */ }
        if (!pid && window.CncBlazeState && CncBlazeState.getPersonaId) {
            pid = CncBlazeState.getPersonaId() || 0;
        }
        if (!pid && $rootScope.blazePersonaId) {
            pid = Number($rootScope.blazePersonaId) || 0;
        }
        return pid || 0;
    }

    $rootScope.dismissServerLostError = function () {
        $rootScope.serverLostError = '';
        try {
            sessionStorage.removeItem('cnc_connection_lost');
        } catch (e) { /* ignore */ }
        try {
            var xhr = new XMLHttpRequest();
            xhr.open('POST', '/cnc/dismiss-match-lost?pid=' +
                encodeURIComponent(String(localShellPersonaId())), true);
            xhr.send(null);
        } catch (e2) { /* ignore */ }
    };

    function restoreLostModalIfNeeded() {
        var delays = [0, 400, 1500, 4000, 10000];
        var i = 0;
        function tick() {
            if ($rootScope.serverLostError) {
                return;
            }
            var pid = localShellPersonaId();
            try {
                var xhr = new XMLHttpRequest();
                xhr.open('GET', '/cnc/match-connection-status?gid=0&pid=' +
                    encodeURIComponent(String(pid)), true);
                xhr.onreadystatechange = function () {
                    if (xhr.readyState !== 4) {
                        return;
                    }
                    var data = null;
                    try {
                        data = JSON.parse(xhr.responseText);
                    } catch (pe) { /* ignore */ }
                    var apply = function () {
                        if (data && (data.lost || data.serverLost || data.shellLost || data.clientLost)) {
                            $rootScope.showServerLostModal();
                            return;
                        }
                        if (pid > 0) {
                            try {
                                sessionStorage.removeItem('cnc_connection_lost');
                            } catch (ce) { /* ignore */ }
                            return;
                        }
                        i += 1;
                        if (i < delays.length) {
                            $timeout(tick, delays[i]);
                        }
                    };
                    if ($rootScope.$$phase) {
                        apply();
                    } else {
                        $rootScope.$apply(apply);
                    }
                };
                xhr.send(null);
            } catch (e3) { /* ignore */ }
        }
        $timeout(tick, delays[0]);
    }
    restoreLostModalIfNeeded();

    $rootScope.matchRecord = { wins: 0, losses: 0, winStreak: 0, lossStreak: 0 };
    function loadMatchRecord() {
        var key = '';
        try {
            if (window.CncPreLanding && CncPreLanding.getLoginKey) {
                key = CncPreLanding.getLoginKey() || '';
            }
        } catch (e) { /* ignore */ }
        var url = '/cnc/match-record';
        if (key) {
            url += '?key=' + encodeURIComponent(key);
        } else {
            var pid = localShellPersonaId();
            if (pid) {
                url += '?pid=' + encodeURIComponent(String(pid));
            }
        }
        try {
            var xhr = new XMLHttpRequest();
            xhr.open('GET', url, true);
            xhr.onreadystatechange = function () {
                if (xhr.readyState !== 4 || xhr.status !== 200) {
                    return;
                }
                var data = null;
                try {
                    data = JSON.parse(xhr.responseText);
                } catch (pe) { /* ignore */ }
                if (!data || !data.ok) {
                    return;
                }
                var apply = function () {
                    $rootScope.matchRecord = {
                        wins: data.wins || 0,
                        losses: data.losses || 0,
                        winStreak: data.winStreak || 0,
                        lossStreak: data.lossStreak || 0,
                        lastGeneralId: data.lastGeneralId || 0,
                        lastFaction: data.lastFaction || '',
                        shortestMatchSecs: data.shortestMatchSecs || 0,
                        bestWinSecs: data.bestWinSecs || 0
                    };
                };
                if ($rootScope.$$phase) {
                    apply();
                } else {
                    $rootScope.$apply(apply);
                }
            };
            xhr.send(null);
        } catch (e2) { /* ignore */ }
    }
    $rootScope.refreshMatchRecord = loadMatchRecord;
    $timeout(loadMatchRecord, 0);
    $timeout(loadMatchRecord, 2000);

    // Global Context Menu Logic (Mouse Tracking)
    $scope.friendContextMenu = { open: false, x: 0, y: 0, friend: null };

    $scope.openFriendContextMenu = function(friend, event) {
        if (event && event.stopPropagation) event.stopPropagation();
        
        $scope.friendContextMenu.friend = friend;
        
        // Calculate Coordinates
        var x = event.clientX;
        var y = event.clientY;
        
        // Boundary protection so the menu doesn't clip off the screen
        if (x + 160 > window.innerWidth) { x = x - 160; }
        if (y + 120 > window.innerHeight) { y = y - 120; }

        $scope.friendContextMenu.x = x;
        $scope.friendContextMenu.y = y;
        $scope.friendContextMenu.open = true;
    };

    $scope.closeFriendContextMenu = function() {
        $scope.friendContextMenu.open = false;
    };

    // Click outside to close context menu
    $document.bind('click', function(event){
        if ($scope.friendContextMenu.open && !$(event.target).closest('.friend-context-menu').length) {
            $scope.$apply(function(){ $scope.closeFriendContextMenu(); });
        }
    });

    function isTextField(el) {
        if (!el || !el.tagName) {
            return false;
        }
        var tag = el.tagName.toUpperCase();
        if (tag === 'TEXTAREA') {
            return true;
        }
        if (tag !== 'INPUT') {
            return false;
        }
        var type = (el.type || 'text').toLowerCase();
        return type === 'text' || type === 'password' || type === 'search' || type === 'email' || type === '';
    }

    // Esc walks back: dialogs, then the current page, then quit confirm on the main menu.
    function handleShellEscape() {
        if (!$rootScope.preLandingComplete || $rootScope.authRefusal || $rootScope.quittingOpen) {
            return;
        }
        var el = document.activeElement;
        if (isTextField(el)) {
            el.blur();
            return;
        }
        if ($rootScope.confirmQuitOpen) {
            $rootScope.cancelConfirmQuit();
            return;
        }
        if ($rootScope.serverLostError) {
            $rootScope.dismissServerLostError();
            return;
        }
        if ($rootScope.optionsOpen) {
            $rootScope.closeOptions();
            return;
        }
        if ($rootScope.creditsOpen) {
            $rootScope.closeCredits();
            return;
        }
        if ($rootScope.alertPopupOpen) {
            $rootScope.alertPopupOpen = false;
            return;
        }
        var packet = { handled: false };
        $rootScope.$broadcast('cnc:escape', packet);
        if (packet.handled || $rootScope.lobbyOpen) {
            return;
        }
        $rootScope.openConfirmQuit();
    }

    var escHeld = false;
    $document.bind('keydown', function (event) {
        var key = event.keyCode || event.which;
        if (key !== 27) {
            return;
        }
        if (escHeld) {
            return;
        }
        escHeld = true;
        if (event.preventDefault) {
            event.preventDefault();
        }
        if ($scope.$$phase) {
            handleShellEscape();
        } else {
            $scope.$apply(handleShellEscape);
        }
    });
    $document.bind('keyup', function (event) {
        var key = event.keyCode || event.which;
        if (key === 27) {
            escHeld = false;
        }
    });
});

// ==========================================
// DASHBOARD CONTROLLER (Header, Live Tab, Sidebar)
// ==========================================
CCApp.controller('DashboardController', function($scope, $timeout, $rootScope) {
    
    // 1. Core User Info (Header) -- $rootScope.playerName from Blaze + UnknownPlayer fallback
    $scope.wins = 0;
    $scope.losses = 0;
    function applyMatchRecord(rec) {
        if (!rec) {
            return;
        }
        $scope.wins = rec.wins || 0;
        $scope.losses = rec.losses || 0;
    }
    applyMatchRecord($rootScope.matchRecord);
    $rootScope.$watch('matchRecord', applyMatchRecord, true);
    $scope.premiumDays = 90;
    $scope.selectedMode = "PVE";
    
    // 2. Main Navigation Routing
    $scope.activeTab = 'LIVE'; 
    $scope.activeSubTab = '';

    var PLAYTEST_LOCKED_TABS = {
        PROFILE: 1,
        CUSTOMIZE: 1,
        WATCH: 1,
        LEARN: 1,
        SUPPORT: 1
    };

    $scope.playtestNavLocked = function (tabName) {
        return !!($rootScope.playtest && PLAYTEST_LOCKED_TABS[tabName]);
    };

    /** Aurora folds Learn into Support (Play tutorial, FAQ, Guides, Troubleshoot). */
    function auroraShell() {
        return !!(window.CncShellTheme && CncShellTheme.family() === 'aurora');
    }

    $scope.setTab = function(tabName) {
        if (tabName === 'LEARN' && auroraShell()) {
            tabName = 'SUPPORT';
        }
        if ($scope.playtestNavLocked(tabName)) {
            return;
        }
        $scope.activeTab = tabName;
        if ($rootScope.refreshMatchRecord) {
            $rootScope.refreshMatchRecord();
        }
        if (tabName === 'PROFILE') { $scope.activeSubTab = 'OVERVIEW'; } 
        else if (tabName === 'CUSTOMIZE') { $scope.activeSubTab = 'MODIFY GENERALS'; } 
        else if (tabName === 'SUPPORT') { $scope.activeSubTab = auroraShell() ? 'PLAY TUTORIAL' : 'FAQ'; } 
        else if (tabName === 'LEARN') { $scope.activeSubTab = 'GUIDES'; } 
        else { $scope.activeSubTab = ''; }
    };

    $scope.setSubTab = function(subTabName) { $scope.activeSubTab = subTabName; };

    $scope.hasSubnavRow = function () {
        return $scope.activeTab === 'PROFILE' || $scope.activeTab === 'CUSTOMIZE'
            || $scope.activeTab === 'LEARN' || $scope.activeTab === 'SUPPORT';
    };
    $scope.setMode = function(modeName) { $scope.selectedMode = modeName; };
    $scope.openLobby = function () {
        if ($rootScope.openLobby) {
            $rootScope.openLobby();
        }
    };
    $scope.quickMatch = function () {
        if ($rootScope.quickMatch) {
            $rootScope.quickMatch();
        }
    };

    // Aurora: the general on the main menu (js/shell-general-art.js): the one played last
    // (matchRecord.lastGeneralId, from Refracted), else the lobby default from OPTIONS.
    $scope.generalArt = null;
    $scope.generalLine = '';
    $scope.generalIcon = '';
    var generalShown = '';
    function refreshGeneral() {
        var rec = $rootScope.matchRecord || {};
        var pick = (window.CncGeneralArt && CncGeneralArt.current)
            ? CncGeneralArt.current({ generalId: rec.lastGeneralId }) : null;
        $scope.generalLine = pick ? pick.line : '';
        $scope.generalIcon = pick ? pick.icon : '';
        var src = pick ? pick.src : '';
        if (src === generalShown) {
            return;
        }
        generalShown = src;
        $scope.generalArt = src ? { src: src, fit: pick.fit, bust: pick.bust, style: pick.style } : null;
    }
    $rootScope.$watch('matchRecord.lastGeneralId', function () { refreshGeneral(); });
    $scope.winRate = function () {
        var played = ($scope.wins || 0) + ($scope.losses || 0);
        return played ? Math.round(($scope.wins / played) * 100) : 0;
    };
    refreshGeneral();
    // Lobby defaults are edited in OPTIONS; pick the change up when the dialog closes.
    $scope.$on('$destroy', $rootScope.$watch('optionsOpen', function (open, was) {
        if (was && !open) {
            refreshGeneral();
        }
    }));

    // 3. Shared Faction Data (Header Cluster)
    $scope.activeFaction = 'apa'; 
    $scope.playerFactions = [
        { id: 'apa', name: 'APA', level: 19, progress: 85, color: 'red', icon: 'view/image/Factionlogos/APA_sm.png' },
        { id: 'gla', name: 'GLA', level: 1, progress: 20, color: 'green', icon: 'view/image/Factionlogos/GLA_sm.png' },
        { id: 'eu', name: 'EU', level: 3, progress: 40, color: 'blue', icon: 'view/image/Factionlogos/EU_sm.png' }
    ];

    // ==========================================
    // LIVE TAB LOGIC (Slideshow & News)
    // ==========================================
    var playtestHero = "images/aurora-early-access.jpg";
    $scope.liveImages = $rootScope.playtest ? [playtestHero] : [
        "images/cnc_background.png",
        "view/image/franchise_site_bg_c_dark.jpg",
        "view/image/blue_tex1.jpg",
        "view/image/dark_gray_tex.jpg"
    ];
    $scope.currentLiveImageIndex = 0;
    $scope.previousLiveImage = $scope.liveImages[0];
    $scope.currentLiveImage = $scope.liveImages[0];
    $scope.isFading = true;

    var liveTimer;
    function startLiveSlideshow() {
        if(liveTimer) $timeout.cancel(liveTimer);
        liveTimer = $timeout(function() {
            $scope.previousLiveImage = $scope.currentLiveImage;
            $scope.isFading = false;
            $timeout(function() {
                $scope.currentLiveImageIndex = ($scope.currentLiveImageIndex + 1) % $scope.liveImages.length;
                $scope.currentLiveImage = $scope.liveImages[$scope.currentLiveImageIndex];
                $scope.isFading = true; 
            }, 50);
            startLiveSlideshow(); 
        }, 5000);
    }
    
    $scope.setLiveImage = function(index) {
        if (index === $scope.currentLiveImageIndex) return; 
        if (liveTimer) $timeout.cancel(liveTimer); 
        $scope.previousLiveImage = $scope.currentLiveImage; 
        $scope.isFading = false; 
        $timeout(function() {
            $scope.currentLiveImageIndex = index;
            $scope.currentLiveImage = $scope.liveImages[index];
            $scope.isFading = true; 
            if (!$rootScope.playtest) {
                startLiveSlideshow();
            }
        }, 50);
    };

    if (!$rootScope.playtest) {
        startLiveSlideshow();
    }

    // News: the realm's feed from the Aurora site (js/shell-news.js), administered on its
    // News admin page. An open article is re-pointed at its updated copy, or closed if it
    // was withdrawn.
    $scope.newsEntries = [];
    if (window.CncNews) {
        var stopNews = window.CncNews.subscribe(function (articles) {
            var apply = function () {
                $scope.newsEntries = articles;
                if ($scope.activeNewsArticle) {
                    var open = null;
                    for (var n = 0; n < articles.length; n++) {
                        if (articles[n].id === $scope.activeNewsArticle.id) {
                            open = articles[n];
                        }
                    }
                    $scope.activeNewsArticle = open;
                }
            };
            if ($scope.$$phase || ($scope.$root && $scope.$root.$$phase)) {
                apply();
            } else {
                $scope.$apply(apply);
            }
        });
        $scope.$on('$destroy', stopNews);
    }

    $scope.newsRefreshCooling = false;
    $scope.refreshNews = function () {
        if (!window.CncNews || !window.CncNews.refresh()) {
            return;
        }
        $scope.newsRefreshCooling = true;
        window.setTimeout(function () {
            $scope.$apply(function () { $scope.newsRefreshCooling = false; });
        }, window.CncNews.refreshCooldownMs);
    };

    // Aurora (Alternate): the feature banner leads with the newest article.
    var FEATURE_FALLBACK = 'view/image/lobby-card-art/monsoon.png';
    $scope.featureImage = function () {
        var lead = $scope.newsEntries[0];
        if (lead && lead.image) {
            return lead.image;
        }
        return $rootScope.playtest ? playtestHero : FEATURE_FALLBACK;
    };

    $scope.activeNewsArticle = null;
    $scope.openArticle = function(article) {
        if (!article) {
            return;
        }
        $scope.activeNewsArticle = article;
        $scope.mapOpen = false;
    };
    $scope.closeArticle = function() { $scope.activeNewsArticle = null; };

    var SUBTAB_HOME = {
        PROFILE: 'OVERVIEW',
        CUSTOMIZE: 'MODIFY GENERALS',
        SUPPORT: 'FAQ',
        LEARN: 'GUIDES'
    };
    $scope.$on('cnc:escape', function (e, packet) {
        if ($scope.friendContextMenu && $scope.friendContextMenu.open) {
            $scope.closeFriendContextMenu();
            packet.handled = true;
            return;
        }
        if ($scope.mapOpen) {
            $scope.mapOpen = false;
            packet.handled = true;
            return;
        }
        if ($scope.joinedRoom) {
            $scope.leaveRoom();
            packet.handled = true;
            return;
        }
        if ($scope.activeNewsArticle) {
            $scope.closeArticle();
            packet.handled = true;
            return;
        }
        var homeSub = ($scope.activeTab === 'SUPPORT' && auroraShell()) ? 'PLAY TUTORIAL' : SUBTAB_HOME[$scope.activeTab];
        if (homeSub && $scope.activeSubTab && $scope.activeSubTab !== homeSub) {
            $scope.setSubTab(homeSub);
            packet.handled = true;
            return;
        }
        if ($scope.activeTab && $scope.activeTab !== 'LIVE') {
            $scope.setTab('LIVE');
            packet.handled = true;
        }
    });

    // ==========================================
    // SIDEBAR LOGIC (Feeds, Chat, Friends)
    // ==========================================
    $scope.sidebarActiveTab = 'FRIENDS'; 
    $scope.setSidebarTab = function(tabName) {
        if ($rootScope.playtest) {
            return;
        }
        $scope.sidebarActiveTab = tabName;
    };

    // Sidebar: Feeds
    $scope.feedsData = [ 
        { text: "You have recruited", highlight: "General 2", time: "5 minutes ago" },
        { text: "You earned", highlight: "Demolitionist Medal", time: "2 hours ago" }
    ];

    // Sidebar: Friends Groups & Selection
    $scope.friendsGroups = { playing: true, online: true, offline: true };
    $scope.toggleFriendGroup = function(group) { $scope.friendsGroups[group] = !$scope.friendsGroups[group]; };

    $scope.selectedFriendId = null;
    $scope.selectFriend = function(friend) {
        $scope.selectedFriendId = (friend && friend.id) ? friend.id : null;
    };

    var defaultAvatar = 'view/image/guest_avatar.svg';

    $scope.friendsOnline = [
        { id: 'brossea', name: 'Brossea', status: 'In main lobby', avatar: defaultAvatar },
        { id: 'dabrifa', name: 'Dabrifa', status: 'In chat room', avatar: defaultAvatar }
    ];

    $scope.friendsOffline = [
        { id: 'stryker', name: 'CommanderStryker', status: 'Offline', avatar: defaultAvatar },
        { id: 'jake33', name: 'jakedate', status: 'Offline', avatar: defaultAvatar }
    ];

    // Sidebar: Chat & Rooms
    $scope.publicRooms = [ { name: "General Discussion" }, { name: "Beginners" }, { name: "1v1 Matchmaking" } ];
    $scope.joinedRoom = null;
    $scope.chatMessages = [];
    $scope.newChatMessage = "";

    $scope.joinRoom = function(room) {
        $scope.joinedRoom = room; $scope.mapOpen = false; 
        $scope.chatMessages = [ 
            { user: "jakedate", text: "why is this game" },
            { user: "EA_Baelor", text: "Welcome to the chat!" }
        ];
    };
    $scope.leaveRoom = function() { $scope.joinedRoom = null; $scope.chatMessages = []; };
    
    $scope.sendMessage = function() {
        if ($scope.newChatMessage.trim() !== "") {
            $scope.chatMessages.push({ user: $rootScope.playerName || 'UnknownPlayer', text: $scope.newChatMessage });
            $scope.newChatMessage = "";
        }
    };

    // ==========================================
    // MAP LOGIC + LIVE PRESENCE
    // ==========================================
    $scope.onlinePlayers = 0;
    $scope.onlineServers = 0;
    $scope.buildRfr = '—';
    $scope.buildPrism = '—';
    $scope.buildCnc = '150805';
    $scope.buildGame = '150805';
    $rootScope.buildRfr = $rootScope.buildRfr || '—';
    $rootScope.buildPrism = $rootScope.buildPrism || '—';
    $rootScope.buildGame = $rootScope.buildGame || '150805';
    $scope.mapOpen = false; 
    $scope.toggleMap = function() { $scope.mapOpen = !$scope.mapOpen; };

    function publishBuild(name, value) {
        if (!value) {
            return;
        }
        $scope[name] = value;
        $rootScope[name] = value;
    }

    function applyBuildInfo(body) {
        if (!body) {
            return;
        }
        if (body.rfr) {
            publishBuild('buildRfr', body.rfr);
        }
        // Prism version is client-owned
        try {
            if (window.__PRISM_VERSION) {
                publishBuild('buildPrism', window.__PRISM_VERSION);
            } else if (body.prism) {
                publishBuild('buildPrism', body.prism);
            }
        } catch (eP) {
            if (body.prism) {
                publishBuild('buildPrism', body.prism);
            }
        }
        var shellBuild = body.cnc_rl || body.cnc;
        if (shellBuild) {
            $scope.buildCnc = shellBuild;
        }
        // gamebuild is the exe version from GetGameBuild(). Fall back to the shell constant.
        var game = body.gamebuild || shellBuild;
        if (game) {
            publishBuild('buildGame', game);
        }
        if (body.debugDll === true || body.debugDll === 1 || body.debugDll === '1') {
            $rootScope.showDebugTools = true;
        }
    }

    function refreshBuildInfo() {
        try {
            var injected = window.__CNC_BUILD;
            if (injected && typeof injected === 'object') {
                applyBuildInfo(injected);
            }
        } catch (e0) { /* ignore */ }
        try {
            if (window.__PRISM_VERSION) {
                $scope.buildPrism = window.__PRISM_VERSION;
            }
        } catch (e1) { /* ignore */ }
        function applyBody(body) {
            if (!body || body === true || !body.ok) {
                return;
            }
            applyBuildInfo(body);
        }
        // Client Prism rewrites this to /cnc/build-info?prism=<prism.core.dll file version>.
        var buildUrl = '/cnc/build-info';
        try {
            if (window.jQuery && jQuery.ajax) {
                jQuery.ajax({
                    url: buildUrl,
                    type: 'GET',
                    dataType: 'json',
                    timeout: 4000,
                    success: function (body) {
                        if ($scope.$$phase) {
                            applyBody(body);
                        } else {
                            $scope.$apply(function () { applyBody(body); });
                        }
                    }
                });
                return;
            }
        } catch (e) { /* ignore */ }
        try {
            var xhr = new XMLHttpRequest();
            xhr.open('GET', buildUrl, true);
            xhr.onreadystatechange = function () {
                if (xhr.readyState !== 4 || xhr.status < 200 || xhr.status >= 300) {
                    return;
                }
                var body = null;
                try {
                    body = window.JSON ? JSON.parse(xhr.responseText) : null;
                } catch (pe) { return; }
                if ($scope.$$phase) {
                    applyBody(body);
                } else {
                    $scope.$apply(function () { applyBody(body); });
                }
            };
            xhr.send(null);
        } catch (e2) { /* ignore */ }
    }

    refreshBuildInfo();

    function refreshPresenceCounts() {
        function applyBody(body) {
            if (!body || body === true || !body.ok) {
                return;
            }
            var players = body.players != null ? body.players : body.count;
            var servers = body.servers != null ? body.servers : 0;
            if (players == null) {
                return;
            }
            $scope.onlinePlayers = players;
            $scope.onlineServers = servers;
        }
        try {
            if (window.jQuery && jQuery.ajax) {
                jQuery.ajax({
                    url: '/cnc/online-count',
                    type: 'GET',
                    dataType: 'json',
                    timeout: 4000,
                    success: function (body) {
                        if ($scope.$$phase) {
                            applyBody(body);
                        } else {
                            $scope.$apply(function () { applyBody(body); });
                        }
                    }
                });
                return;
            }
        } catch (e) { /* ignore */ }
        try {
            var xhr = new XMLHttpRequest();
            xhr.open('GET', '/cnc/online-count', true);
            xhr.onreadystatechange = function () {
                if (xhr.readyState !== 4 || xhr.status < 200 || xhr.status >= 300) {
                    return;
                }
                var body = null;
                try {
                    body = window.JSON ? JSON.parse(xhr.responseText) : null;
                } catch (pe) { return; }
                if ($scope.$$phase) {
                    applyBody(body);
                } else {
                    $scope.$apply(function () { applyBody(body); });
                }
            };
            xhr.send(null);
        } catch (e2) { /* ignore */ }
    }

    refreshPresenceCounts();
    var presencePoll;
    function schedulePresencePoll() {
        presencePoll = $timeout(function () {
            refreshPresenceCounts();
            schedulePresencePoll();
        }, 5000);
    }
    schedulePresencePoll();
    $scope.$on('$destroy', function () {
        if (presencePoll) {
            $timeout.cancel(presencePoll);
        }
    });
});