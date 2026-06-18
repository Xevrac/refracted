/// Blaze Component and Command definitions
/// This module provides component IDs, command names, and lookup functions

use std::collections::HashMap;

/// Component information
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub id: u16,
    pub name: &'static str,
    pub commands: HashMap<u16, &'static str>,
}

/// Get component name by ID
pub fn get_component_name(component_id: u16) -> &'static str {
    match component_id {
        1 => "AuthenticationComponent", // 0x1
        3 => "ExampleComponent", // 0x3
        4 => "GameManager", // 0x4
        5 => "RedirectorComponent", // 0x5
        6 => "PlayGroupsComponent", // 0x6
        7 => "StatsComponent", // 0x7
        9 => "UtilComponent", // 0x9
        10 => "CensusDataComponent", // 0xA
        11 => "ClubsComponent", // 0xB
        12 => "GameReportLegacyComponent", // 0xC
        13 => "LeagueComponent", // 0xD
        14 => "MailComponent", // 0xE
        15 => "MessagingComponent", // 0xF
        20 => "LockerComponent", // 0x14
        21 => "RoomsComponent", // 0x15
        23 => "TournamentsComponent", // 0x17
        24 => "CommerceInfoComponent", // 0x18
        25 => "AssociationListsComponent", // 0x19
        27 => "GpsContentControllerComponent", // 0x1B
        28 => "GameReportingComponent", // 0x1C
        2000 => "DynamicFilterComponent", // 0x7D0
        2049 => "RspComponent", // 0x801
        2050 => "PacksComponent", // 0x802
        2051 => "InventoryComponent", // 0x803
        30722 => "UserSessions", // 0x7802
        69 => "DynamicMessagingComponent", // core/69
        70 => "WebofferSurveyComponent", // core/70
        71 => "TickerComponent", // core/71
        1002 => "NucleusIdentityComponent", // core/1002
        _ => "UnknownComponent",
    }
}

