use crate::common::build_profile::BuildProfile;

/// Heuristic for Battlefield Labs: classify from Blaze/HTTP/XML blobs (preAuth, env, client config).
/// Order: OpenBeta → LabsAlpha → Labs.
pub fn labs_hint_build_profile_from_text(text: &str) -> BuildProfile {
    let lower = text.to_ascii_lowercase();

    if lower.contains("releasetypeoverride=\"beta\"")
        || lower.contains("beta_online_access")
        || lower.contains("beta_earlyaccess")
        || lower.contains("bf-community-pc-event")
        || lower.contains("environmentid=ch1-release_bf-community-pc-event")
        || lower.contains("productid=ftsd")
        || lower.contains("eventprod-mp-cgw")
        || lower.contains("eventprod-eventbridge")
        || lower.contains("prod_alphaprod")
    {
        return BuildProfile::OpenBeta;
    }

    if lower.contains("prod_previewprod")
        || lower.contains("glacier-m1p")
        || lower.contains("releasetypeoverride=\"unknown\"")
        || lower.contains("application_version=nil")
        || lower.contains("sdk_version=1%2e29%2e0")
        || lower.contains("bf-2026-pc-labs")
        || lower.contains("dice-dev-retail-x64-0.1.0")
    {
        return BuildProfile::LabsAlpha;
    }

    if lower.contains("prod_labsprod")
        || lower.contains("bflabs-prod-gt-cgw")
        || lower.contains("bflabs-prod-eventbridge")
        || lower.contains("bflabs-prod-gsgw")
        || lower.contains("ch1-release-mp11")
        || lower.contains("bf-community-pc-labs")
        || lower.contains("ch1-release_bf-community-pc-labs")
        || lower.contains("dice-dev-retail")
    {
        return BuildProfile::Labs;
    }

    BuildProfile::Unknown
}
