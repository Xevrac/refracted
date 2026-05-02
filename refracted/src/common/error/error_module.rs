use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlazeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),

    #[error("Certificate generation error: {0}")]
    CertificateGeneration(#[from] rcgen::Error),

    #[error("HTTP/2 error: {0}")]
    Http2(String),

    #[error("Invalid packet format: {0}")]
    InvalidPacket(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("TDF encoding error: {0}")]
    TdfEncoding(String),

    #[error("Unknown component/command: component={0}, command={1}")]
    UnknownCommand(u16, u16),

    #[error("Connection closed")]
    ConnectionClosed,

    // UserSessions Component Errors
    #[error("The referenced user was not found")]
    UserNotFound,

    #[error("The referenced session was not found")]
    SessionNotFound,

    #[error("The session could not be added because one from that user already exists")]
    DuplicateSession,

    #[error("The extended data could not be returned because there is no extended data for the given session")]
    NoExtendedData,

    #[error("The extended data attribute could not be added because the maximum number of attributes has been reached for this session")]
    MaxDataReached,

    #[error("The extended data attribute could not be removed because key was not found")]
    KeyNotFound,

    #[error("The session did not belong to the calling instance")]
    InvalidSessionInstance,

    #[error("Invalid parameter(s)")]
    InvalidParam,

    #[error("The minimum of characters is 3")]
    MinimumCharacters,

    #[error("A duplicate user already exists")]
    UserExists,

    #[error("Attempted to resume a session on an unsupported connection type")]
    ResumableSessionConnectionInvalid,

    #[error("Specified session to resume was not found")]
    ResumableSessionNotFound,

    // Access Group Errors
    #[error("The specified access group was not found")]
    AccessGroupInvalidGroup,

    #[error("The specified access group is default group for the referenced user")]
    AccessGroupDefaultGroup,

    #[error("The referenced user does not belong to the specified group")]
    AccessGroupNotCurrentGroup,

    #[error("The referenced user already belong to the specified group")]
    AccessGroupCurrentGroup,

    #[error("There is no group is found for specified external id and client type")]
    AccessGroupNoGroupFound,

    // GeoIP Errors
    #[error("Parameters in GeoIp request are incomplete: city, state region and country must be supplied")]
    GeoIpIncompleteParameters,

    #[error("Unable to resolve latitude & longitude because GeoIp request contains unknown city, state region or country or there was HTTP error")]
    GeoIpUnableToResolve,

    #[error("The requested user with opt-in field disabled")]
    GeoIpUserOptout,

    // Entity Errors
    #[error("The entity type is not recognized by the component")]
    EntityTypeNotFound,

    #[error("No entity is found matching the type name and name provided")]
    EntityNotFound,

    #[error("The entity provided is recognized, but searching by name is not supported")]
    NotSupported,

    // Authorization Errors
    #[error("Authorization required")]
    AuthorizationRequired,
}

pub type BlazeResult<T> = Result<T, BlazeError>;

// UserSessions Component Error Codes
pub const USER_ERR_USER_NOT_FOUND: u32 = 1;
pub const USER_ERR_SESSION_NOT_FOUND: u32 = 2;
pub const USER_ERR_DUPLICATE_SESSION: u32 = 3;
pub const USER_ERR_NO_EXTENDED_DATA: u32 = 4;
pub const USER_ERR_MAX_DATA_REACHED: u32 = 5;
pub const USER_ERR_KEY_NOT_FOUND: u32 = 6;
pub const USER_ERR_INVALID_SESSION_INSTANCE: u32 = 7;
pub const USER_ERR_INVALID_PARAM: u32 = 8;
pub const USER_ERR_MINIMUM_CHARACTERS: u32 = 9;
pub const USER_ERR_EXISTS: u32 = 20;
pub const USER_ERR_RESUMABLE_SESSION_CONNECTION_INVALID: u32 = 21;
pub const USER_ERR_RESUMABLE_SESSION_NOT_FOUND: u32 = 22;

// Access Group Error Codes
pub const ACCESS_GROUP_ERR_INVALID_GROUP: u32 = 10;
pub const ACCESS_GROUP_ERR_DEFAULT_GROUP: u32 = 11;
pub const ACCESS_GROUP_ERR_NOT_CURRENT_GROUP: u32 = 12;
pub const ACCESS_GROUP_ERR_CURRENT_GROUP: u32 = 13;
pub const ACCESS_GROUP_ERR_NO_GROUP_FOUND: u32 = 14;

// GeoIP Error Codes
pub const GEOIP_INCOMPLETE_PARAMETERS: u32 = 15;
pub const GEOIP_UNABLE_TO_RESOLVE: u32 = 16;
pub const GEOIP_ERR_USER_OPTOUT: u32 = 23;

// Entity Error Codes
pub const ERR_ENTITY_TYPE_NOT_FOUND: u32 = 17;
pub const ERR_ENTITY_NOT_FOUND: u32 = 18;
pub const ERR_NOT_SUPPORTED: u32 = 19;

// Authorization Error Codes
pub const ERR_AUTHORIZATION_REQUIRED: u32 = 1074266112;

