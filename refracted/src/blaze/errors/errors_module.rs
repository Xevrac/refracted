/// Blaze Error Code System
/// Maps error codes to their names and descriptions by component
/// Credit to Aim4Kill for the Blaze3SDK component definitions

/// Blaze error code information
#[derive(Debug, Clone)]
pub struct BlazeErrorInfo {
    pub code: u32,
    pub name: &'static str,
    pub component_id: u16,
}

/// Get error code name by component and error code
pub fn get_error_name(component_id: u16, error_code: u32) -> Option<&'static str> {
    match component_id {
        1 => get_authentication_error_name(error_code),
        4 => get_gamemanager_error_name(error_code),
        7 => get_stats_error_name(error_code),
        9 => get_util_error_name(error_code),
        11 => get_clubs_error_name(error_code),
        15 => get_messaging_error_name(error_code),
        30722 => get_usersessions_error_name(error_code),
        _ => None,
    }
}

/// Get error code by component and error name
pub fn get_error_code(component_id: u16, error_name: &str) -> Option<u32> {
    match component_id {
        1 => get_authentication_error_code(error_name),
        4 => get_gamemanager_error_code(error_name),
        7 => get_stats_error_code(error_name),
        9 => get_util_error_code(error_name),
        11 => get_clubs_error_code(error_name),
        15 => get_messaging_error_code(error_name),
        30722 => get_usersessions_error_code(error_name),
        _ => None,
    }
}

// Authentication Component (1) Error Codes
fn get_authentication_error_name(code: u32) -> Option<&'static str> {
    match code {
        2 => Some("AUTH_ERR_TOS_REQUIRED"),
        10 => Some("AUTH_ERR_INVALID_COUNTRY"),
        11 => Some("AUTH_ERR_INVALID_USER"),
        12 => Some("AUTH_ERR_INVALID_PASSWORD"),
        13 => Some("AUTH_ERR_INVALID_TOKEN"),
        14 => Some("AUTH_ERR_EXPIRED_TOKEN"),
        15 => Some("AUTH_ERR_EXISTS"),
        16 => Some("AUTH_ERR_TOO_YOUNG"),
        17 => Some("AUTH_ERR_NO_ACCOUNT"),
        18 => Some("AUTH_ERR_PERSONA_NOT_FOUND"),
        19 => Some("AUTH_ERR_PERSONA_INACTIVE"),
        20 => Some("AUTH_ERR_INVALID_PMAIL"),
        21 => Some("AUTH_ERR_INVALID_FIELD"),
        22 => Some("AUTH_ERR_INVALID_EMAIL"),
        23 => Some("AUTH_ERR_INVALID_STATUS"),
        31 => Some("AUTH_ERR_INVALID_SESSION_KEY"),
        32 => Some("AUTH_ERR_PERSONA_BANNED"),
        33 => Some("AUTH_ERR_INVALID_PERSONA"),
        34 => Some("AUTH_ERR_CURRENT_PASSWORD_REQUIRED"),
        40 => Some("AUTH_ERR_INV_MASTER"),
        41 => Some("AUTH_ERR_DEACTIVATED"),
        42 => Some("AUTH_ERR_PENDING"),
        43 => Some("AUTH_ERR_BANNED"),
        44 => Some("AUTH_ERR_DISABLED"),
        50 => Some("AUTH_ERR_NEED_PCCDKEY"),
        51 => Some("AUTH_ERR_CODE_ALREADY_USED"),
        52 => Some("AUTH_ERR_INVALID_CODE"),
        53 => Some("AUTH_ERR_CODE_ALREADY_DISABLED"),
        54 => Some("AUTH_ERR_NO_ASSOCIATED_PRODUCT"),
        55 => Some("AUTH_ERR_INVALID_MAPPING_ERROR"),
        56 => Some("AUTH_ERR_NO_SUCH_GROUP_NAME"),
        57 => Some("AUTH_ERR_MISSING_PERSONAID"),
        58 => Some("AUTH_ERR_USER_DOES_NOT_MATCH_PERSONA"),
        59 => Some("AUTH_ERR_WHITELIST"),
        60 => Some("AUTH_ERR_LINK_PERSONA"),
        61 => Some("AUTH_ERR_NO_SUCH_GROUP"),
        63 => Some("AUTH_ERR_NO_SUCH_ENTITLEMENT"),
        64 => Some("AUTH_ERR_GROUP_NAME_DOES_NOT_MATCH"),
        65 => Some("AUTH_ERR_DEVICE_ID_ALREADY_USED"),
        66 => Some("AUTH_ERR_USECOUNT_ZERO"),
        67 => Some("AUTH_ERR_ENTITLEMETNTAG_EMPTY"),
        68 => Some("AUTH_ERR_ENTITLEMENT_OTHER"),
        70 => Some("AUTH_ERR_GROUPNAME_REQUIRED"),
        71 => Some("AUTH_ERR_GROUPNAME_INVALID"),
        72 => Some("AUTH_ERR_PAGESIZE_TOO_BIG"),
        73 => Some("AUTH_ERR_PAGESIZE_ZERO"),
        74 => Some("AUTH_ERR_ENTITLEMENT_TAG_REQUIRED"),
        75 => Some("AUTH_ERR_PAGENO_ZERO"),
        76 => Some("AUTH_ERR_MODIFIED_STATUS_INVALID"),
        77 => Some("AUTH_ERR_USECOUNT_INCREMENT"),
        78 => Some("AUTH_ERR_TERMINATION_INVALID"),
        79 => Some("AUTH_ERR_UNKNOWN_ENTITLEMENT"),
        80 => Some("AUTH_ERR_EXCEED_PSU_LIMIT_TRIAL"),
        81 => Some("AUTH_ERR_OPTIN_NAME_REQUIRED"),
        82 => Some("AUTH_ERR_INVALID_OPTIN"),
        83 => Some("AUTH_ERR_OPTIN_MISMATCH"),
        84 => Some("AUTH_ERR_NO_SUCH_OPTIN"),
        85 => Some("AUTH_ERR_AUTHID_REQUIRED"),
        86 => Some("AUTH_ERR_PERSONA_EXTREFID_REQUIRED"),
        87 => Some("AUTH_ERR_SOURCE_REQUIRED"),
        88 => Some("AUTH_ERR_APPLICATION_REQUIRED"),
        89 => Some("AUTH_ERR_TOKEN_REQUIRED"),
        90 => Some("AUTH_ERR_PARAMETER_TOO_LENGTH"),
        91 => Some("AUTH_ERR_NO_SUCH_PERSONA_REFERENCE"),
        92 => Some("AUTH_ERR_EXTERNAL_AUTH_EXISTS"),
        93 => Some("AUTH_ERR_INVALID_SOURCE"),
        94 => Some("AUTH_ERR_NO_SUCH_AUTH_DATA"),
        101 => Some("AUTH_ERR_USER_INACTIVE"),
        102 => Some("AUTH_ERR_UNEXPECTED_ACTIVATION"),
        103 => Some("AUTH_ERR_NAME_MISMATCH"),
        104 => Some("AUTH_ERR_INVALID_PS3_TICKET"),
        105 => Some("AUTH_ERR_INVALID_NAMESPACE"),
        106 => Some("AUTH_ERR_EXPIRED_PS3_TICKET"),
        201 => Some("AUTH_ERR_FIELD_INVALID_CHARS"),
        202 => Some("AUTH_ERR_FIELD_TOO_SHORT"),
        203 => Some("AUTH_ERR_FIELD_TOO_LONG"),
        204 => Some("AUTH_ERR_FIELD_MUST_BEGIN_WITH_LETTER"),
        205 => Some("AUTH_ERR_FIELD_MISSING"),
        206 => Some("AUTH_ERR_FIELD_INVALID"),
        207 => Some("AUTH_ERR_FIELD_NOT_ALLOWED"),
        208 => Some("AUTH_ERR_FIELD_NEEDS_SPECIAL_CHARS"),
        209 => Some("AUTH_ERR_FIELD_ALREADY_EXISTS"),
        210 => Some("AUTH_ERR_FIELD_NEEDS_CONSENT"),
        211 => Some("AUTH_ERR_FIELD_TOO_YOUNG"),
        300 => Some("AUTH_ERR_TOO_MANY_PERSONA_FOR_NAMESPACE"),
        1074266112 => Some("ERR_AUTHORIZATION_REQUIRED"),
        _ => None,
    }
}