/// Get command name for a component and command ID
pub fn get_command_name(component_id: u16, command_id: u16) -> Option<String> {
    let component_name = get_component_name(component_id);
    
    match (component_id, command_id) {
        // UtilComponent (9)
        (9, 1) => Some(format!("{}.fetchClientConfig", component_name)),
        (9, 2) => Some(format!("{}.ping", component_name)),
        (9, 3) => Some(format!("{}.setClientData", component_name)),
        (9, 4) => Some(format!("{}.localizeStrings", component_name)),
        (9, 5) => Some(format!("{}.getTelemetryServer", component_name)),
        (9, 6) => Some(format!("{}.getTickerServer", component_name)),
        (9, 7) => Some(format!("{}.preAuth", component_name)),
        (9, 8) => Some(format!("{}.postAuth", component_name)),
        (9, 10) => Some(format!("{}.userSettingsLoad", component_name)),
        (9, 11) => Some(format!("{}.userSettingsSave", component_name)),
        (9, 12) => Some(format!("{}.userSettingsLoadAll", component_name)),
        (9, 14) => Some(format!("{}.deleteUserSettings", component_name)),
        (9, 20) => Some(format!("{}.filterForProfanity", component_name)),
        (9, 21) => Some(format!("{}.fetchQosConfig", component_name)),
        (9, 22) => Some(format!("{}.setClientMetrics", component_name)),
        (9, 23) => Some(format!("{}.setConnectionState", component_name)),
        (9, 24) => Some(format!("{}.getPssConfig", component_name)),
        (9, 25) => Some(format!("{}.getUserOptions", component_name)),
        (9, 26) => Some(format!("{}.setUserOptions", component_name)),
        (9, 27) => Some(format!("{}.suspendUserPing", component_name)),
        
        // AuthenticationComponent (1)
        (1, 10) => Some(format!("{}.createAccount", component_name)),
        (1, 20) => Some(format!("{}.updateAccount", component_name)),
        (1, 28) => Some(format!("{}.updateParentalEmail", component_name)),
        (1, 29) => Some(format!("{}.listUserEntitlements2", component_name)),
        (1, 30) => Some(format!("{}.getAccount", component_name)),
        (1, 31) => Some(format!("{}.grantEntitlement", component_name)),
        (1, 32) => Some(format!("{}.listEntitlements", component_name)),
        (1, 33) => Some(format!("{}.hasEntitlement", component_name)),
        (1, 34) => Some(format!("{}.getUseCount", component_name)),
        (1, 35) => Some(format!("{}.decrementUseCount", component_name)),
        (1, 36) => Some(format!("{}.getAuthToken", component_name)),
        (1, 37) => Some(format!("{}.getHandoffToken", component_name)),
        (1, 38) => Some(format!("{}.getPasswordRules", component_name)),
        (1, 39) => Some(format!("{}.grantEntitlement2", component_name)),
        (1, 40) => Some(format!("{}.login", component_name)),
        (1, 41) => Some(format!("{}.acceptTos", component_name)),
        (1, 42) => Some(format!("{}.getTosInfo", component_name)),
        (1, 43) => Some(format!("{}.modifyEntitlement2", component_name)),
        (1, 44) => Some(format!("{}.consumecode", component_name)),
        (1, 45) => Some(format!("{}.passwordForgot", component_name)),
        (1, 46) => Some(format!("{}.getTermsAndConditionsContent", component_name)),
        (1, 47) => Some(format!("{}.getPrivacyPolicyContent", component_name)),
        (1, 48) => Some(format!("{}.listPersonaEntitlements2", component_name)),
        (1, 50) => Some(format!("{}.silentLogin", component_name)),
        (1, 51) => Some(format!("{}.checkAgeReq", component_name)),
        (1, 52) => Some(format!("{}.getOptIn", component_name)),
        (1, 53) => Some(format!("{}.enableOptIn", component_name)),
        (1, 54) => Some(format!("{}.disableOptIn", component_name)),
        (1, 60) => Some(format!("{}.expressLogin", component_name)), // 0x3C
        (1, 70) => Some(format!("{}.logout", component_name)), // 0x46
        (1, 80) => Some(format!("{}.createPersona", component_name)),
        (1, 90) => Some(format!("{}.getPersona", component_name)),
        (1, 100) => Some(format!("{}.listPersonas", component_name)),
        (1, 110) => Some(format!("{}.loginPersona", component_name)),
        (1, 120) => Some(format!("{}.logoutPersona", component_name)),
        (1, 140) => Some(format!("{}.deletePersona", component_name)),
        (1, 141) => Some(format!("{}.disablePersona", component_name)),
        (1, 143) => Some(format!("{}.listDeviceAccounts", component_name)),
        (1, 150) => Some(format!("{}.xboxCreateAccount", component_name)),
        (1, 152) => Some(format!("{}.originLogin", component_name)),
        (1, 160) => Some(format!("{}.xboxAssociateAccount", component_name)),
        (1, 170) => Some(format!("{}.xboxLogin", component_name)),
        (1, 180) => Some(format!("{}.ps3CreateAccount", component_name)),
        (1, 190) => Some(format!("{}.ps3AssociateAccount", component_name)),
        (1, 200) => Some(format!("{}.ps3Login", component_name)),
        (1, 210) => Some(format!("{}.validateSessionKey", component_name)),
        (1, 230) => Some(format!("{}.createWalUserSession", component_name)),
        (1, 241) => Some(format!("{}.acceptLegalDocs", component_name)),
        (1, 242) => Some(format!("{}.getLegalDocsInfo", component_name)),
        (1, 246) => Some(format!("{}.getTermsOfServiceContent", component_name)),
        (1, 300) => Some(format!("{}.deviceLoginGuest", component_name)),
        (1, 500) => Some(format!("{}.checkSinglePlayerLogin", component_name)),
        
        // GameManager (4)
        (4, 1) => Some(format!("{}.createGame", component_name)),
        (4, 2) => Some(format!("{}.destroyGame", component_name)),
        (4, 3) => Some(format!("{}.advanceGameState", component_name)),
        (4, 4) => Some(format!("{}.setGameSettings", component_name)),
        (4, 5) => Some(format!("{}.setPlayerCapacity", component_name)),
        (4, 6) => Some(format!("{}.setPresenceMode", component_name)),
        (4, 7) => Some(format!("{}.setGameAttributes", component_name)),
        (4, 8) => Some(format!("{}.setPlayerAttributes", component_name)),
        (4, 9) => Some(format!("{}.joinGame", component_name)),
        (4, 10) => Some(format!("{}.GMA", component_name)),
        // EA stock: startMatchmaking. CNC 3.19.4 GameManager uses the same id for removePlayer.
        (4, 11) => Some(format!("{}.removePlayer", component_name)),
        (4, 12) => Some(format!("{}.cancelMatchmaking", component_name)),
        (4, 13) => Some(format!("{}.finalizeGameCreation", component_name)),
        (4, 14) => Some(format!("{}.listGames", component_name)),
        (4, 15) => Some(format!("{}.setPlayerCustomData", component_name)),
        (4, 16) => Some(format!("{}.createGameTemplate", component_name)),
        // EA stock id 17; CNC `returnDedicatedServerToPool` is RPC id **20** (0x14).
        (4, 17) => Some(format!("{}.returnDedicatedServerToPool", component_name)),
        (4, 18) => Some(format!("{}.leaveGame", component_name)),
        (4, 19) => Some(format!("{}.selectHost", component_name)),
        (4, 20) => Some(format!(
            "{}.{}",
            component_name,
            if crate::common::game::get_current_game_id().as_str() == "cnc" {
                "returnDedicatedServerToPool/NotifyGameSetup"
            } else {
                "migrateHost"
            }
        )),
        (4, 21) => Some(format!(
            "{}.{}",
            component_name,
            if crate::common::game::get_current_game_id().as_str() == "cnc" {
                "NotifyPlayerJoining"
            } else {
                "updateGameHostMigrationStatus"
            }
        )),
        (4, 22) => Some(format!("{}.resetDedicatedServer", component_name)),
        (4, 23) => Some(format!(
            "{}.{}",
            component_name,
            if crate::common::game::get_current_game_id().as_str() == "cnc" {
                "NotifyPlayerJoiningQueue"
            } else {
                "updateGameSession"
            }
        )),
        (4, 24) => Some(format!("{}.banPlayer", component_name)),
        // CNC Blaze 3.19.x uses command 25 for `resetDedicatedServer` (CreateGameRequest / JoinGameResponse); EA uses 22.
        (4, 25) => Some(format!(
            "{}.{}",
            component_name,
            if crate::common::game::get_current_game_id().as_str() == "cnc" {
                "resetDedicatedServer"
            } else {
                "unbanPlayer"
            }
        )),
        // C&C: async NotifyPlatformHostInitialized (decimal 71 = 0x47). Not in stock EA table at this id.
        (4, 71) => Some(format!(
            "{}.{}",
            component_name,
            if crate::common::game::get_current_game_id().as_str() == "cnc" {
                "NotifyPlatformHostInitialized"
            } else {
                "command71"
            }
        )),
        (4, 26) => Some(format!("{}.updateMeshConnection", component_name)),
        (4, 27) => Some(format!("{}.removePlayerFromBannedList", component_name)),
        (4, 28) => Some(format!("{}.clearBannedList", component_name)),
        (4, 29) => Some(format!("{}.getBannedPlayers", component_name)),
        // CNC notify id 30 = NotifyPlayerJoinCompleted; RPC addQueuedPlayerToGame is id 38 (0x26).
        (4, 30) => Some(format!("{}.NotifyPlayerJoinCompleted", component_name)),
        (4, 31) => Some(format!("{}.updateGameName", component_name)),
        (4, 32) => Some(format!("{}.ejectHost", component_name)),
        (4, 33) => Some(format!("{}.updateGameHostMigrationStart", component_name)),
        (4, 34) => Some(format!("{}.listGameData", component_name)),
        (4, 35) => Some(format!("{}.getGameDataFromId", component_name)),
        (4, 36) => Some(format!("{}.getGameDataFromIdList", component_name)),
        (4, 37) => Some(format!("{}.writeGameData", component_name)),
        (4, 38) => Some(format!("{}.addQueuedPlayerToGame", component_name)),
        (4, 39) => Some(format!("{}.lockGameForJoining", component_name)),
        (4, 40) => Some(format!("{}.unlockGameForJoining", component_name)),
        (4, 41) => Some(format!(
            "{}.{}",
            component_name,
            if crate::common::game::get_current_game_id().as_str() == "cnc" {
                "meshEndpointsConnected"
            } else {
                "setGameModRegister"
            }
        )), // RPC id 65 / 0x41
        (4, 90) => Some(format!(
            "{}.{}",
            component_name,
            if crate::common::game::get_current_game_id().as_str() == "cnc" {
                "NotifyPlayerAttribChange"
            } else {
                "notifyPlayerAttribChange"
            }
        )),
        (4, 42) => Some(format!("{}.getGameListSubscription", component_name)),
        (4, 100) => Some(format!("{}.getGameListSnapshot", component_name)), // 0x64
        (4, 201) => Some(format!(
            "{}.{}",
            component_name,
            if crate::common::game::get_current_game_id().as_str() == "cnc" {
                "NotifyGameListUpdate"
            } else {
                "notifyGameListUpdate"
            }
        )),
        (4, 43) => Some(format!("{}.destroyGameList", component_name)),
        (4, 44) => Some(format!("{}.getFullGameData", component_name)),
        (4, 45) => Some(format!("{}.getMatchmakingConfig", component_name)),
        (4, 46) => Some(format!("{}.getGameDataFromIdListByUser", component_name)),
        (4, 47) => Some(format!("{}.adminListGames", component_name)),
        (4, 48) => Some(format!("{}.getMatchmakingSessionStatus", component_name)),
        (4, 49) => Some(format!("{}.getExternalSessionData", component_name)),
        (4, 50) => Some(format!("{}.updateExternalSessionPresence", component_name)),
        (4, 51) => Some(format!("{}.lookupGameDataByString", component_name)),
        (4, 52) => Some(format!("{}.getGameList", component_name)),
        (4, 53) => Some(format!("{}.getGameDataFromIdListByUser", component_name)),
        (4, 54) => Some(format!("{}.getGameDataFromIdListByUser", component_name)),
        (4, 55) => Some(format!("{}.getGameDataFromIdListByUser", component_name)),
        (4, 56) => Some(format!("{}.getGameDataFromIdListByUser", component_name)),
        (4, 57) => Some(format!("{}.getGameDataFromIdListByUser", component_name)),
        (4, 58) => Some(format!("{}.getGameDataFromIdListByUser", component_name)),
        (4, 59) => Some(format!("{}.getGameDataFromIdListByUser", component_name)),
        (4, 60) => Some(format!("{}.getGameDataFromIdListByUser", component_name)),
        (4, 103) => Some(format!("{}.getFullGameData", component_name)), // 0x67 - alternate ID
        (4, 108) => Some(format!("{}.setPlayerTeam", component_name)), // 0x6C
        (4, 109) => Some(format!("{}.changeGameTeamId", component_name)), // 0x6D
        (4, 110) => Some(format!("{}.migrateAdminPlayer", component_name)), // 0x6E
        (4, 111) => Some(format!("{}.getUserSetGameListSubscription", component_name)), // 0x6F
        (4, 112) => Some(format!("{}.swapPlayersTeam", component_name)), // 0x70
        (4, 113) => Some(format!("{}.getGameDataByUser", component_name)),
        (4, 150) => Some(format!("{}.registerDynamicDedicatedServerCreator", component_name)), // 0x96
        (4, 151) => Some(format!("{}.unregisterDynamicDedicatedServerCreator", component_name)), // 0x97
        (4, 220) => Some(format!(
            "{}.{}",
            component_name,
            if crate::common::game::get_current_game_id().as_str() == "cnc" {
                "NotifyCreateDynamicDedicatedServerGame"
            } else {
                "notifyCreateDynamicDedicatedServerGame"
            }
        )),
        // Note: (4, 19) is selectHost - replayGame also exists as command 16 (0x10) and 19 (0x13) in some implementations
        
        // StatsComponent (7)
        (7, 1) => Some(format!("{}.getStatDescs", component_name)),
        (7, 2) => Some(format!("{}.getStats", component_name)),
        (7, 3) => Some(format!("{}.getStatGroupList", component_name)),
        (7, 4) => Some(format!("{}.getStatGroup", component_name)),
        (7, 5) => Some(format!("{}.getStatsByGroup", component_name)),
        (7, 6) => Some(format!("{}.getDateRange", component_name)),
        (7, 7) => Some(format!("{}.getEntityCount", component_name)),
        (7, 8) => Some(format!("{}.updateStats", component_name)),
        (7, 9) => Some(format!("{}.wipeStats", component_name)),
        (7, 10) => Some(format!("{}.getLeaderboardGroup", component_name)),
        (7, 11) => Some(format!("{}.getLeaderboardFolderGroup", component_name)),
        (7, 12) => Some(format!("{}.getLeaderboard", component_name)),
        (7, 13) => Some(format!("{}.getCenteredLeaderboard", component_name)),
        (7, 14) => Some(format!("{}.getFilteredLeaderboard", component_name)),
        (7, 15) => Some(format!("{}.getKeyScopesMap", component_name)),
        (7, 16) => Some(format!("{}.getStatsByGroupAsync", component_name)),
        (7, 17) => Some(format!("{}.getLeaderboardTreeAsync", component_name)),
        (7, 18) => Some(format!("{}.getLeaderboardEntityCount", component_name)),
        (7, 19) => Some(format!("{}.getStatCategoryList", component_name)),
        (7, 20) => Some(format!("{}.getPeriodIds", component_name)),
        (7, 21) => Some(format!("{}.getLeaderboardRaw", component_name)),
        (7, 22) => Some(format!("{}.getCenteredLeaderboardRaw", component_name)),
        (7, 23) => Some(format!("{}.getFilteredLeaderboardRaw", component_name)),
        (7, 24) => Some(format!("{}.changeKeyscopeValue", component_name)),
        (7, 25) => Some(format!("{}.getEntityRank", component_name)),
        
        // ClubsComponent (11)
        (11, 1100) => Some(format!("{}.createClub", component_name)),
        (11, 1200) => Some(format!("{}.getClubs", component_name)),
        (11, 1300) => Some(format!("{}.findClubs", component_name)),
        (11, 1310) => Some(format!("{}.findClubs2", component_name)),
        (11, 1400) => Some(format!("{}.removeMember", component_name)),
        (11, 1500) => Some(format!("{}.sendInvitation", component_name)),
        (11, 1600) => Some(format!("{}.getInvitations", component_name)),
        (11, 1700) => Some(format!("{}.revokeInvitation", component_name)),
        (11, 1800) => Some(format!("{}.acceptInvitation", component_name)),
        (11, 1900) => Some(format!("{}.declineInvitation", component_name)),
        (11, 2000) => Some(format!("{}.getMembers", component_name)),
        (11, 2100) => Some(format!("{}.promoteToGM", component_name)),
        (11, 2150) => Some(format!("{}.demoteToMember", component_name)),
        (11, 2200) => Some(format!("{}.updateClubSettings", component_name)),
        (11, 2300) => Some(format!("{}.postNews", component_name)),
        (11, 2400) => Some(format!("{}.getNews", component_name)),
        (11, 2450) => Some(format!("{}.setNewsItemHidden", component_name)),
        (11, 2500) => Some(format!("{}.setMetadata", component_name)),
        (11, 2510) => Some(format!("{}.setMetadata2", component_name)),
        (11, 2600) => Some(format!("{}.getClubsComponentSettings", component_name)),
        (11, 2650) => Some(format!("{}.transferOwnership", component_name)),
        (11, 2700) => Some(format!("{}.getClubMembershipForUsers", component_name)),
        (11, 2800) => Some(format!("{}.sendPetition", component_name)),
        (11, 2900) => Some(format!("{}.getPetitions", component_name)),
        (11, 3000) => Some(format!("{}.acceptPetition", component_name)),
        (11, 3100) => Some(format!("{}.declinePetition", component_name)),
        (11, 3200) => Some(format!("{}.revokePetition", component_name)),
        (11, 3300) => Some(format!("{}.joinClub", component_name)),
        (11, 3310) => Some(format!("{}.joinOrPetitionClub", component_name)),
        (11, 3400) => Some(format!("{}.getClubRecordbook", component_name)),
        (11, 3410) => Some(format!("{}.resetClubRecords", component_name)),
        (11, 3500) => Some(format!("{}.updateMemberOnlineStatus", component_name)),
        (11, 3600) => Some(format!("{}.getClubAwards", component_name)),
        (11, 3700) => Some(format!("{}.updateMemberMetadata", component_name)),
        (11, 3800) => Some(format!("{}.findClubsAsync", component_name)),
        (11, 3810) => Some(format!("{}.findClubs2Async", component_name)),
        (11, 3900) => Some(format!("{}.listRivals", component_name)),
        (11, 4000) => Some(format!("{}.getClubTickerMessages", component_name)),
        (11, 4100) => Some(format!("{}.setClubTickerMessagesSubscription", component_name)),
        (11, 4200) => Some(format!("{}.changeClubStrings", component_name)),
        (11, 4300) => Some(format!("{}.countMessages", component_name)),
        (11, 4400) => Some(format!("{}.getMembersAsync", component_name)),
        (11, 4500) => Some(format!("{}.getClubBans", component_name)),
        (11, 4600) => Some(format!("{}.getUserBans", component_name)),
        (11, 4700) => Some(format!("{}.banMember", component_name)),
        (11, 4800) => Some(format!("{}.unbanMember", component_name)),
        (11, 4900) => Some(format!("{}.GetClubsComponentInfo", component_name)),
        (11, 5000) => Some(format!("{}.disbandClub", component_name)),
        (11, 5100) => Some(format!("{}.getNewsForClubs", component_name)),
        (11, 5200) => Some(format!("{}.getPetitionsForClubs", component_name)),
        (11, 5300) => Some(format!("{}.getClubTickerMessagesForClubs", component_name)),
        (11, 5400) => Some(format!("{}.countMessagesForClubs", component_name)),
        (11, 5500) => Some(format!("{}.getMemberOnlineStatus", component_name)),
        (11, 5600) => Some(format!("{}.getMemberStatusInClub", component_name)),
        (11, 5700) => Some(format!("{}.logEvent", component_name)),
        (11, 5800) => Some(format!("{}.wipeStats", component_name)),
        
        // GameReportLegacyComponent (12 / 0xC)
        (12, 1) => Some(format!("{}.submitGameReport", component_name)),
        (12, 2) => Some(format!("{}.submitOfflineGameReport", component_name)),
        (12, 3) => Some(format!("{}.submitGameEvents", component_name)),
        (12, 4) => Some(format!("{}.getGameReports", component_name)),
        (12, 5) => Some(format!("{}.getGameReportView", component_name)),
        (12, 6) => Some(format!("{}.getGameReportViewInfo", component_name)),
        (12, 7) => Some(format!("{}.getGameReportViewInfoList", component_name)),
        (12, 8) => Some(format!("{}.getGameReportTypes", component_name)),
        (12, 100) => Some(format!("{}.submitTrustedMidGameReport", component_name)),
        (12, 101) => Some(format!("{}.submitTrustedEndGameReport", component_name)),
        
        // MessagingComponent (15)
        (15, 1) => Some(format!("{}.sendMessage", component_name)),
        (15, 2) => Some(format!("{}.fetchMessages", component_name)),
        (15, 3) => Some(format!("{}.purgeMessages", component_name)),
        (15, 4) => Some(format!("{}.touchMessages", component_name)),
        (15, 5) => Some(format!("{}.getMessages", component_name)),
        (15, 6) => Some(format!("{}.sendSourceMessage", component_name)),
        (15, 7) => Some(format!("{}.sendGlobalMessage", component_name)),
        
        // TournamentsComponent (23 / 0x17)
        (23, 1) => Some(format!("{}.getTournaments", component_name)),
        (23, 2) => Some(format!("{}.getAllTournaments", component_name)),
        (23, 3) => Some(format!("{}.getMemberCounts", component_name)),
        (23, 4) => Some(format!("{}.getTrophies", component_name)),
        (23, 5) => Some(format!("{}.getMyTournamentId", component_name)),
        (23, 6) => Some(format!("{}.joinTournament", component_name)),
        (23, 7) => Some(format!("{}.leaveTournament", component_name)),
        (23, 8) => Some(format!("{}.resetTournament", component_name)),
        (23, 9) => Some(format!("{}.getMyTournamentDetails", component_name)),
        (23, 10) => Some(format!("{}.resetAllTournamentMembers", component_name)),
        
        // UserSessions (30722 / 0x7802)
        (30722, 1) => Some(format!("{}.UserSessionExtendedDataUpdate", component_name)), // Notification
        (30722, 2) => Some(format!("{}.UserAdded", component_name)), // Notification
        (30722, 3) => Some(format!("{}.fetchExtendedData", component_name)),
        (30722, 4) => Some(format!("{}.UserRemoved", component_name)), // Notification
        (30722, 5) => Some(format!("{}.UserUpdated", component_name)), // Notification (also updateExtendedDataAttribute)
        // Cmd 8: client MESSAGE = updateHardwareFlags; server NOTIFICATION after login = UserAuthenticated (same command id).
        (30722, 8) => Some(format!("{}.updateHardwareFlags", component_name)),
        (30722, 12) => Some(format!("{}.lookupUser", component_name)),
        (30722, 13) => Some(format!("{}.lookupUsers", component_name)),
        (30722, 14) => Some(format!("{}.lookupUsersByPrefix", component_name)),
        (30722, 20) => Some(format!("{}.updateNetworkInfo", component_name)),
        (30722, 23) => Some(format!("{}.lookupUserGeoIPData", component_name)),
        (30722, 24) => Some(format!("{}.overrideUserGeoIPData", component_name)),
        (30722, 25) => Some(format!("{}.updateUserSessionClientData", component_name)),
        (30722, 26) => Some(format!("{}.setUserInfoAttribute", component_name)),
        (30722, 27) => Some(format!("{}.resetUserGeoIPData", component_name)),
        (30722, 32) => Some(format!("{}.lookupUserSessionId", component_name)),
        (30722, 33) => Some(format!("{}.fetchLastLocaleUsedAndAuthError", component_name)),
        (30722, 34) => Some(format!("{}.fetchUserFirstLastAuthTime", component_name)),
        (30722, 35) => Some(format!("{}.resumeSession", component_name)),
        (30722, 60) => Some(format!("{}.setClientState", component_name)), // Command 60
        
        // RedirectorComponent (5)
        (5, 1) => Some(format!("{}.getServerInstance", component_name)),
        
        // AssociationListsComponent (25 / 0x19)
        (25, 1) => Some(format!("{}.addUsersToList", component_name)),
        (25, 2) => Some(format!("{}.removeUsersFromList", component_name)),
        (25, 3) => Some(format!("{}.clearLists", component_name)),
        (25, 4) => Some(format!("{}.setUsersToList", component_name)),
        (25, 5) => Some(format!("{}.getListForUser", component_name)),
        (25, 6) => Some(format!("{}.getLists", component_name)),
        (25, 7) => Some(format!("{}.subscribeToLists", component_name)),
        (25, 8) => Some(format!("{}.unsubscribeFromLists", component_name)),
        (25, 9) => Some(format!("{}.getConfigListsInfo", component_name)),
        
        // GameReportingComponent (28 / 0x1C)
        (28, 1) => Some(format!("{}.submitGameReport", component_name)),
        (28, 2) => Some(format!("{}.submitOfflineGameReport", component_name)),
        (28, 3) => Some(format!("{}.submitGameEvents", component_name)),
        (28, 4) => Some(format!("{}.getGameReportQuery", component_name)),
        (28, 5) => Some(format!("{}.getGameReportQueriesList", component_name)),
        (28, 6) => Some(format!("{}.getGameReports", component_name)),
        (28, 7) => Some(format!("{}.getGameReportView", component_name)),
        (28, 8) => Some(format!("{}.getGameReportViewInfo", component_name)),
        (28, 9) => Some(format!("{}.getGameReportViewInfoList", component_name)),
        (28, 10) => Some(format!("{}.getGameReportTypes", component_name)),
        (28, 11) => Some(format!("{}.updateMetric", component_name)),
        (28, 12) => Some(format!("{}.getGameReportColumnInfo", component_name)),
        (28, 13) => Some(format!("{}.getGameReportColumnValues", component_name)),
        (28, 100) => Some(format!("{}.submitTrustedMidGameReport", component_name)),
        (28, 101) => Some(format!("{}.submitTrustedEndGameReport", component_name)),
        
        // RspComponent (2049 / 0x801)
        (2049, 1) => Some(format!("{}.getRsp", component_name)),
        (2049, 50) => Some(format!("{}.getConfig", component_name)), // 0x32 - alternate command name
        
        // InventoryComponent (2051 / 0x803)
        (2051, 1) => Some(format!("{}.getItems", component_name)), // 0x01
        (2051, 5) => Some(format!("{}.drainConsumeable", component_name)),
        (2051, 6) => Some(format!("{}.getTemplate", component_name)), // 0x06
        
        // NucleusIdentityComponent (1002) — core/1002
        (1002, 1) => Some(format!("{}.updateEntitlement", component_name)),
        (1002, 2) => Some(format!("{}.updatePersona", component_name)),
        (1002, 3) => Some(format!("{}.deletePersona", component_name)),
        (1002, 4) => Some(format!("{}.getOptin", component_name)),
        (1002, 5) => Some(format!("{}.postOptin", component_name)),
        (1002, 6) => Some(format!("{}.deleteOptin", component_name)),
        (1002, 7) => Some(format!("{}.getAccount", component_name)),
        (1002, 8) => Some(format!("{}.getAccountEntitlements", component_name)),
        (1002, 9) => Some(format!("{}.getPersonaEntitlements", component_name)),
        (1002, 10) => Some(format!("{}.getEntitlement", component_name)),
        (1002, 11) => Some(format!("{}.postEntitlement", component_name)),
        (1002, 12) => Some(format!("{}.searchOrPostEntitlement", component_name)),
        (1002, 13) => Some(format!("{}.postEntitlementPersonaLink", component_name)),
        (1002, 14) => Some(format!("{}.postProfileInfo", component_name)),
        (1002, 15) => Some(format!("{}.getProfileInfo", component_name)),
        (1002, 16) => Some(format!("{}.getPid", component_name)),
        (1002, 17) => Some(format!("{}.getPersonaList", component_name)),
        (1002, 19) => Some(format!("{}.getPersonaInfo", component_name)),
        
        _ => None,
    }
}

/// Check if a component/command combination is handled
pub fn is_command_handled(component_id: u16, command_id: u16) -> bool {
    match (component_id, command_id) {
        // UtilComponent - handled commands
        (9, 1) | (9, 2) | (9, 5) | (9, 7) | (9, 8) | (9, 9) | (9, 22) | (9, 28) => true,
        
        // AuthenticationComponent - handled commands
        (1, 10) | (1, 70) => true, // createAccount/login (0x0a), logout (0x46)
        
        // MessagingComponent (15) - handled commands
        (15, 1) => true, // sendMessage
        
        // UserSessions (0x7802) - handled commands
        (30722, 1) | (30722, 8) | (30722, 11) | (30722, 12) | (30722, 20) | (30722, 21) | (30722, 60) => true,
        
        // GameManager - handled commands
        (4, 3) | (4, 5) | (4, 7) | (4, 10) | (4, 16) | (4, 17) | (4, 100) | (4, 113) => true,
        
        // StatsComponent - handled commands
        (7, 0) | (7, 3840) | (7, 10496) | (7, 14080) | (7, 16640) | (7, 20224) | (7, 22784) | (7, 28928) => true,
        
        _ => false,
    }
}

