use crate::blaze::tdf::TdfEncoder;
use crate::common::error::BlazeResult;
use crate::session::session_module::{
    get_build_profile, get_user_session, hint_build_profile_from_text, set_build_profile, BuildProfile,
};
use bytes::Bytes;

pub fn handle_util_ping(_payload: &[u8]) -> BlazeResult<Bytes> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut response = Vec::new();
    let stim = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i32;
    response.extend_from_slice(&TdfEncoder::encode_int("STIM", stim));
    Ok(Bytes::from(response))
}

pub fn handle_util_preauth(payload: &[u8]) -> BlazeResult<Bytes> {
    let profile_hint = if let Ok(s) = std::str::from_utf8(payload) {
        hint_build_profile_from_text(s)
    } else {
        hint_build_profile_from_text(&String::from_utf8_lossy(payload))
    };

    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_string("ASRC", "324320"));
    let cids_list = vec![
        30728, 1, 30729, 25, 30730, 555, 30731, 4, 30732, 9, 10, 63490, 403, 13, 15, 30720, 30721,
        30722, 30723, 30724, 30725, 30726, 30727,
    ];
    response.extend_from_slice(&TdfEncoder::encode_list("CIDS", &cids_list));
    response.extend_from_slice(&TdfEncoder::encode_string("CLID", "GLACIER_LABS_COM_BLZ_SERVER"));

    let mut conf_struct = Vec::new();
    let mut conf_map = indexmap::IndexMap::new();
    conf_map.insert("associationListSkipInitialSet".to_string(), "1".to_string());
    conf_map.insert("autoReconnectEnabled".to_string(), "0".to_string());
    conf_map.insert("cachedUserRefreshInterval".to_string(), "1s".to_string());
    conf_map.insert("clientUserMetricsUpdateRate".to_string(), "60000".to_string());
    conf_map.insert("connIdleTimeout".to_string(), "90s".to_string());
    conf_map.insert("defaultRequestTimeout".to_string(), "20s".to_string());
    conf_map.insert("enableLoginQueueEstimate".to_string(), "false".to_string());
    conf_map.insert("loginRateSeconds".to_string(), "200".to_string());
    conf_map.insert("maxReconnectAttempts".to_string(), "30".to_string());
    conf_map.insert("nonResumableTimeoutScale".to_string(), "2.0".to_string());
    conf_map.insert("nucleusConnect".to_string(), "https://accounts.ea.com".to_string());
    conf_map.insert("nucleusConnectTrusted".to_string(), "https://accounts2s.ea.com".to_string());
    conf_map.insert("nucleusPortal".to_string(), "https://signin.ea.com".to_string());
    conf_map.insert("nucleusProxy".to_string(), "https://gateway.ea.com".to_string());
    conf_map.insert("pingPeriod".to_string(), "30s".to_string());
    conf_map.insert("userManagerMaxCachedUsers".to_string(), "0".to_string());
    conf_map.insert("voipHeadsetUpdateRate".to_string(), "1000".to_string());
    conf_map.insert("voipSTTKey".to_string(), "mpfZMofmoDkGPCu3tGJ52lpzC4pxTFWJ3eT9EQLYJp6P".to_string());
    conf_map.insert("voipSTTProfile".to_string(), "66307".to_string());
    conf_map.insert("voipSTTUrl".to_string(), "wss://api.us-east.speech-to-text.watson.cloud.ibm.com/instances/3f4f32cd-1f90-46d6-99a5-aeb15bd6b179/v1/recognize".to_string());
    conf_map.insert("voipTTSKey".to_string(), "x0PH2NdiMzbAS5BkZOqD-8P4KpSgjw1PTsCIfSeG7dDl".to_string());
    conf_map.insert("voipTTSProvider".to_string(), "1".to_string());
    conf_map.insert("voipTTSUrl".to_string(), "https://api.us-east.text-to-speech.watson.cloud.ibm.com/instances/119632b1-80bf-4ca5-97f2-dda53c99ef35/v1/synthesize".to_string());
    conf_struct.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("CONF", &conf_map));
    response.extend_from_slice(&TdfEncoder::encode_struct("CONF", &conf_struct));
    response.extend_from_slice(&TdfEncoder::encode_string("ESRC", "324320"));
    response.extend_from_slice(&TdfEncoder::encode_list("HPLT", &[4, 22, 23, 24]));

    let inst_value = if profile_hint == BuildProfile::LabsAlpha { "bf-2026-pc-labs" } else { "bf-community-pc-labs" };
    response.extend_from_slice(&TdfEncoder::encode_string("INST", inst_value));
    response.extend_from_slice(&TdfEncoder::encode_long("LQTK", 4805485006359740375));
    response.extend_from_slice(&TdfEncoder::encode_long("MAID", 1383089204));
    response.extend_from_slice(&TdfEncoder::encode_int("MINR", 0));
    response.extend_from_slice(&TdfEncoder::encode_string("NASP", "cem_ea_id"));
    response.extend_from_slice(&TdfEncoder::encode_string("PILD", ""));
    response.extend_from_slice(&TdfEncoder::encode_string("PLAT", "pc"));

    let qos_ports = crate::common::game::current_service_ports();
    let mut qoss_struct = Vec::new();
    qoss_struct.extend_from_slice(&TdfEncoder::encode_int("CQFR", 300000000));
    qoss_struct.extend_from_slice(&TdfEncoder::encode_int("CQRR", 0));
    let mut ltps_map = indexmap::IndexMap::new();
    let region_ip_pairs = vec![
        ("aws-bah", "16.24.71.144"), ("aws-brz", "15.228.242.209"), ("aws-cmh", "18.226.28.192"),
        ("aws-cpt", "13.247.231.45"), ("aws-dub", "3.253.21.5"), ("aws-fra", "18.193.108.49"),
        ("aws-hkg", "54.46.69.91"), ("aws-iad", "44.202.116.134"), ("aws-icn", "43.200.175.136"),
        ("aws-lhr", "52.56.243.208"), ("aws-nrt", "13.113.192.27"), ("aws-pdx", "34.217.116.104"),
        ("aws-sin", "13.251.110.168"), ("aws-sjc", "3.101.56.212"), ("aws-syd", "3.26.77.15"),
    ];
    for (region_code, ip_addr) in region_ip_pairs {
        let mut region_struct = Vec::new();
        region_struct.extend_from_slice(&TdfEncoder::encode_string("PSA ", ip_addr));
        region_struct.extend_from_slice(&TdfEncoder::encode_int("PSP ", qos_ports.qos_data as i32));
        ltps_map.insert(region_code.to_string(), region_struct);
    }
    qoss_struct.extend_from_slice(&TdfEncoder::encode_string_struct_map_ordered("LTPS", &ltps_map));
    let mut qcnf_struct = Vec::new();
    qcnf_struct.extend_from_slice(&TdfEncoder::encode_int("DPSP", qos_ports.qos_data as i32));
    qcnf_struct.extend_from_slice(&TdfEncoder::encode_string("QCA ", "qoscoordinator.gameservices.ea.com"));
    qcnf_struct.extend_from_slice(&TdfEncoder::encode_int("QCP ", qos_ports.qos_alt as i32));
    let qpr_value = if profile_hint == BuildProfile::LabsAlpha { "bf-2026-common-labs" } else { "bf-community-common-labs" };
    qcnf_struct.extend_from_slice(&TdfEncoder::encode_string("QPR ", qpr_value));
    qoss_struct.extend_from_slice(&TdfEncoder::encode_struct("QCNF", &qcnf_struct));
    qoss_struct.extend_from_slice(&TdfEncoder::encode_int("SQRR", 15000000));
    qoss_struct.extend_from_slice(&TdfEncoder::encode_int("VERS", 1));
    response.extend_from_slice(&TdfEncoder::encode_struct("QOSS", &qoss_struct));
    response.extend_from_slice(&TdfEncoder::encode_string("RELT", "prod"));
    response.extend_from_slice(&TdfEncoder::encode_string("RSRC", "324320"));
    response.extend_from_slice(&TdfEncoder::encode_string("SVER", "Blaze 18.3.0 (CL# 2087509)"));

    if profile_hint != BuildProfile::Unknown {
        set_build_profile(profile_hint, "blaze.preAuth");
    }
    Ok(Bytes::from(response))
}