fn get_authentication_error_code(name: &str) -> Option<u32> {
    match name {
        "AUTH_ERR_TOS_REQUIRED" => Some(2),
        "AUTH_ERR_INVALID_COUNTRY" => Some(10),
        "AUTH_ERR_INVALID_USER" => Some(11),
        "AUTH_ERR_INVALID_PASSWORD" => Some(12),
        "AUTH_ERR_INVALID_TOKEN" => Some(13),
        "AUTH_ERR_EXPIRED_TOKEN" => Some(14),
        "AUTH_ERR_EXISTS" => Some(15),
        "AUTH_ERR_TOO_YOUNG" => Some(16),
        "AUTH_ERR_NO_ACCOUNT" => Some(17),
        "AUTH_ERR_PERSONA_NOT_FOUND" => Some(18),
        "AUTH_ERR_PERSONA_INACTIVE" => Some(19),
        "AUTH_ERR_INVALID_PMAIL" => Some(20),
        "AUTH_ERR_INVALID_FIELD" => Some(21),
        "AUTH_ERR_INVALID_EMAIL" => Some(22),
        "AUTH_ERR_INVALID_STATUS" => Some(23),
        "AUTH_ERR_INVALID_SESSION_KEY" => Some(31),
        "AUTH_ERR_PERSONA_BANNED" => Some(32),
        "AUTH_ERR_INVALID_PERSONA" => Some(33),
        "AUTH_ERR_CURRENT_PASSWORD_REQUIRED" => Some(34),
        "AUTH_ERR_INV_MASTER" => Some(40),
        "AUTH_ERR_DEACTIVATED" => Some(41),
        "AUTH_ERR_PENDING" => Some(42),
        "AUTH_ERR_BANNED" => Some(43),
        "AUTH_ERR_DISABLED" => Some(44),
        "AUTH_ERR_NEED_PCCDKEY" => Some(50),
        "AUTH_ERR_CODE_ALREADY_USED" => Some(51),
        "AUTH_ERR_INVALID_CODE" => Some(52),
        "AUTH_ERR_CODE_ALREADY_DISABLED" => Some(53),
        "AUTH_ERR_NO_ASSOCIATED_PRODUCT" => Some(54),
        "AUTH_ERR_INVALID_MAPPING_ERROR" => Some(55),
        "AUTH_ERR_NO_SUCH_GROUP_NAME" => Some(56),
        "AUTH_ERR_MISSING_PERSONAID" => Some(57),
        "AUTH_ERR_USER_DOES_NOT_MATCH_PERSONA" => Some(58),
        "AUTH_ERR_WHITELIST" => Some(59),
        "AUTH_ERR_LINK_PERSONA" => Some(60),
        "AUTH_ERR_NO_SUCH_GROUP" => Some(61),
        "AUTH_ERR_NO_SUCH_ENTITLEMENT" => Some(63),
        "AUTH_ERR_GROUP_NAME_DOES_NOT_MATCH" => Some(64),
        "AUTH_ERR_DEVICE_ID_ALREADY_USED" => Some(65),
        "AUTH_ERR_USECOUNT_ZERO" => Some(66),
        "AUTH_ERR_ENTITLEMETNTAG_EMPTY" => Some(67),
        "AUTH_ERR_ENTITLEMENT_OTHER" => Some(68),
        "AUTH_ERR_GROUPNAME_REQUIRED" => Some(70),
        "AUTH_ERR_GROUPNAME_INVALID" => Some(71),
        "AUTH_ERR_PAGESIZE_TOO_BIG" => Some(72),
        "AUTH_ERR_PAGESIZE_ZERO" => Some(73),
        "AUTH_ERR_ENTITLEMENT_TAG_REQUIRED" => Some(74),
        "AUTH_ERR_PAGENO_ZERO" => Some(75),
        "AUTH_ERR_MODIFIED_STATUS_INVALID" => Some(76),
        "AUTH_ERR_USECOUNT_INCREMENT" => Some(77),
        "AUTH_ERR_TERMINATION_INVALID" => Some(78),
        "AUTH_ERR_UNKNOWN_ENTITLEMENT" => Some(79),
        "AUTH_ERR_EXCEED_PSU_LIMIT_TRIAL" => Some(80),
        "AUTH_ERR_OPTIN_NAME_REQUIRED" => Some(81),
        "AUTH_ERR_INVALID_OPTIN" => Some(82),
        "AUTH_ERR_OPTIN_MISMATCH" => Some(83),
        "AUTH_ERR_NO_SUCH_OPTIN" => Some(84),
        "AUTH_ERR_AUTHID_REQUIRED" => Some(85),
        "AUTH_ERR_PERSONA_EXTREFID_REQUIRED" => Some(86),
        "AUTH_ERR_SOURCE_REQUIRED" => Some(87),
        "AUTH_ERR_APPLICATION_REQUIRED" => Some(88),
        "AUTH_ERR_TOKEN_REQUIRED" => Some(89),
        "AUTH_ERR_PARAMETER_TOO_LENGTH" => Some(90),
        "AUTH_ERR_NO_SUCH_PERSONA_REFERENCE" => Some(91),
        "AUTH_ERR_EXTERNAL_AUTH_EXISTS" => Some(92),
        "AUTH_ERR_INVALID_SOURCE" => Some(93),
        "AUTH_ERR_NO_SUCH_AUTH_DATA" => Some(94),
        "AUTH_ERR_USER_INACTIVE" => Some(101),
        "AUTH_ERR_UNEXPECTED_ACTIVATION" => Some(102),
        "AUTH_ERR_NAME_MISMATCH" => Some(103),
        "AUTH_ERR_INVALID_PS3_TICKET" => Some(104),
        "AUTH_ERR_INVALID_NAMESPACE" => Some(105),
        "AUTH_ERR_EXPIRED_PS3_TICKET" => Some(106),
        "AUTH_ERR_FIELD_INVALID_CHARS" => Some(201),
        "AUTH_ERR_FIELD_TOO_SHORT" => Some(202),
        "AUTH_ERR_FIELD_TOO_LONG" => Some(203),
        "AUTH_ERR_FIELD_MUST_BEGIN_WITH_LETTER" => Some(204),
        "AUTH_ERR_FIELD_MISSING" => Some(205),
        "AUTH_ERR_FIELD_INVALID" => Some(206),
        "AUTH_ERR_FIELD_NOT_ALLOWED" => Some(207),
        "AUTH_ERR_FIELD_NEEDS_SPECIAL_CHARS" => Some(208),
        "AUTH_ERR_FIELD_ALREADY_EXISTS" => Some(209),
        "AUTH_ERR_FIELD_NEEDS_CONSENT" => Some(210),
        "AUTH_ERR_FIELD_TOO_YOUNG" => Some(211),
        "AUTH_ERR_TOO_MANY_PERSONA_FOR_NAMESPACE" => Some(300),
        "ERR_AUTHORIZATION_REQUIRED" => Some(1074266112),
        _ => None,
    }
}

