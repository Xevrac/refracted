use crate::blaze::tdf::TdfEncoder;
use crate::client::labs::LABS_SESSION_OBJECT_ID;
use crate::common::error::BlazeResult;
use crate::session::session_module::UserSession;
use bytes::Bytes;

/// `USER` struct for UserAdded / lookupUser — field order matches Blaze `UserAdded` (`AID`, `AIDS`, `ALOC`, …).
fn encode_user_identification_inner(session: &UserSession) -> Vec<u8> {
    let mut user_struct = Vec::new();
    user_struct.extend_from_slice(&TdfEncoder::encode_long("AID ", session.user_id as i64));

    let mut aids_struct = Vec::new();
    let mut eaid_struct = Vec::new();
    eaid_struct.extend_from_slice(&TdfEncoder::encode_string("NAME", &session.display_name));
    eaid_struct.extend_from_slice(&TdfEncoder::encode_long("NID ", session.user_id as i64));
    eaid_struct.extend_from_slice(&TdfEncoder::encode_long("PCID", session.persona_id as i64));
    aids_struct.extend_from_slice(&TdfEncoder::encode_struct("EAID", &eaid_struct));
    let mut exid_struct = Vec::new();
    exid_struct.extend_from_slice(&TdfEncoder::encode_int("PSID", session.psid as i32));
    exid_struct.extend_from_slice(&TdfEncoder::encode_int("STID", 0));
    exid_struct.extend_from_slice(&TdfEncoder::encode_string("SWID", ""));
    exid_struct.extend_from_slice(&TdfEncoder::encode_int("XBID", 0));
    aids_struct.extend_from_slice(&TdfEncoder::encode_struct("EXID", &exid_struct));
    aids_struct.extend_from_slice(&TdfEncoder::encode_int("PLAT", 4));
    user_struct.extend_from_slice(&TdfEncoder::encode_struct("AIDS", &aids_struct));

    user_struct.extend_from_slice(&TdfEncoder::encode_int("ALOC", 2053657939));
    user_struct.extend_from_slice(&TdfEncoder::encode_int("CNTY", 21843));
    user_struct.extend_from_slice(&TdfEncoder::encode_int("EXID", 0));
    user_struct.extend_from_slice(&TdfEncoder::encode_long("ID  ", session.persona_id as i64));
    user_struct.extend_from_slice(&TdfEncoder::encode_int("ISPP", 1));
    user_struct.extend_from_slice(&TdfEncoder::encode_string("NAME", &session.display_name));
    user_struct.extend_from_slice(&TdfEncoder::encode_string("NASP", "cem_ea_id"));
    user_struct.extend_from_slice(&TdfEncoder::encode_long("ORIG", session.persona_id as i64));
    user_struct.extend_from_slice(&TdfEncoder::encode_long("PIDI", session.user_id as i64));
    user_struct
}

/// Encode a TDF list where each item is a struct payload.
fn encode_struct_list(tag: &str, structs: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    let tag_encoded = TdfEncoder::make_tag(tag);
    out.push(tag_encoded[0]);
    out.push(tag_encoded[1]);
    out.push(tag_encoded[2]);
    out.push(0x4); // LIST
    out.push(0x3); // STRUCT item type
    out.extend_from_slice(&TdfEncoder::encode_varint(structs.len() as u64));
    for s in structs {
        out.extend_from_slice(s);
        out.push(0x00); // struct terminator
    }
    out
}