pub fn handle_util_fetch_client_config(payload: &[u8]) -> BlazeResult<Bytes> {
    let mut response = Vec::new();
    let cfid = TdfEncoder::find_string_field(payload, "CFID").unwrap_or_default();
    let unknown_ob = get_build_profile() == BuildProfile::OpenBeta;
    let conf_map = crate::client::labs::fetch_client_config_conf_map(cfid.as_str(), unknown_ob);
    let tenancy = conf_map.get("clientGrpcTenancy").map(|s| s.as_str()).unwrap_or("");
    let grpc_url = conf_map.get("clientGrpcUrl").map(|s| s.as_str()).unwrap_or("");
    crate::session::session_module::record_last_fetch_client_config(&cfid, tenancy, grpc_url);
    response.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("CONF", &conf_map));
    Ok(Bytes::from(response))
}

pub fn handle_util_post_auth(_payload: &[u8]) -> BlazeResult<Bytes> {
    let mut response = Vec::new();
    let mut tele_struct = Vec::new();
    tele_struct.extend_from_slice(&TdfEncoder::encode_string("ADRS", "https://river.data.ea.com"));
    tele_struct.extend_from_slice(&TdfEncoder::encode_int("ANON", 0));
    tele_struct.extend_from_slice(&TdfEncoder::encode_string("BKEY", ""));
    tele_struct.extend_from_slice(&TdfEncoder::encode_int("CTRY", 17230));
    tele_struct.extend_from_slice(&TdfEncoder::encode_string("DISA", "AD,AF,AG,AI,AL,AM,AN,AO,AQ,AR,AS,AW,AX,AZ,BA,BB,BD,BF,BH,BI,BJ,BM,BN,BO,BR,BS,BT,BV,BW,BY,BZ,CC,CD,CF,CG,CI,CK,CL,CM,CN,CO,CR,CU,CV,CX,DJ,DM,DO,DZ,EC,EG,EH,ER,ET,FJ,FK,FM,FO,GA,GD,GE,GF,GG,GH,GI,GL,GM,GN,GP,GQ,GS,GT,GU,GW,GY,HM,HN,HT,ID,IL,IM,IN,IO,IQ,IR,IS,JE,JM,JO,KE,KG,KH,KI,KM,KN,KP,KR,KW,KY,KZ,LA,LB,LC,LI,LK,LR,LS,LY,MA,MC,MD,ME,MG,MH,ML,MM,MN,MO,MP,MQ,MR,MS,MU,MV,MW,MY,MZ,NA,NC,NE,NF,NG,NI,NP,NR,NU,OM,PA,PE,PF,PG,PH,PK,PM,PN,PS,PW,PY,QA,RE,RS,RW,SA,SB,SC,SD,SG,SH,SJ,SL,SM,SN,SO,SR,ST,SV,SY,SZ,TC,TD,TF,TG,TH,TJ,TK,TL,TM,TN,TO,TT,TV,TZ,UA,UG,UM,UY,UZ,VA,VC,VE,VG,VN,VU,WF,WS,YE,YT,ZM,ZW,ZZ"));
    tele_struct.extend_from_slice(&TdfEncoder::encode_int("ECCT", 0));
    tele_struct.extend_from_slice(&TdfEncoder::encode_int("EDCT", 0));
    tele_struct.extend_from_slice(&TdfEncoder::encode_string("FILT", "-GAME/COMM/EXPD"));
    tele_struct.extend_from_slice(&TdfEncoder::encode_int("LOC", 2053653326));
    tele_struct.extend_from_slice(&TdfEncoder::encode_int("MINR", 0));
    tele_struct.extend_from_slice(&TdfEncoder::encode_string("NOOK", "US,CA,MX"));
    tele_struct.extend_from_slice(&TdfEncoder::encode_string("PENV", "prod"));
    tele_struct.extend_from_slice(&TdfEncoder::encode_int("PORT", 443));
    tele_struct.extend_from_slice(&TdfEncoder::encode_string("PURL", "https://pin-river.data.ea.com"));
    tele_struct.extend_from_slice(&TdfEncoder::encode_int("SDLY", 15000));
    tele_struct.extend_from_slice(&TdfEncoder::encode_string("SESS", "ZPzJoJThDrE"));
    tele_struct.extend_from_slice(&TdfEncoder::encode_string("SKEY", "123"));
    tele_struct.extend_from_slice(&TdfEncoder::encode_int("SPCT", 75));
    tele_struct.extend_from_slice(&TdfEncoder::encode_string("STIM", "Default"));
    tele_struct.extend_from_slice(&TdfEncoder::encode_string("SVNM", "telemetry-3-common"));
    response.extend_from_slice(&TdfEncoder::encode_struct("TELE", &tele_struct));
    let session = get_user_session();
    let uid = session.user_id;
    let mut tick_struct = Vec::new();
    tick_struct.extend_from_slice(&TdfEncoder::encode_string("ADRS", "10.10.78.150"));
    tick_struct.extend_from_slice(&TdfEncoder::encode_int("PORT", 8999));
    let tick_skey = format!("{uid},10.10.78.150:8999,bf-community-common-labs,10,50,50,50,50,0,0");
    tick_struct.extend_from_slice(&TdfEncoder::encode_string("SKEY", &tick_skey));
    response.extend_from_slice(&TdfEncoder::encode_struct("TICK", &tick_struct));
    let mut urop_struct = Vec::new();
    urop_struct.extend_from_slice(&TdfEncoder::encode_int("TMOP", 1));
    urop_struct.extend_from_slice(&TdfEncoder::encode_long("UID", uid as i64));
    response.extend_from_slice(&TdfEncoder::encode_struct("UROP", &urop_struct));
    Ok(Bytes::from(response))
}

