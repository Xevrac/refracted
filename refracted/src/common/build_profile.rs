/// Blaze / client build channel used for emulator behaviour (Labs stacks, event, preview).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    Unknown,
    /// BFLabs main stack (`prod_labsprod`, CFID `labs` / `ch1-release-mp11` / `glacier-mp` first branch).
    Labs,
    LabsAlpha,
    /// Community event / open beta (`event` CFID, `eventprod`, `prod_alphaprod`).
    OpenBeta,
}