fn encode_lookup_users_edat_placeholder() -> Vec<u8> {
    use std::collections::HashMap;
    let mut edat = Vec::new();
    edat.extend_from_slice(&TdfEncoder::encode_string("BPS ", ""));
    edat.extend_from_slice(&TdfEncoder::encode_string("CTY ", ""));
    edat.extend_from_slice(&TdfEncoder::encode_int("CTYP", 0));
    edat.extend_from_slice(&TdfEncoder::encode_list("CVAR", &[]));
    let mut dmap = HashMap::new();
    dmap.insert(2013396993, 0);
    edat.extend_from_slice(&TdfEncoder::encode_int_map("DMAP", &dmap));
    edat.extend_from_slice(&TdfEncoder::encode_int("HWFG", 0));
    edat.extend_from_slice(&TdfEncoder::encode_string("ISP ", ""));
    let mut qdat = Vec::new();
    qdat.extend_from_slice(&TdfEncoder::encode_int("BWHR", 0));
    qdat.extend_from_slice(&TdfEncoder::encode_int("CNFG", 0));
    qdat.extend_from_slice(&TdfEncoder::encode_int("DBPS", 0));
    qdat.extend_from_slice(&TdfEncoder::encode_int("NAHR", 0));
    qdat.extend_from_slice(&TdfEncoder::encode_int("NATT", 0));
    qdat.extend_from_slice(&TdfEncoder::encode_int("UBPS", 0));
    edat.extend_from_slice(&TdfEncoder::encode_struct("QDAT", &qdat));
    edat.extend_from_slice(&TdfEncoder::encode_string("TZ  ", ""));
    edat.extend_from_slice(&TdfEncoder::encode_int("UATT", 0));
    edat.extend_from_slice(&TdfEncoder::encode_int("XPLT", 1));
    edat
}

/// Create an error response for UserSessions component
pub fn create_user_sessions_error_response(error_code: u32) -> Vec<u8> {
    let mut response = Vec::new();

    // CNTX Integer: 0 (context)
    response.extend_from_slice(&TdfEncoder::encode_int("CNTX", 0));

    // ERRC Integer: error code
    response.extend_from_slice(&TdfEncoder::encode_int("ERRC", error_code as i32));

    response
}

/// Handle UserSessions.updateNetworkInfo command (Command=20)
/// Sets the QOS data, latency information, and network address for the current user
/// Returns empty payload (0 bytes) for successful update
pub fn handle_user_sessions_update_network_info(payload: &[u8]) -> BlazeResult<Bytes> {
    use crate::session::{merge_network_snapshot, NetworkSnapshot};

    let mut ips = TdfEncoder::find_all_u32_fields(payload, "IP  ");
    if ips.is_empty() {
        ips = TdfEncoder::scan_all_u32_fields(payload, "IP  ");
    }
    let mut ports = TdfEncoder::find_all_int_fields(payload, "PORT");
    if ports.is_empty() {
        ports = TdfEncoder::scan_all_int_fields(payload, "PORT");
    }
    let bps = TdfEncoder::find_string_field(payload, "BPS ")
        .or_else(|| TdfEncoder::find_string_field(payload, "BPS"))
        .or_else(|| TdfEncoder::scan_first_string_field(payload, "BPS "))
        .or_else(|| TdfEncoder::scan_first_string_field(payload, "BPS"))
        .filter(|s| !s.is_empty());
    let mut n = NetworkSnapshot::default();
    if ips.len() >= 2 {
        n.exip_ip = Some(ips[0]);
        n.inip_ip = Some(ips[1]);
    } else if ips.len() == 1 {
        n.exip_ip = Some(ips[0]);
    }
    if ports.len() >= 2 {
        n.exip_port = Some(ports[0]);
        n.inip_port = Some(ports[1]);
    } else if ports.len() == 1 {
        n.exip_port = Some(ports[0]);
    }
    n.bps = bps;
    merge_network_snapshot(n);
    crate::debug_println!(
        "\x1b[38;2;150;150;255m[Blaze]\x1b[0m handle_user_sessions_update_network_info payload {} (IPs={}, PORTs={}), merged snapshot for extended data echo",
        payload.len(),
        ips.len(),
        ports.len()
    );
    Ok(Bytes::from(Vec::new()))
}

/// Handle UserSessions.updateHardwareFlags command (Command=8)
/// This command updates hardware flags for the current user
pub fn handle_user_sessions_update_hardware_flags(payload: &[u8]) -> BlazeResult<Bytes> {
    crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m handle_user_sessions_update_hardware_flags entered, payload size: {}", payload.len());
    // Extract HWFG value from request and update session state
    if let Some(hwfg_value) = TdfEncoder::find_int_field(payload, "HWFG") {
        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Extracted HWFG value: {}", hwfg_value);
        use crate::session::set_hwfg;
        set_hwfg(hwfg_value as u32);
        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Updated session HWFG to {}", hwfg_value);
    } else {
        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m No HWFG field found in payload");
    }
    
    // Return empty response for successful updateHardwareFlags
    // Empty payload is valid for this command
    crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Returning empty response for updateHardwareFlags");
    Ok(Bytes::from(Vec::new()))
}

