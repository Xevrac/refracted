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
    $rootScope.lobbyOpen = false;
    $rootScope.lobbyView = 'matchmake';

    $rootScope.openLobby = function () {
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

    // Patreon — open OS browser via Refracted (/cnc/open-url). In-game modal is a bad fit
    // (login/payments + X-Frame-Options). Fallback to window.open for local shell preview.
    $rootScope.PATREON_URL = 'https://www.patreon.com/cw/refracted_';
    $rootScope.openSupportUs = function () {
        var url = $rootScope.PATREON_URL;
        var opened = false;
        try {
            var xhr = new XMLHttpRequest();
            xhr.open('POST', '/cnc/open-url?url=' + encodeURIComponent(url), true);
            xhr.setRequestHeader('Content-Type', 'application/json');
            xhr.onreadystatechange = function () {
                if (xhr.readyState !== 4) { return; }
                if (xhr.status >= 200 && xhr.status < 300) {
                    opened = true;
                    return;
                }
                if (!opened) {
                    try { window.open(url, '_blank'); } catch (e2) { /* ignore */ }
                }
            };
            xhr.onerror = function () {
                if (!opened) {
                    try { window.open(url, '_blank'); } catch (e3) { /* ignore */ }
                }
            };
            xhr.send(JSON.stringify({ url: url }));
        } catch (e) {
            try { window.open(url, '_blank'); } catch (e4) { /* ignore */ }
        }
    };

    $rootScope.alertPopupOpen = false;
    $rootScope.toggleAlertPopup = function () {
        $rootScope.alertPopupOpen = !$rootScope.alertPopupOpen;
    };

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
});

// ==========================================
// DASHBOARD CONTROLLER (Header, Live Tab, Sidebar)
// ==========================================
CCApp.controller('DashboardController', function($scope, $timeout, $rootScope) {
    
    // 1. Core User Info (Header) -- $rootScope.playerName from Blaze + UnknownPlayer fallback
    $scope.wins = 2;
    $scope.losses = 1;
    $scope.premiumDays = 90;
    $scope.selectedMode = "PVE";
    
    // 2. Main Navigation Routing
    $scope.activeTab = 'LIVE'; 
    $scope.activeSubTab = '';

    $scope.setTab = function(tabName) {
        $scope.activeTab = tabName;
        if (tabName === 'PROFILE') { $scope.activeSubTab = 'OVERVIEW'; } 
        else if (tabName === 'CUSTOMIZE') { $scope.activeSubTab = 'MODIFY GENERALS'; } 
        else if (tabName === 'SUPPORT') { $scope.activeSubTab = 'FAQ'; } 
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
    $scope.liveImages = [
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
            startLiveSlideshow(); 
        }, 50);
    };

    startLiveSlideshow();

    // RESTORED: News Entries for Live Tab Sidebar
    $scope.newsEntries = [
        { id: 1, title: "July Patch is awaiting you!", hasImage: true, date: "Tue Oct 15 2013", content: "Good news, we have just updated the game with a small mid-month patch!" },
        { id: 2, title: "Patch 2.0 is live", hasImage: false, date: "Mon Sep 02 2013", content: "Patch 2.0 brings major overhauls to the unit pathing AI..." }
    ];
    
    $scope.activeNewsArticle = null;
    $scope.openArticle = function(article) { $scope.activeNewsArticle = article; $scope.mapOpen = false; };
    $scope.closeArticle = function() { $scope.activeNewsArticle = null; };

    // ==========================================
    // SIDEBAR LOGIC (Feeds, Chat, Friends)
    // ==========================================
    $scope.sidebarActiveTab = 'FRIENDS'; 
    $scope.setSidebarTab = function(tabName) { $scope.sidebarActiveTab = tabName; };

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
    $scope.mapOpen = false; 
    $scope.toggleMap = function() { $scope.mapOpen = !$scope.mapOpen; };

    function applyBuildInfo(body) {
        if (!body) {
            return;
        }
        if (body.rfr) {
            $scope.buildRfr = body.rfr;
        }
        if (body.prism) {
            $scope.buildPrism = body.prism;
        }
        var cnc = body.cnc_rl || body.cnc;
        if (cnc) {
            $scope.buildCnc = cnc;
        }
    }

    function refreshBuildInfo() {
        try {
            var injected = window.__CNC_BUILD;
            if (injected && typeof injected === 'object') {
                applyBuildInfo(injected);
            }
        } catch (e0) { /* ignore */ }
        function applyBody(body) {
            if (!body || body === true || !body.ok) {
                return;
            }
            applyBuildInfo(body);
        }
        try {
            if (window.jQuery && jQuery.ajax) {
                jQuery.ajax({
                    url: '/cnc/build-info',
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
            xhr.open('GET', '/cnc/build-info', true);
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