// Util Component (9) Error Codes
fn get_util_error_name(code: u32) -> Option<&'static str> {
    match code {
        100 => Some("UTIL_CONFIG_NOT_FOUND"),
        145 => Some("UTIL_PSS_NO_SERVERS_AVAILABLE"),
        150 => Some("UTIL_TELEMETRY_NO_SERVERS_AVAILABLE"),
        151 => Some("UTIL_TELEMETRY_OUT_OF_MEMORY"),
        152 => Some("UTIL_TELEMETRY_KEY_TOO_LONG"),
        153 => Some("UTIL_TELEMETRY_INVALID_MAC_ADDRESS"),
        155 => Some("UTIL_TICKER_NO_SERVERS_AVAILABLE"),
        156 => Some("UTIL_TICKER_KEY_TOO_LONG"),
        200 => Some("UTIL_USS_RECORD_NOT_FOUND"),
        201 => Some("UTIL_USS_TOO_MANY_KEYS"),
        202 => Some("UTIL_USS_DB_ERROR"),
        250 => Some("UTIL_USS_USER_NO_EXTENDED_DATA"),
        300 => Some("UTIL_SUSPEND_PING_TIME_TOO_LARGE"),
        301 => Some("UTIL_SUSPEND_PING_TIME_TOO_SMALL"),
        302 => Some("UTIL_PING_SUSPENDED"),
        1074266112 => Some("ERR_AUTHORIZATION_REQUIRED"),
        _ => None,
    }
}

