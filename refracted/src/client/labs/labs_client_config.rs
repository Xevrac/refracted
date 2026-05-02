use indexmap::IndexMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CfidConfigKind {
    IdentityParams,
    LabsLike,
    Event,
    Ch1ReleaseF2p,
    GlacierM1p,
    Ch1ReleaseMp,
    Unknown,
}

fn classify_cfid(cfid: &str) -> CfidConfigKind {
    match cfid {
        "IdentityParams" => CfidConfigKind::IdentityParams,
        "ch1-release-mp11" | "labs" | "glacier-mp" => CfidConfigKind::LabsLike,
        "event" => CfidConfigKind::Event,
        "ch1-release-f2p" => CfidConfigKind::Ch1ReleaseF2p,
        "glacier-m1p" => CfidConfigKind::GlacierM1p,
        "ch1-release-mp" => CfidConfigKind::Ch1ReleaseMp,
        _ => CfidConfigKind::Unknown,
    }
}

fn conf_identity_params_labs() -> IndexMap<String, String> {
    let mut m = IndexMap::new();
    m.insert("display".into(), "console2/welcome".into());
    m.insert("redirect_uri".into(), "http://127.0.0.1/success".into());
    m
}

const FETCH_CLIENT_CONFIG_KEYS_20: [&str; 20] = [
    "clientBBPrefixUrl",
    "clientEventBridgeUrl",
    "clientGrpcTenancy",
    "clientGrpcUrl",
    "clientMicaBaseUrl",
    "eadpFriendsEndpoint",
    "eadpSocialPrivacyEndpoint",
    "gatewayClientId",
    "gatewayConnectionCount",
    "gatewaySPStatsCategory",
    "gatewayStatsCategory",
    "gatewayUnaryRequestTimeoutSeconds",
    "receiptsEadpFeed",
    "reportMPStatsRetryCount",
    "serverEventBridgeUrl",
    "serverGrpcMtlsUrl",
    "serverGrpcTenancy",
    "serverGrpcUrl",
    "titleNucleusRedirectUrl",
    "xboxGamePassProductId",
];

fn conf_from_keys_and_values_20(values: [&str; 20]) -> IndexMap<String, String> {
    let mut m = IndexMap::new();
    for (k, v) in FETCH_CLIENT_CONFIG_KEYS_20.iter().zip(values.iter()) {
        m.insert((*k).to_string(), (*v).to_string());
    }
    m
}

fn conf_ch1_release_mp11_labs() -> IndexMap<String, String> {
    conf_from_keys_and_values_20([
        "https://eaassets-a.akamaihd.net/battlelog/battlebinary",
        "https://bflabs-prod-eventbridge.ops.dice.se/",
        "prod_labsprod-prod_labsprod-santiago-common",
        "https://bflabs-prod-gt-cgw.ops.dice.se/",
        "undefined",
        "",
        "",
        "GLACIER_LBGW_BK_OL_SERVER",
        "6",
        "glacier_sp",
        "glacier_mp",
        "90",
        "dice.glacier.receipts.prod_labsprod.v1",
        "3",
        "https://bflabs-prod-eventbridge.ops.dice.se/",
        "https://bflabs-prod-gsgw-mtls.ops.dice.se/",
        "prod_labsprod-prod_labsprod-santiago-common",
        "https://bflabs-prod-gsgw.ops.dice.se/",
        "http://127.0.0.1/success",
        "CFQ7TTC0K5DJ",
    ])
}

fn conf_open_beta() -> IndexMap<String, String> {
    conf_from_keys_and_values_20([
        "https://eaassets-a.akamaihd.net/battlelog/battlebinary",
        "https://eventprod-eventbridge.ops.dice.se/",
        "prod_alphaprod-prod_alphaprod-santiago-common",
        "https://eventprod-mp-cgw.ops.dice.se/",
        "https://prod-mica.ops.dice.se/",
        "",
        "",
        "GLACIER_EVGW_BK_OL_SERVER",
        "20",
        "glacier_sp",
        "glacier_mp",
        "90",
        "dice.glacier.receipts.prod_alphaprod.v1",
        "3",
        "https://eventprod-eventbridge.ops.dice.se/",
        "https://eventprod-gsgw-mtls.ops.dice.se/",
        "prod_alphaprod-prod_alphaprod-santiago-common",
        "https://eventprod-gsgw.ops.dice.se/",
        "http://127.0.0.1/success",
        "CFQ7TTC0K5DJ",
    ])
}

fn conf_ch1_release_f2p_labs() -> IndexMap<String, String> {
    conf_from_keys_and_values_20([
        "https://eaassets-a.akamaihd.net/battlelog/battlebinary",
        "https://santiago-prod-eventbridge.ops.dice.se/",
        "prod_default-prod_default-santiago-common",
        "https://santiago-prod-mp-cgw.ops.dice.se/",
        "https://portal.battlefield.com/",
        "",
        "",
        "GLACIER_GW_BK_OL_SERVER",
        "6",
        "glacier_sp",
        "glacier_mp",
        "90",
        "dice.glacier.receipts.prod_default.v1",
        "3",
        "https://santiago-prod-eventbridge.ops.dice.se/",
        "https://santiago-prod-gsgw-mtls.ops.dice.se/",
        "prod_default-prod_default-santiago-common",
        "https://notused.trama/",
        "http://127.0.0.1/success",
        "CFQ7TTC0K5DJ",
    ])
}

fn conf_glacier_m1p_labs() -> IndexMap<String, String> {
    conf_from_keys_and_values_20([
        "https://eaassets-a.akamaihd.net/battlelog/battlebinary",
        "https://bflabs-prod-eventbridge.ops.dice.se/",
        "prod_previewprod-prod_previewprod-santiago-common",
        "https://bflabs-prod-gt-cgw.ops.dice.se/",
        "undefined",
        "",
        "",
        "GLACIER_LBGW_BK_OL_SERVER",
        "6",
        "glacier_sp",
        "glacier_mp",
        "90",
        "dice.glacier.receipts.prod_labsprod.v1",
        "3",
        "https://bflabs-prod-eventbridge.ops.dice.se/",
        "https://bflabs-prod-gsgw-mtls.ops.dice.se/",
        "prod_previewprod-prod_previewprod-santiago-common",
        "https://bflabs-prod-gsgw.ops.dice.se/",
        "http://127.0.0.1/success",
        "CFQ7TTC0K5DJ",
    ])
}

fn conf_ch1_release_mp_labs() -> IndexMap<String, String> {
    conf_ch1_release_f2p_labs()
}

/// CONF map for `Util.fetchClientConfig` for Battlefield Labs stacks.
/// When CFID is unknown, `unknown_use_open_beta_stack` mirrors legacy `get_build_profile() == OpenBeta` fallback.
pub fn fetch_client_config_conf_map(
    cfid: &str,
    unknown_use_open_beta_stack: bool,
) -> IndexMap<String, String> {
    let kind = classify_cfid(cfid);
    match kind {
        CfidConfigKind::IdentityParams => conf_identity_params_labs(),
        CfidConfigKind::LabsLike => conf_ch1_release_mp11_labs(),
        CfidConfigKind::Event => conf_open_beta(),
        CfidConfigKind::Ch1ReleaseF2p => conf_ch1_release_f2p_labs(),
        CfidConfigKind::GlacierM1p => conf_glacier_m1p_labs(),
        CfidConfigKind::Ch1ReleaseMp => conf_ch1_release_mp_labs(),
        CfidConfigKind::Unknown => {
            if unknown_use_open_beta_stack {
                conf_open_beta()
            } else {
                conf_ch1_release_mp11_labs()
            }
        }
    }
}