impl BlazeError {
    /// Convert BlazeError to its corresponding error code
    pub fn to_error_code(&self) -> u32 {
        match self {
            // UserSessions Component Errors
            BlazeError::UserNotFound => USER_ERR_USER_NOT_FOUND,
            BlazeError::SessionNotFound => USER_ERR_SESSION_NOT_FOUND,
            BlazeError::DuplicateSession => USER_ERR_DUPLICATE_SESSION,
            BlazeError::NoExtendedData => USER_ERR_NO_EXTENDED_DATA,
            BlazeError::MaxDataReached => USER_ERR_MAX_DATA_REACHED,
            BlazeError::KeyNotFound => USER_ERR_KEY_NOT_FOUND,
            BlazeError::InvalidSessionInstance => USER_ERR_INVALID_SESSION_INSTANCE,
            BlazeError::InvalidParam => USER_ERR_INVALID_PARAM,
            BlazeError::MinimumCharacters => USER_ERR_MINIMUM_CHARACTERS,
            BlazeError::UserExists => USER_ERR_EXISTS,
            BlazeError::ResumableSessionConnectionInvalid => {
                USER_ERR_RESUMABLE_SESSION_CONNECTION_INVALID
            }
            BlazeError::ResumableSessionNotFound => USER_ERR_RESUMABLE_SESSION_NOT_FOUND,

            // Access Group Errors
            BlazeError::AccessGroupInvalidGroup => ACCESS_GROUP_ERR_INVALID_GROUP,
            BlazeError::AccessGroupDefaultGroup => ACCESS_GROUP_ERR_DEFAULT_GROUP,
            BlazeError::AccessGroupNotCurrentGroup => ACCESS_GROUP_ERR_NOT_CURRENT_GROUP,
            BlazeError::AccessGroupCurrentGroup => ACCESS_GROUP_ERR_CURRENT_GROUP,
            BlazeError::AccessGroupNoGroupFound => ACCESS_GROUP_ERR_NO_GROUP_FOUND,

            // GeoIP Errors
            BlazeError::GeoIpIncompleteParameters => GEOIP_INCOMPLETE_PARAMETERS,
            BlazeError::GeoIpUnableToResolve => GEOIP_UNABLE_TO_RESOLVE,
            BlazeError::GeoIpUserOptout => GEOIP_ERR_USER_OPTOUT,

            // Entity Errors
            BlazeError::EntityTypeNotFound => ERR_ENTITY_TYPE_NOT_FOUND,
            BlazeError::EntityNotFound => ERR_ENTITY_NOT_FOUND,
            BlazeError::NotSupported => ERR_NOT_SUPPORTED,

            // Authorization Errors
            BlazeError::AuthorizationRequired => ERR_AUTHORIZATION_REQUIRED,

                    // Generic errors - try to map to component-specific errors
                    BlazeError::UnknownCommand(component, command) => {
                        // Special case: Component 9 Command 22 (setClientMetrics) should return authorization error
                        if *component == 9 && *command == 22 {
                            ERR_AUTHORIZATION_REQUIRED
                        } else {
                            // For other unknown commands, return 0 (no error) or component-specific error
                            // Most components use error code 0 for "no error" but we'll use a generic error
                            0 // Will be handled as UNKNOWN_ERROR
                        }
                    }
            // Generic errors (use 0 for unknown)
            _ => 0,
        }
    }
    
    /// Get component ID for this error (if applicable)
    pub fn component_id(&self) -> Option<u16> {
        if self.is_user_sessions_error() {
            Some(30722) // UserSessions component
        } else {
            None
        }
    }

    /// Check if this error is a UserSessions component error
    pub fn is_user_sessions_error(&self) -> bool {
        matches!(
            self,
            BlazeError::UserNotFound
                | BlazeError::SessionNotFound
                | BlazeError::DuplicateSession
                | BlazeError::NoExtendedData
                | BlazeError::MaxDataReached
                | BlazeError::KeyNotFound
                | BlazeError::InvalidSessionInstance
                | BlazeError::InvalidParam
                | BlazeError::MinimumCharacters
                | BlazeError::UserExists
                | BlazeError::ResumableSessionConnectionInvalid
                | BlazeError::ResumableSessionNotFound
                | BlazeError::AccessGroupInvalidGroup
                | BlazeError::AccessGroupDefaultGroup
                | BlazeError::AccessGroupNotCurrentGroup
                | BlazeError::AccessGroupCurrentGroup
                | BlazeError::AccessGroupNoGroupFound
                | BlazeError::GeoIpIncompleteParameters
                | BlazeError::GeoIpUnableToResolve
                | BlazeError::GeoIpUserOptout
                | BlazeError::EntityTypeNotFound
                | BlazeError::EntityNotFound
                | BlazeError::NotSupported
        )
    }
}

// Custom From implementation for h2::Error
impl From<h2::Error> for BlazeError {
    fn from(err: h2::Error) -> Self {
        BlazeError::Http2(err.to_string())
    }
}

/// True when a TCP read/write failed because the remote side closed or reset (normal for game clients).
pub fn io_is_expected_peer_close(e: &std::io::Error) -> bool {
    use std::io::ErrorKind::*;
    match e.kind() {
        ConnectionReset | ConnectionAborted | BrokenPipe => true,
        UnexpectedEof => true,
        _ => {
            #[cfg(windows)]
            {
                matches!(e.raw_os_error(), Some(10053 | 10054))
            }
            #[cfg(not(windows))]
            {
                false
            }
        }
    }
}