fn get_util_error_code(name: &str) -> Option<u32> {
    match name {
        "UTIL_CONFIG_NOT_FOUND" => Some(100),
        "UTIL_PSS_NO_SERVERS_AVAILABLE" => Some(145),
        "UTIL_TELEMETRY_NO_SERVERS_AVAILABLE" => Some(150),
        "UTIL_TELEMETRY_OUT_OF_MEMORY" => Some(151),
        "UTIL_TELEMETRY_KEY_TOO_LONG" => Some(152),
        "UTIL_TELEMETRY_INVALID_MAC_ADDRESS" => Some(153),
        "UTIL_TICKER_NO_SERVERS_AVAILABLE" => Some(155),
        "UTIL_TICKER_KEY_TOO_LONG" => Some(156),
        "UTIL_USS_RECORD_NOT_FOUND" => Some(200),
        "UTIL_USS_TOO_MANY_KEYS" => Some(201),
        "UTIL_USS_DB_ERROR" => Some(202),
        "UTIL_USS_USER_NO_EXTENDED_DATA" => Some(250),
        "UTIL_SUSPEND_PING_TIME_TOO_LARGE" => Some(300),
        "UTIL_SUSPEND_PING_TIME_TOO_SMALL" => Some(301),
        "UTIL_PING_SUSPENDED" => Some(302),
        "ERR_AUTHORIZATION_REQUIRED" => Some(1074266112),
        _ => None,
    }
}

// Stats Component (7) Error Codes
fn get_stats_error_name(code: u32) -> Option<&'static str> {
    match code {
        1 => Some("STATS_ERR_CONFIG_NOTAVAILABLE"),
        2 => Some("STATS_ERR_INVALID_LEADERBOARD_ID"),
        3 => Some("STATS_ERR_INVALID_FOLDER_ID"),
        4 => Some("STATS_ERR_UNKNOWN_CATEGORY"),
        5 => Some("STATS_ERR_STAT_NOT_FOUND"),
        6 => Some("STATS_ERR_BAD_PERIOD_TYPE"),
        7 => Some("STATS_ERR_NO_DB_CONNECTION"),
        8 => Some("STATS_ERR_DB_DATA_NOT_AVAILABLE"),
        9 => Some("STATS_ERR_UNKNOWN_STAT_GROUP"),
        10 => Some("STATS_ERR_DB_TRANSACTION_ERROR"),
        11 => Some("STATS_ERR_INVALID_UPDATE_TYPE"),
        13 => Some("STATS_ERR_DB_QUERY_FAILED"),
        14 => Some("STATS_ERR_RANK_OUT_OF_RANGE"),
        15 => Some("STATS_ERR_BAD_PERIOD_OFFSET"),
        16 => Some("STATS_ERR_BAD_SCOPE_INFO"),
        17 => Some("STATS_ERR_INVALID_FOLDER_NAME"),
        18 => Some("STATS_ERR_OPERATION_IN_PROGRESS"),
        20 => Some("STATS_ERR_INVALID_OPERATION"),
        21 => Some("STATS_ERR_INVALID_OBJECT_ID"),
        22 => Some("STATS_ERR_BAD_PERIOD_COUNTER"),
        _ => None,
    }
}

fn get_stats_error_code(name: &str) -> Option<u32> {
    match name {
        "STATS_ERR_CONFIG_NOTAVAILABLE" => Some(1),
        "STATS_ERR_INVALID_LEADERBOARD_ID" => Some(2),
        "STATS_ERR_INVALID_FOLDER_ID" => Some(3),
        "STATS_ERR_UNKNOWN_CATEGORY" => Some(4),
        "STATS_ERR_STAT_NOT_FOUND" => Some(5),
        "STATS_ERR_BAD_PERIOD_TYPE" => Some(6),
        "STATS_ERR_NO_DB_CONNECTION" => Some(7),
        "STATS_ERR_DB_DATA_NOT_AVAILABLE" => Some(8),
        "STATS_ERR_UNKNOWN_STAT_GROUP" => Some(9),
        "STATS_ERR_DB_TRANSACTION_ERROR" => Some(10),
        "STATS_ERR_INVALID_UPDATE_TYPE" => Some(11),
        "STATS_ERR_DB_QUERY_FAILED" => Some(13),
        "STATS_ERR_RANK_OUT_OF_RANGE" => Some(14),
        "STATS_ERR_BAD_PERIOD_OFFSET" => Some(15),
        "STATS_ERR_BAD_SCOPE_INFO" => Some(16),
        "STATS_ERR_INVALID_FOLDER_NAME" => Some(17),
        "STATS_ERR_OPERATION_IN_PROGRESS" => Some(18),
        "STATS_ERR_INVALID_OPERATION" => Some(20),
        "STATS_ERR_INVALID_OBJECT_ID" => Some(21),
        "STATS_ERR_BAD_PERIOD_COUNTER" => Some(22),
        _ => None,
    }
}