/// UserSessions.lookupUser (Command 12) — client expects `USER` identification; empty replies were ~16B and dropped the session.
pub fn handle_user_sessions_lookup_user(_payload: &[u8]) -> BlazeResult<Bytes> {
    use crate::session::get_user_session;
    let session = get_user_session();
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_int("CNTX", 1016290622));
    response.extend_from_slice(&TdfEncoder::encode_int("ERRC", 0));
    let user_inner = encode_user_identification_inner(&session);
    response.extend_from_slice(&TdfEncoder::encode_struct("USER", &user_inner));
    crate::debug_println!(
        "\x1b[38;2;150;150;255m[Blaze]\x1b[0m lookupUser OK (USER {} bytes)",
        user_inner.len()
    );
    Ok(Bytes::from(response))
}

/// Handle UserSessions Command=60 - critical for client flow
pub fn handle_user_sessions_command_60(_payload: &[u8]) -> BlazeResult<Bytes> {
    // Return empty response for Command=60
    // This appears to be a client state update command
    Ok(Bytes::from(Vec::new()))
}

/// UserSessions command 8 — `UserAuthenticated` notification payload (stable field order).
pub fn handle_user_sessions_authenticated(_payload: &[u8]) -> BlazeResult<Bytes> {
    use crate::blaze::handlers::auth_handlers::blaze_session_key;
    use crate::session::get_user_session;
    use std::time::{SystemTime, UNIX_EPOCH};

    let session = get_user_session();
    let session_key = blaze_session_key(session.user_id as i64, session.persona_id as i64);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let mut response = Vec::new();

    response.extend_from_slice(&TdfEncoder::encode_int("1CON", 0));

    let mut aids = Vec::new();
    let mut eaid = Vec::new();
    eaid.extend_from_slice(&TdfEncoder::encode_string("NAME", &session.display_name));
    eaid.extend_from_slice(&TdfEncoder::encode_long("NID ", session.user_id as i64));
    eaid.extend_from_slice(&TdfEncoder::encode_long("PCID", session.persona_id as i64));
    aids.extend_from_slice(&TdfEncoder::encode_struct("EAID", &eaid));

    let mut exid = Vec::new();
    exid.extend_from_slice(&TdfEncoder::encode_int("PSID", session.psid as i32));
    exid.extend_from_slice(&TdfEncoder::encode_int("STID", 0));
    exid.extend_from_slice(&TdfEncoder::encode_string("SWID", ""));
    exid.extend_from_slice(&TdfEncoder::encode_int("XBID", 0));
    aids.extend_from_slice(&TdfEncoder::encode_struct("EXID", &exid));
    aids.extend_from_slice(&TdfEncoder::encode_int("PLAT", 4));
    response.extend_from_slice(&TdfEncoder::encode_struct("AIDS", &aids));

    response.extend_from_slice(&TdfEncoder::encode_int("ALOC", 2053657939));
    response.extend_from_slice(&TdfEncoder::encode_int("ANON", 0));

    let mut apcs = Vec::new();
    apcs.extend_from_slice(&TdfEncoder::encode_string("SDAT", ""));
    response.extend_from_slice(&TdfEncoder::encode_struct("APCS", &apcs));

    response.extend_from_slice(&TdfEncoder::encode_long("BUID", session.persona_id as i64));
    response.extend_from_slice(&TdfEncoder::encode_object_id(
        "CGID",
        30722,
        2,
        LABS_SESSION_OBJECT_ID,
    ));
    response.extend_from_slice(&TdfEncoder::encode_int("CNTY", 21843));
    response.extend_from_slice(&TdfEncoder::encode_string("DSNM", &session.display_name));
    response.extend_from_slice(&TdfEncoder::encode_int("FRST", 0));
    response.extend_from_slice(&TdfEncoder::encode_string("KEY", &session_key));
    response.extend_from_slice(&TdfEncoder::encode_long("LAST", now));
    response.extend_from_slice(&TdfEncoder::encode_long("LLOG", now));
    response.extend_from_slice(&TdfEncoder::encode_string("NASP", "cem_ea_id"));
    response.extend_from_slice(&TdfEncoder::encode_int("PAAI", 0));
    response.extend_from_slice(&TdfEncoder::encode_long("PID", session.persona_id as i64));
    response.extend_from_slice(&TdfEncoder::encode_int("PLAT", 4));
    response.extend_from_slice(&TdfEncoder::encode_long("UID", session.user_id as i64));

    Ok(Bytes::from(response))
}