pub fn handle_util_set_client_state(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_util_get_telemetry_server(_payload: &[u8]) -> BlazeResult<Bytes> {
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_string("ADRS", "https://river.data.ea.com"));
    response.extend_from_slice(&TdfEncoder::encode_int("ANON", 0));
    response.extend_from_slice(&TdfEncoder::encode_binary("BKEY", &[]));
    response.extend_from_slice(&TdfEncoder::encode_int("CTRY", 17230));
    response.extend_from_slice(&TdfEncoder::encode_string("DISA", "AD,AF,AG,AI,AL,AM,AN,AO,AQ,AR,AS,AW,AX,AZ,BA,BB,BD,BF,BH,BI,BJ,BM,BN,BO,BR,BS,BT,BV,BW,BY,BZ,CC,CD,CF,CG,CI,CK,CL,CM,CN,CO,CR,CU,CV,CX,DJ,DM,DO,DZ,EC,EG,EH,ER,ET,FJ,FK,FM,FO,GA,GD,GE,GF,GG,GH,GI,GL,GM,GN,GP,GQ,GS,GT,GU,GW,GY,HM,HN,HT,ID,IL,IM,IN,IO,IQ,IR,IS,JE,JM,JO,KE,KG,KH,KI,KM,KN,KP,KR,KW,KY,KZ,LA,LB,LC,LI,LK,LR,LS,LY,MA,MC,MD,ME,MG,MH,ML,MM,MN,MO,MP,MQ,MR,MS,MU,MV,MW,MY,MZ,NA,NC,NE,NF,NG,NI,NP,NR,NU,OM,PA,PE,PF,PG,PH,PK,PM,PN,PS,PW,PY,QA,RE,RS,RW,SA,SB,SC,SD,SG,SH,SJ,SL,SM,SN,SO,SR,ST,SV,SY,SZ,TC,TD,TF,TG,TH,TJ,TK,TL,TM,TN,TO,TT,TV,TZ,UA,UG,UM,UY,UZ,VA,VC,VE,VG,VN,VU,WF,WS,YE,YT,ZM,ZW,ZZ"));
    response.extend_from_slice(&TdfEncoder::encode_int("ECCT", 0));
    response.extend_from_slice(&TdfEncoder::encode_int("EDCT", 0));
    response.extend_from_slice(&TdfEncoder::encode_string("FILT", "-GAME/COMM/EXPD"));
    response.extend_from_slice(&TdfEncoder::encode_int("LOC ", 2053653326));
    response.extend_from_slice(&TdfEncoder::encode_int("MINR", 0));
    response.extend_from_slice(&TdfEncoder::encode_string("NOOK", "US,CA,MX"));
    response.extend_from_slice(&TdfEncoder::encode_string("PENV", "prod"));
    response.extend_from_slice(&TdfEncoder::encode_int("PORT", 443));
    response.extend_from_slice(&TdfEncoder::encode_string("PURL", "https://pin-river.data.ea.com"));
    response.extend_from_slice(&TdfEncoder::encode_int("SDLY", 15000));
    response.extend_from_slice(&TdfEncoder::encode_string("SESS", "bPzJoJThDrE"));
    response.extend_from_slice(&TdfEncoder::encode_string("SKEY", "1"));
    response.extend_from_slice(&TdfEncoder::encode_int("SPCT", 75));
    response.extend_from_slice(&TdfEncoder::encode_string("STIM", "Default"));
    response.extend_from_slice(&TdfEncoder::encode_string("SVNM", "telemetry-3-common"));
    Ok(Bytes::from(response))
}

pub fn handle_util_set_client_state_28(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}