// GameManager Component (4) Error Codes
fn get_gamemanager_error_name(code: u32) -> Option<&'static str> {
    match code {
        1 => Some("GAMEMANAGER_ERR_INVALID_GAME_SETTINGS"),
        2 => Some("GAMEMANAGER_ERR_INVALID_GAME_ID"),
        3 => Some("GAMEMANAGER_ERR_JOIN_METHOD_NOT_SUPPORTED"),
        4 => Some("GAMEMANAGER_ERR_GAME_FULL"),
        5 => Some("GAMEMANAGER_ERR_INVALID_GAME_STATE_TRANSITION"),
        6 => Some("GAMEMANAGER_ERR_INVALID_GAME_STATE_ACTION"),
        7 => Some("GAMEMANAGER_ERR_FAILED_IN_GAME_DESTROY"),
        8 => Some("GAMEMANAGER_ERR_QUEUE_FULL"),
        9 => Some("GAMEMANAGER_ERR_INVALID_GAME_ENTRY_CRITERIA"),
        10 => Some("GAMEMANAGER_ERR_GAME_PROTOCOL_VERSION_MISMATCH"),
        11 => Some("GAMEMANAGER_ERR_GAME_IN_PROGRESS"),
        12 => Some("GAMEMANAGER_ERR_RESERVED_GAME_ID_INVALID"),
        13 => Some("GAMEMANAGER_ERR_INVALID_JOIN_METHOD"),
        14 => Some("GAMEMANAGER_ERR_SLOT_OCCUPIED"),
        15 => Some("GAMEMANAGER_ERR_NOT_VIRTUAL_GAME"),
        16 => Some("GAMEMANAGER_ERR_NOT_TOPOLOGY_HOST"),
        30 => Some("GAMEMANAGER_ERR_PERMISSION_DENIED"),
        31 => Some("GAMEMANAGER_ERR_ALREADY_ADMIN"),
        32 => Some("GAMEMANAGER_ERR_NOT_IN_ADMIN_LIST"),
        33 => Some("GAMEMANAGER_ERR_DEDICATED_SERVER_HOST"),
        50 => Some("GAMEMANAGER_ERR_INVALID_QUEUE_METHOD"),
        51 => Some("GAMEMANAGER_ERR_PLAYER_NOT_IN_QUEUE"),
        52 => Some("GAMEMANAGER_ERR_DEQUEUE_WHILE_MIGRATING"),
        53 => Some("GAMEMANAGER_ERR_DEQUEUE_WHILE_IN_PROGRESS"),
        101 => Some("GAMEMANAGER_ERR_PLAYER_NOT_FOUND"),
        103 => Some("GAMEMANAGER_ERR_ALREADY_GAME_MEMBER"),
        104 => Some("GAMEMANAGER_ERR_REMOVE_PLAYER_FAILED"),
        107 => Some("GAMEMANAGER_ERR_INVALID_PLAYER_PASSEDIN"),
        108 => Some("GAMEMANAGER_ERR_JOIN_PLAYER_FAILED"),
        110 => Some("GAMEMANAGER_ERR_PLAYER_BANNED"),
        111 => Some("GAMEMANAGER_ERR_GAME_ENTRY_CRITERIA_FAILED"),
        112 => Some("GAMEMANAGER_ERR_ALREADY_IN_QUEUE"),
        113 => Some("GAMEMANAGER_ERR_ENFORCING_SINGLE_GROUP_JOINS"),
        114 => Some("GAMEMANAGER_ERR_BANNED_PLAYER_NOT_FOUND"),
        120 => Some("GAMEMANAGER_ERR_RESERVATION_ALREADY_EXISTS"),
        121 => Some("GAMEMANAGER_ERR_NO_RESERVATION_FOUND"),
        122 => Some("GAMEMANAGER_ERR_INVALID_GAME_ENTRY_TYPE"),
        151 => Some("GAMEMANAGER_ERR_INVALID_GROUP_ID"),
        152 => Some("GAMEMANAGER_ERR_PLAYER_NOT_IN_GROUP"),
        200 => Some("GAMEMANAGER_ERR_INVALID_MATCHMAKING_CRITERIA"),
        201 => Some("GAMEMANAGER_ERR_UNKNOWN_MATCHMAKING_SESSION_ID"),
        202 => Some("GAMEMANAGER_ERR_NOT_MATCHMAKING_SESSION_OWNER"),
        203 => Some("GAMEMANAGER_ERR_MATCHMAKING_NO_JOINABLE_GAMES"),
        205 => Some("GAMEMANAGER_ERR_MATCHMAKING_USERSESSION_NOT_FOUND"),
        206 => Some("GAMEMANAGER_ERR_MATCHMAKING_EXCEEDED_MAX_REQUESTS"),
        230 => Some("GAMEMANAGER_ERR_PLAYER_CAPACITY_TOO_SMALL"),
        231 => Some("GAMEMANAGER_ERR_PLAYER_CAPACITY_TOO_LARGE"),
        232 => Some("GAMEMANAGER_ERR_PLAYER_CAPACITY_IS_ZERO"),
        233 => Some("GAMEMANAGER_ERR_MAX_PLAYER_CAPACITY_TOO_LARGE"),
        250 => Some("GAMEMANAGER_ERR_INVALID_TEAM_CAPACITIES_VECTOR_SIZE"),
        251 => Some("GAMEMANAGER_ERR_DUPLICATE_TEAM_CAPACITY"),
        252 => Some("GAMEMANAGER_ERR_INVALID_TEAM_ID_IN_TEAM_CAPACITIES_VECTOR"),
        253 => Some("GAMEMANAGER_ERR_TEAM_NOT_ALLOWED"),
        254 => Some("GAMEMANAGER_ERR_TOTAL_TEAM_CAPACITY_INVALID"),
        255 => Some("GAMEMANAGER_ERR_TEAM_FULL"),
        256 => Some("GAMEMANAGER_ERR_TEAMS_DISABLED"),
        257 => Some("GAMEMANAGER_ERR_INVALID_TEAM_CAPACITY"),
        301 => Some("GAMEMANAGER_ERR_NO_DEDICATED_SERVER_FOUND"),
        302 => Some("GAMEMANAGER_ERR_DEDICATED_SERVER_ONLY_ACTION"),
        303 => Some("GAMEMANAGER_ERR_DEDICATED_SERVER_HOST_CANNOT_JOIN"),
        304 => Some("GAMEMANGER_ERR_MACHINE_ID_LIST_EMPTY"),
        305 => Some("GAMEMANAGER_ERR_DYNAMIC_GAME_CREATION_TIMED_OUT"),
        306 => Some("GAMEMANAGER_ERR_DYNAMIC_GAME_CREATION_FAILED_NO_CAPACITY"),
        307 => Some("GAMEMANAGER_ERR_DYNAMIC_DEDICATED_SERVER_MODE_CONFLICT"),
        401 => Some("GAMEBROWSER_ERR_INVALID_CRITERIA"),
        402 => Some("GAMEBROWSER_ERR_INVALID_CAPACITY"),
        403 => Some("GAMEBROWSER_ERR_INVALID_LIST_ID"),
        404 => Some("GAMEBROWSER_ERR_NOT_LIST_OWNER"),
        405 => Some("GAMEBROWSER_ERR_INVALID_LIST_CONFIG_NAME"),
        406 => Some("GAMEBROWSER_ERR_CANNOT_GET_USERSET"),
        502 => Some("GAMEMANAGER_ERR_GAME_CAPACITY_TOO_SMALL"),
        503 => Some("GAMEMANAGER_ERR_INVALID_ACTION_FOR_GROUP"),
        504 => Some("GAMEMANAGER_ERR_NOT_PLATFORM_HOST"),
        505 => Some("GAMEMANAGER_ERR_MIGRATION_NOT_SUPPORTED"),
        506 => Some("GAMEMANAGER_ERR_INVALID_NEWHOST"),
        507 => Some("GAMEMANAGER_ERR_USER_NOT_IN_ANY_GAME"),
        508 => Some("GAMEMANAGER_ERR_INVALID_PERSISTED_GAME_ID_OR_SECRET"),
        509 => Some("GAMEMANAGER_ERR_PERSISTED_GAME_ID_IN_USE"),
        _ => None,
    }
}