pub fn handle_user_sessions_added(_payload: &[u8]) -> BlazeResult<Bytes> {
    use crate::session::get_user_session;
    let session = get_user_session();

    let mut response = Vec::new();

    let mut apcs_struct = Vec::new();
    apcs_struct.extend_from_slice(&TdfEncoder::encode_string("SDAT", ""));
    response.extend_from_slice(&TdfEncoder::encode_struct("APCS", &apcs_struct));

    let mut data_struct = Vec::new();

    data_struct.extend_from_slice(&TdfEncoder::encode_int_single_byte("ADDR", 127));

    data_struct.extend_from_slice(&TdfEncoder::encode_string("BPS ", ""));
    data_struct.extend_from_slice(&TdfEncoder::encode_string("CTY ", ""));
    data_struct.extend_from_slice(&TdfEncoder::encode_int("CTYP", 0));

    data_struct.extend_from_slice(&TdfEncoder::encode_string_list("CVAR", &[]));

    data_struct.extend_from_slice(&TdfEncoder::encode_double_list_int_int(
        "DMAP",
        &[2013396993i64],
        &[0i64],
    ));

    data_struct.extend_from_slice(&TdfEncoder::encode_int("HWFG", session.hwfg as i32));
    data_struct.extend_from_slice(&TdfEncoder::encode_string("ISP ", ""));

    let mut qdat_struct = Vec::new();
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("BWHR", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("CNFG", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("DBPS", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("NAHR", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("NATT", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("UBPS", 0));
    data_struct.extend_from_slice(&TdfEncoder::encode_struct("QDAT", &qdat_struct));

    data_struct.extend_from_slice(&TdfEncoder::encode_string("TZ  ", ""));
    data_struct.extend_from_slice(&TdfEncoder::encode_int("UATT", 0));

    let ulst_list = vec![(30722i32, 2i32, LABS_SESSION_OBJECT_ID)];
    data_struct.extend_from_slice(&TdfEncoder::encode_object_id_list("ULST", &ulst_list));

    data_struct.extend_from_slice(&TdfEncoder::encode_int("XPLT", 1));
    response.extend_from_slice(&TdfEncoder::encode_struct("DATA", &data_struct));

    let user_inner = encode_user_identification_inner(&session);
    response.extend_from_slice(&TdfEncoder::encode_struct("USER", &user_inner));

    Ok(Bytes::from(response))
}

/// First `UserSessionExtendedDataUpdate`: top-level `DATA` only (no CNTX/ERRC/SUBS/USID).
/// CVAR = empty long list; ULST third id = `LABS_SESSION_OBJECT_ID`.
pub fn handle_user_session_extended_data_update_first(_payload: &[u8]) -> BlazeResult<Bytes> {
    use crate::session::get_user_session;
    let session = get_user_session();

    let mut data_struct = Vec::new();

    let mut addr_valu = Vec::new();
    let exip_ip = session.network_exip_ip.unwrap_or(1383274225);
    let exip_port = session.network_exip_port.unwrap_or(65535);
    let inip_ip = session.network_inip_ip.unwrap_or(3232267267);
    let inip_port = session.network_inip_port.unwrap_or(65535);
    let mut exip = Vec::new();
    if exip_ip <= i32::MAX as u32 {
        exip.extend_from_slice(&TdfEncoder::encode_int("IP  ", exip_ip as i32));
    } else {
        exip.extend_from_slice(&TdfEncoder::encode_long("IP  ", exip_ip as i64));
    }
    exip.extend_from_slice(&TdfEncoder::encode_int("MACI", 0));
    exip.extend_from_slice(&TdfEncoder::encode_int("PORT", exip_port));
    addr_valu.extend_from_slice(&TdfEncoder::encode_struct("EXIP", &exip));

    let mut inip = Vec::new();
    inip.extend_from_slice(&TdfEncoder::encode_long("IP  ", inip_ip as i64));
    inip.extend_from_slice(&TdfEncoder::encode_int("MACI", 0));
    inip.extend_from_slice(&TdfEncoder::encode_int("PORT", inip_port));
    addr_valu.extend_from_slice(&TdfEncoder::encode_struct("INIP", &inip));

    addr_valu.extend_from_slice(&TdfEncoder::encode_long("MACI", 4103281682i64));
    data_struct.extend_from_slice(&TdfEncoder::encode_int_single_byte("ADDR", 2));
    data_struct.extend_from_slice(&TdfEncoder::encode_struct("VALU", &addr_valu));

    let bps = session
        .network_bps
        .as_deref()
        .unwrap_or("aws-pdx");
    data_struct.extend_from_slice(&TdfEncoder::encode_string("BPS ", bps));
    data_struct.extend_from_slice(&TdfEncoder::encode_string("CTY ", ""));
    data_struct.extend_from_slice(&TdfEncoder::encode_int("CTYP", 0));
    data_struct.extend_from_slice(&TdfEncoder::encode_long_list("CVAR", &[]));

    const DMAP_KEYS: [i64; 12] = [
        262155,
        262164,
        262165,
        262166,
        262167,
        262168,
        262174,
        262175,
        262176,
        262177,
        262178,
        2013396993,
    ];
    let dmap_vals = [0i64; 12];
    data_struct.extend_from_slice(&TdfEncoder::encode_double_list_int_int("DMAP", &DMAP_KEYS, &dmap_vals));

    data_struct.extend_from_slice(&TdfEncoder::encode_int("HWFG", session.hwfg as i32));
    data_struct.extend_from_slice(&TdfEncoder::encode_string("ISP ", ""));

    const PSM_REGIONS: [&str; 15] = [
        "aws-bah",
        "aws-brz",
        "aws-cmh",
        "aws-cpt",
        "aws-dub",
        "aws-fra",
        "aws-hkg",
        "aws-iad",
        "aws-icn",
        "aws-lhr",
        "aws-nrt",
        "aws-pdx",
        "aws-sin",
        "aws-sjc",
        "aws-syd",
    ];
    let psm_vals = [268374015i64; 15];
    data_struct.extend_from_slice(&TdfEncoder::encode_double_list_string_int(
        "PSM ",
        &PSM_REGIONS,
        &psm_vals,
    ));

    let mut qdat_struct = Vec::new();
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("BWHR", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("CNFG", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("DBPS", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("NAHR", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("NATT", 5));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("UBPS", 0));
    data_struct.extend_from_slice(&TdfEncoder::encode_struct("QDAT", &qdat_struct));

    data_struct.extend_from_slice(&TdfEncoder::encode_string("TZ  ", ""));
    data_struct.extend_from_slice(&TdfEncoder::encode_int("UATT", 0));

    let ulst_list = vec![(30722, 2, LABS_SESSION_OBJECT_ID)];
    data_struct.extend_from_slice(&TdfEncoder::encode_object_id_list("ULST", &ulst_list));

    data_struct.extend_from_slice(&TdfEncoder::encode_int("XPLT", 1));

    Ok(TdfEncoder::encode_struct("DATA", &data_struct))
}

/// Handle SECOND UserSessionExtendedDataUpdate notification
/// ADDR: type-6 byte 2 + VALU struct (same shape as the first extended update)
pub fn handle_user_session_extended_data_update_second(_payload: &[u8]) -> BlazeResult<Bytes> {
    use crate::session::get_user_session;
    let session = get_user_session();
    
    let mut response = Vec::new();

    // CNTX Integer: 1016290622
    response.extend_from_slice(&TdfEncoder::encode_int("CNTX", 1016290622));

    // ERRC Integer: 0
    response.extend_from_slice(&TdfEncoder::encode_int("ERRC", 0));

    // DATA Struct
    let mut data_struct = Vec::new();

    let mut addr_valu = Vec::new();
    let exip_ip = session.network_exip_ip.unwrap_or(2179085813u32);
    let exip_port = session.network_exip_port.unwrap_or(51741);
    let inip_ip = session.network_inip_ip.unwrap_or(168430082);
    let inip_port = session.network_inip_port.unwrap_or(3659);
    let mut exip = Vec::new();
    exip.extend_from_slice(&TdfEncoder::encode_long("IP  ", exip_ip as i64));
    exip.extend_from_slice(&TdfEncoder::encode_int("MACI", 0));
    exip.extend_from_slice(&TdfEncoder::encode_int("PORT", exip_port));
    addr_valu.extend_from_slice(&TdfEncoder::encode_struct("EXIP", &exip));

    let mut inip = Vec::new();
    if inip_ip <= i32::MAX as u32 {
        inip.extend_from_slice(&TdfEncoder::encode_int("IP  ", inip_ip as i32));
    } else {
        inip.extend_from_slice(&TdfEncoder::encode_long("IP  ", inip_ip as i64));
    }
    inip.extend_from_slice(&TdfEncoder::encode_int("MACI", 0));
    inip.extend_from_slice(&TdfEncoder::encode_int("PORT", inip_port));
    addr_valu.extend_from_slice(&TdfEncoder::encode_struct("INIP", &inip));

    addr_valu.extend_from_slice(&TdfEncoder::encode_int("MACI", 1383089204));
    data_struct.extend_from_slice(&TdfEncoder::encode_int_single_byte("ADDR", 2));
    data_struct.extend_from_slice(&TdfEncoder::encode_struct("VALU", &addr_valu));

    let bps = session.network_bps.as_deref().unwrap_or("aws-hkg");
    data_struct.extend_from_slice(&TdfEncoder::encode_string("BPS", bps));

    // CTY String: ""
    data_struct.extend_from_slice(&TdfEncoder::encode_string("CTY", ""));

    // CTYP Integer: 0
    data_struct.extend_from_slice(&TdfEncoder::encode_int("CTYP", 0));

    // CVAR IntList: []
    data_struct.extend_from_slice(&TdfEncoder::encode_list("CVAR", &[]));

    // DMAP Map<Integer, Integer> - All 0 for second notification
    let mut dmap_map = std::collections::HashMap::new();
    dmap_map.insert(262155, 0);
    dmap_map.insert(262164, 0);
    dmap_map.insert(262165, 0);
    dmap_map.insert(262166, 0);
    dmap_map.insert(262167, 0);
    dmap_map.insert(262174, 0);
    dmap_map.insert(262175, 0);
    dmap_map.insert(262176, 0);
    dmap_map.insert(262177, 0);
    dmap_map.insert(2013396993, 0);
    data_struct.extend_from_slice(&TdfEncoder::encode_int_map("DMAP", &dmap_map));

    data_struct.extend_from_slice(&TdfEncoder::encode_int("HWFG", session.hwfg as i32));

    // ISP String: ""
    data_struct.extend_from_slice(&TdfEncoder::encode_string("ISP", ""));

    // PSM Map<String, Integer> - Values for second notification
    let mut psm_map = std::collections::HashMap::new();
    psm_map.insert("aws-arn".to_string(), 262);
    psm_map.insert("aws-bah".to_string(), 259);
    psm_map.insert("aws-bom".to_string(), 206);
    psm_map.insert("aws-brz".to_string(), 384);
    psm_map.insert("aws-cmh".to_string(), 259);
    psm_map.insert("aws-cpt".to_string(), 376);
    psm_map.insert("aws-dub".to_string(), 262);
    psm_map.insert("aws-fra".to_string(), 243);
    psm_map.insert("aws-hkg".to_string(), 61);
    psm_map.insert("aws-iad".to_string(), 274);
    psm_map.insert("aws-icn".to_string(), 146);
    psm_map.insert("aws-lhr".to_string(), 252);
    psm_map.insert("aws-nrt".to_string(), 108);
    psm_map.insert("aws-pdx".to_string(), 240);
    psm_map.insert("aws-sin".to_string(), 102);
    psm_map.insert("aws-sjc".to_string(), 224);
    psm_map.insert("aws-syd".to_string(), 204);
    data_struct.extend_from_slice(&TdfEncoder::encode_string_int_map("PSM", &psm_map));

    // QDAT Struct
    let mut qdat_struct = Vec::new();
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("BWHR", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("CNFG", 2));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("DBPS", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("NAHR", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("NATT", 1));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("UBPS", 0));
    data_struct.extend_from_slice(&TdfEncoder::encode_struct("QDAT", &qdat_struct));

    // TZ String: ""
    data_struct.extend_from_slice(&TdfEncoder::encode_string("TZ", ""));

    // UATT Integer: 0
    data_struct.extend_from_slice(&TdfEncoder::encode_int("UATT", 0));

    let ulst_list = vec![(30722, 2, LABS_SESSION_OBJECT_ID)];
    data_struct.extend_from_slice(&TdfEncoder::encode_object_id_list("ULST", &ulst_list));

    // XPLT Integer: 1
    data_struct.extend_from_slice(&TdfEncoder::encode_int("XPLT", 1));

    response.extend_from_slice(&TdfEncoder::encode_struct("DATA", &data_struct));

    // SUBS Integer: 1
    response.extend_from_slice(&TdfEncoder::encode_int("SUBS", 1));

    // USID Long: Use persona_id from session (not user_id)
    response.extend_from_slice(&TdfEncoder::encode_long("USID", session.persona_id as i64));

    Ok(Bytes::from(response))
}

/// Handle THIRD UserSessionExtendedDataUpdate notification (426 bytes)
/// Hex payload starts with: 921d2103 86493206 02da1b35 03978a70 (same start but different PSM)
pub fn handle_user_session_extended_data_update_third(_payload: &[u8]) -> BlazeResult<Bytes> {
    use crate::session::get_user_session;
    let session = get_user_session();
    
    // Third UserSessionExtendedDataUpdate - 426 bytes with different PSM values
    let mut response = Vec::new();

    // CNTX Integer: 1016290622
    response.extend_from_slice(&TdfEncoder::encode_int("CNTX", 1016290622));

    // ERRC Integer: 0
    response.extend_from_slice(&TdfEncoder::encode_int("ERRC", 0));

    // DATA Struct
    let mut data_struct = Vec::new();

    data_struct.extend_from_slice(&TdfEncoder::encode_int_single_byte("ADDR", 127));

    // BPS String: "aws-syd" (different from first two)
    data_struct.extend_from_slice(&TdfEncoder::encode_string("BPS", "aws-syd"));

    // CTY String: ""
    data_struct.extend_from_slice(&TdfEncoder::encode_string("CTY", ""));

    // CTYP Integer: 0
    data_struct.extend_from_slice(&TdfEncoder::encode_int("CTYP", 0));

    // CVAR IntList: []
    data_struct.extend_from_slice(&TdfEncoder::encode_list("CVAR", &[]));

    // DMAP Map<Integer, Integer> - Network mapping
    let mut dmap_map = std::collections::HashMap::new();
    dmap_map.insert(262155, 1);
    dmap_map.insert(262164, 550);
    dmap_map.insert(262165, 550);
    dmap_map.insert(262166, 550);
    dmap_map.insert(262167, 550);
    dmap_map.insert(262174, 1);
    dmap_map.insert(262175, 1);
    dmap_map.insert(262176, 1);
    dmap_map.insert(262177, 1);
    dmap_map.insert(2013396993, 0);
    data_struct.extend_from_slice(&TdfEncoder::encode_int_map("DMAP", &dmap_map));

    // HWFG Integer: 1
    data_struct.extend_from_slice(&TdfEncoder::encode_int("HWFG", 1));

    // ISP String: ""
    data_struct.extend_from_slice(&TdfEncoder::encode_string("ISP", ""));

    // PSM Map<String, Integer> - Values for third notification
    let mut psm_map = std::collections::HashMap::new();
    psm_map.insert("aws-bah".to_string(), 0x02b2);
    psm_map.insert("aws-brz".to_string(), 0x0586);
    psm_map.insert("aws-cmh".to_string(), 0x038f);
    psm_map.insert("aws-cpt".to_string(), 0x069e);
    psm_map.insert("aws-dub".to_string(), 0x0480);
    psm_map.insert("aws-fra".to_string(), 0x03b2);
    psm_map.insert("aws-hkg".to_string(), 0x0282);
    psm_map.insert("aws-iad".to_string(), 0x038b);
    psm_map.insert("aws-icn".to_string(), 0x02a5);
    psm_map.insert("aws-lhr".to_string(), 0x03b8);
    psm_map.insert("aws-nrt".to_string(), 0x02ad);
    psm_map.insert("aws-pdx".to_string(), 0x02a1);
    psm_map.insert("aws-sin".to_string(), 0x019e);
    psm_map.insert("aws-sjc".to_string(), 0x0291);
    psm_map.insert("aws-syd".to_string(), 0x0002);
    data_struct.extend_from_slice(&TdfEncoder::encode_string_int_map("PSM", &psm_map));

    // QDAT Struct
    let mut qdat_struct = Vec::new();
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("BWHR", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("CNFG", 2));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("DBPS", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("NAHR", 0));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("NATT", 1));
    qdat_struct.extend_from_slice(&TdfEncoder::encode_int("UBPS", 0));
    data_struct.extend_from_slice(&TdfEncoder::encode_struct("QDAT", &qdat_struct));

    // TZ String: ""
    data_struct.extend_from_slice(&TdfEncoder::encode_string("TZ", ""));

    // UATT Integer: 0
    data_struct.extend_from_slice(&TdfEncoder::encode_int("UATT", 0));

    let mut ulst_list = Vec::new();
    ulst_list.push((30722, 2, LABS_SESSION_OBJECT_ID));
    ulst_list.push((4, 2, 52136960662i64)); // GameManager, destroyGame, GameId
    ulst_list.push((4, 1, 50133810230i64)); // GameManager, createGame, GameId
    data_struct.extend_from_slice(&TdfEncoder::encode_object_id_list("ULST", &ulst_list));

    // XPLT Integer: 1
    data_struct.extend_from_slice(&TdfEncoder::encode_int("XPLT", 1));

    response.extend_from_slice(&TdfEncoder::encode_struct("DATA", &data_struct));

    // SUBS Integer: 1
    response.extend_from_slice(&TdfEncoder::encode_int("SUBS", 1));

    // USID Long: Use persona_id from session
    response.extend_from_slice(&TdfEncoder::encode_long("USID", session.persona_id as i64));

    Ok(Bytes::from(response))
}

/// Handle UserSessionExtendedDataUpdate notification (legacy - now redirects to first)
pub fn handle_user_session_extended_data_update(_payload: &[u8]) -> BlazeResult<Bytes> {
    // Legacy handler - redirect to first handler
    handle_user_session_extended_data_update_first(_payload)
}

pub fn handle_user_sessions_command_8(_payload: &[u8]) -> BlazeResult<Bytes> {
    let response = Vec::new();
    Ok(Bytes::from(response))
}

pub fn handle_user_sessions_set_user_cross_platform_opt_in(_payload: &[u8]) -> BlazeResult<Bytes> {
    let response = Vec::new();
    Ok(Bytes::from(response))
}

pub fn handle_user_sessions_lookup_users(_payload: &[u8]) -> BlazeResult<Bytes> {
    use crate::session::get_user_session;
    let session = get_user_session();
    let mut response = Vec::new();

    let mut ulst_entry = Vec::new();
    let edat = encode_lookup_users_edat_placeholder();
    ulst_entry.extend_from_slice(&TdfEncoder::encode_struct("EDAT", &edat));
    ulst_entry.extend_from_slice(&TdfEncoder::encode_int("FLGS", 0));
    let user_inner = encode_user_identification_inner(&session);
    ulst_entry.extend_from_slice(&TdfEncoder::encode_struct("USER", &user_inner));
    response.extend_from_slice(&encode_struct_list("ULST", &[ulst_entry]));

    crate::debug_println!(
        "\x1b[38;2;150;150;255m[Blaze]\x1b[0m lookupUsers reply built (ULST with USER {}, EDAT {})",
        user_inner.len(),
        edat.len()
    );
    Ok(Bytes::from(response))
}

/// Handle UserSessions Command=1 - echo response
pub fn handle_user_sessions_command_1(payload: &[u8]) -> BlazeResult<Bytes> {
    // If payload is empty, return empty response
    if payload.is_empty() {
        return Ok(Bytes::from(Vec::new()));
    }
    Ok(Bytes::from(payload.to_vec()))
}