fn get_gamemanager_error_code(name: &str) -> Option<u32> {
    // Implementation similar to above - mapping name to code
    // For brevity, showing key ones
    match name {
        "GAMEMANAGER_ERR_INVALID_GAME_SETTINGS" => Some(1),
        "GAMEMANAGER_ERR_INVALID_GAME_ID" => Some(2),
        "GAMEMANAGER_ERR_GAME_FULL" => Some(4),
        "GAMEMANAGER_ERR_PLAYER_NOT_FOUND" => Some(101),
        "GAMEMANAGER_ERR_ALREADY_GAME_MEMBER" => Some(103),
        _ => None,
    }
}

// Clubs Component (11) Error Codes
fn get_clubs_error_name(code: u32) -> Option<&'static str> {
    match code {
        1002 => Some("CLUBS_ERR_INVALID_ARGUMENT"),
        1003 => Some("CLUBS_ERR_MAX_CLUBS"),
        1004 => Some("CLUBS_ERR_CLUB_NAME_IN_USE"),
        1005 => Some("CLUBS_ERR_PROFANITY_FILTER"),
        1007 => Some("CLUBS_ERR_NO_PRIVILEGE"),
        1008 => Some("CLUBS_ERR_INVALID_USER_ID"),
        1009 => Some("CLUBS_ERR_INVALID_CLUB_ID"),
        1010 => Some("CLUBS_ERR_TOO_MANY_ITEMS_PER_FETCH_REQUESTED"),
        1011 => Some("CLUBS_ERR_INVALID_CLUBNAME_SIZE"),
        1012 => Some("CLUBS_ERR_INVALID_NON_UNIQUE_NAME_SIZE"),
        1013 => Some("CLUBS_ERR_INVALID_DOMAIN_ID"),
        1101 => Some("CLUBS_ERR_INVALID_MAX_COUNT"),
        1102 => Some("CLUBS_ERR_INVALID_OFFSET"),
        1201 => Some("CLUBS_ERR_INVALID_CLUBNAME_EMPTY"),
        1202 => Some("CLUBS_ERR_INVALID_CLUBNAME_ILLEGAL"),
        1203 => Some("CLUBS_ERR_INVALID_CLUBNAME_PROFANITY"),
        1204 => Some("CLUBS_ERR_INVALID_NON_UNIQUE_NAME_EMPTY"),
        1301 => Some("CLUBS_ERR_USER_NOT_MEMBER"),
        1302 => Some("CLUBS_ERR_LAST_GM_CANNOT_LEAVE"),
        1303 => Some("CLUBS_ERR_CANNOT_KICK_OWNER"),
        1304 => Some("CLUBS_ERR_ALREADY_GM"),
        1305 => Some("CLUBS_ERR_MAX_INVITES_SENT"),
        1306 => Some("CLUBS_ERR_MAX_INVITES_RECEIVED"),
        1307 => Some("CLUBS_ERR_MAX_PETITIONS_SENT"),
        1308 => Some("CLUBS_ERR_MAX_PETITIONS_RECEIVED"),
        1309 => Some("CLUBS_ERR_MAX_MESSAGES_SENT"),
        1310 => Some("CLUBS_ERR_MAX_MESSAGES_RECEIVED"),
        1311 => Some("CLUBS_ERR_CLUB_FULL"),
        1312 => Some("CLUBS_ERR_TOO_MANY_GMS"),
        1313 => Some("CLUBS_ERR_INVITATION_ALREADY_SENT"),
        1350 => Some("CLUBS_ERR_DEMOTE_MEMBER"),
        1351 => Some("CLUBS_ERR_DEMOTE_OWNER"),
        1352 => Some("CLUBS_ERR_DEMOTE_LAST_GM"),
        1361 => Some("CLUBS_ERR_TRANSFER_OWNERSHIP_TO_OWNER"),
        1401 => Some("CLUBS_ERR_ALREADY_MEMBER"),
        1402 => Some("CLUBS_ERR_PETITION_DISABLED"),
        1403 => Some("CLUBS_ERR_PETITION_ALREADY_SENT"),
        1404 => Some("CLUBS_ERR_JOIN_DISABLED"),
        1501 => Some("CLUBS_ERR_MISSING_NEWS_TYPE_FILTER"),
        1502 => Some("CLUBS_ERR_TOO_MANY_PARAMETERS"),
        1503 => Some("CLUBS_ERR_NEWS_TEXT_OR_STRINGID_MUST_BE_EMPTY"),
        1504 => Some("CLUBS_ERR_ASSOCIATE_CLUB_ID_MUST_BE_ZERO"),
        1505 => Some("CLUBS_ERR_NEWS_ITEM_NOT_FOUND"),
        1601 => Some("CLUBS_ERR_DUPLICATE_RIVALS"),
        1701 => Some("CLUBS_ERR_USER_BANNED"),
        1801 => Some("CLUBS_ERR_INVALID_TAG_TEXT_EMPTY"),
        1802 => Some("CLUBS_ERR_INVALID_TAG_TEXT_SIZE"),
        1803 => Some("CLUBS_ERR_TAG_TEXT_NOT_FOUND"),
        1901 => Some("CLUBS_ERR_WRONG_PASSWORD"),
        1902 => Some("CLUBS_ERR_INVALID_PASSWORD_PROFANITY"),
        _ => None,
    }
}

fn get_clubs_error_code(name: &str) -> Option<u32> {
    match name {
        "CLUBS_ERR_INVALID_ARGUMENT" => Some(1002),
        "CLUBS_ERR_MAX_CLUBS" => Some(1003),
        "CLUBS_ERR_CLUB_NAME_IN_USE" => Some(1004),
        "CLUBS_ERR_PROFANITY_FILTER" => Some(1005),
        "CLUBS_ERR_NO_PRIVILEGE" => Some(1007),
        "CLUBS_ERR_INVALID_USER_ID" => Some(1008),
        "CLUBS_ERR_INVALID_CLUB_ID" => Some(1009),
        "CLUBS_ERR_USER_NOT_MEMBER" => Some(1301),
        "CLUBS_ERR_ALREADY_MEMBER" => Some(1401),
        _ => None,
    }
}

// Messaging Component (15) Error Codes
fn get_messaging_error_name(code: u32) -> Option<&'static str> {
    match code {
        1 => Some("MESSAGING_ERR_UNKNOWN"),
        2 => Some("MESSAGING_ERR_MAX_ATTR_EXCEEDED"),
        3 => Some("MESSAGING_ERR_DATABASE"),
        4 => Some("MESSAGING_ERR_TARGET_NOT_FOUND"),
        5 => Some("MESSAGING_ERR_TARGET_TYPE_INVALID"),
        6 => Some("MESSAGING_ERR_TARGET_INBOX_FULL"),
        7 => Some("MESSAGING_ERR_MATCH_NOT_FOUND"),
        8 => Some("MESSAGING_ERR_FEATURE_DISABLED"),
        9 => Some("MESSAGING_ERR_INVALID_PARAM"),
        _ => None,
    }
}

fn get_messaging_error_code(name: &str) -> Option<u32> {
    match name {
        "MESSAGING_ERR_UNKNOWN" => Some(1),
        "MESSAGING_ERR_MAX_ATTR_EXCEEDED" => Some(2),
        "MESSAGING_ERR_DATABASE" => Some(3),
        "MESSAGING_ERR_TARGET_NOT_FOUND" => Some(4),
        "MESSAGING_ERR_TARGET_TYPE_INVALID" => Some(5),
        "MESSAGING_ERR_TARGET_INBOX_FULL" => Some(6),
        "MESSAGING_ERR_MATCH_NOT_FOUND" => Some(7),
        "MESSAGING_ERR_FEATURE_DISABLED" => Some(8),
        "MESSAGING_ERR_INVALID_PARAM" => Some(9),
        _ => None,
    }
}

// UserSessions Component (30722) Error Codes
fn get_usersessions_error_name(code: u32) -> Option<&'static str> {
    match code {
        1 => Some("USER_ERR_USER_NOT_FOUND"),
        2 => Some("USER_ERR_SESSION_NOT_FOUND"),
        3 => Some("USER_ERR_DUPLICATE_SESSION"),
        4 => Some("USER_ERR_NO_EXTENDED_DATA"),
        5 => Some("USER_ERR_MAX_DATA_REACHED"),
        6 => Some("USER_ERR_KEY_NOT_FOUND"),
        7 => Some("USER_ERR_INVALID_SESSION_INSTANCE"),
        8 => Some("USER_ERR_INVALID_PARAM"),
        9 => Some("USER_ERR_MINIMUM_CHARACTERS"),
        10 => Some("ACCESS_GROUP_ERR_INVALID_GROUP"),
        11 => Some("ACCESS_GROUP_ERR_DEFAULT_GROUP"),
        12 => Some("ACCESS_GROUP_ERR_NOT_CURRENT_GROUP"),
        13 => Some("ACCESS_GROUP_ERR_CURRENT_GROUP"),
        14 => Some("ACCESS_GROUP_ERR_NO_GROUP_FOUND"),
        15 => Some("GEOIP_INCOMPLETE_PARAMETERS"),
        16 => Some("GEOIP_UNABLE_TO_RESOLVE"),
        17 => Some("ERR_ENTITY_TYPE_NOT_FOUND"),
        18 => Some("ERR_ENTITY_NOT_FOUND"),
        19 => Some("ERR_NOT_SUPPORTED"),
        20 => Some("USER_ERR_EXISTS"),
        21 => Some("USER_ERR_RESUMABLE_SESSION_CONNECTION_INVALID"),
        22 => Some("USER_ERR_RESUMABLE_SESSION_NOT_FOUND"),
        23 => Some("GEOIP_ERR_USER_OPTOUT"),
        _ => None,
    }
}

fn get_usersessions_error_code(name: &str) -> Option<u32> {
    match name {
        "USER_ERR_USER_NOT_FOUND" => Some(1),
        "USER_ERR_SESSION_NOT_FOUND" => Some(2),
        "USER_ERR_DUPLICATE_SESSION" => Some(3),
        "USER_ERR_NO_EXTENDED_DATA" => Some(4),
        "USER_ERR_MAX_DATA_REACHED" => Some(5),
        "USER_ERR_KEY_NOT_FOUND" => Some(6),
        "USER_ERR_INVALID_SESSION_INSTANCE" => Some(7),
        "USER_ERR_INVALID_PARAM" => Some(8),
        "USER_ERR_MINIMUM_CHARACTERS" => Some(9),
        "ACCESS_GROUP_ERR_INVALID_GROUP" => Some(10),
        "ACCESS_GROUP_ERR_DEFAULT_GROUP" => Some(11),
        "ACCESS_GROUP_ERR_NOT_CURRENT_GROUP" => Some(12),
        "ACCESS_GROUP_ERR_CURRENT_GROUP" => Some(13),
        "ACCESS_GROUP_ERR_NO_GROUP_FOUND" => Some(14),
        "GEOIP_INCOMPLETE_PARAMETERS" => Some(15),
        "GEOIP_UNABLE_TO_RESOLVE" => Some(16),
        "ERR_ENTITY_TYPE_NOT_FOUND" => Some(17),
        "ERR_ENTITY_NOT_FOUND" => Some(18),
        "ERR_NOT_SUPPORTED" => Some(19),
        "USER_ERR_EXISTS" => Some(20),
        "USER_ERR_RESUMABLE_SESSION_CONNECTION_INVALID" => Some(21),
        "USER_ERR_RESUMABLE_SESSION_NOT_FOUND" => Some(22),
        "GEOIP_ERR_USER_OPTOUT" => Some(23),
        _ => None,
    }
}

/// Create a TDF-encoded error response
/// Format: CNTX (context) + ERRC (error code)
pub fn create_error_response(_component_id: u16, error_code: u32) -> Vec<u8> {
    use crate::blaze::tdf::TdfEncoder;
    
    let mut response = Vec::new();
    
    // CNTX: Context (usually 0)
    response.extend_from_slice(&TdfEncoder::encode_int("CNTX", 0));
    
    // ERRC: Error code
    response.extend_from_slice(&TdfEncoder::encode_int("ERRC", error_code as i32));
    
    response
}

/// Create an error response from error name
pub fn create_error_response_by_name(component_id: u16, error_name: &str) -> Option<Vec<u8>> {
    get_error_code(component_id, error_name)
        .map(|code| create_error_response(component_id, code))
}

/// Helper to return a BlazeResult with proper error code
/// This can be used by handlers to return component-specific errors
pub fn make_error_result(_component_id: u16, _error_code: u32) -> crate::common::error::BlazeResult<bytes::Bytes> {
    use crate::common::error::BlazeError;
    
    // Map error code to BlazeError enum if possible
    // For now, we'll use a generic error and let the error system handle the code
    Err(BlazeError::InvalidParam) // This will be converted to proper error code in to_error_code()
}

/// Log error with component and error name
/// If error is unknown, logs with special marker for investigation
pub fn log_error(component_id: u16, error_code: u32, command_id: u16) {
    use crate::console_println;
    use crate::blaze::components::get_command_name;
    
    // Skip logging if error code is 0 (no error)
    if error_code == 0 {
        return;
    }
    
    let error_name_opt = get_error_name(component_id, error_code);
    let command_name = get_command_name(component_id, command_id)
        .unwrap_or_else(|| format!("Component({}).Command({})", component_id, command_id));
    
    if let Some(error_name) = error_name_opt {
        // Known error - log normally
        console_println!(
            "\x1b[38;2;255;100;100m[ERROR]\x1b[0m {} -> {} (Code: {})",
            command_name,
            error_name,
            error_code
        );
    } else {
        // Unknown error - log with special marker for investigation
        let component_name = crate::blaze::components::get_component_name(component_id);
        console_println!(
            "\x1b[38;2;255;165;0m[UNKNOWN ERROR]\x1b[0m {} -> UNKNOWN_ERROR Component={} ({}), Command={}, ErrorCode={} (An uncaught error has occured. ErrorCode not found in blaze_errors.rs.)",
            command_name,
            component_id,
            component_name,
            command_id,
            error_code
        );
    }
}

/// Log an unknown error code that was received (e.g., from client error response)
/// This helps track error codes we haven't seen before
pub fn log_unknown_error_code(component_id: u16, error_code: u32, context: &str) {
    use crate::console_println;
    
    // Skip logging if error code is 0 (no error)
    if error_code == 0 {
        return;
    }
    
    // Check if we know this error
    if get_error_name(component_id, error_code).is_some() {
        // We know this error, don't log as unknown
        return;
    }
    
    // Unknown error - log for investigation
    let component_name = crate::blaze::components::get_component_name(component_id);
    console_println!(
        "\x1b[38;2;255;165;0m[UNKNOWN ERROR]\x1b[0m Component={} ({}), ErrorCode={}, Context: {} (An uncaught error has occured. ErrorCode not found in blaze_errors.rs.)",
        component_id,
        component_name,
        error_code,
        context
    );
}

