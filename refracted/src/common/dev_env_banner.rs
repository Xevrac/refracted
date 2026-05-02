const PREFIXES: &[&str] = &["CNC_EMU_", "REFRACTED_"];

pub fn dev_env_banner_line() -> Option<String> {
    let mut pairs: Vec<(String, String)> = std::env::vars()
        .filter(|(k, _)| PREFIXES.iter().any(|p| k.starts_with(p)))
        .collect();
    if pairs.is_empty() {
        return None;
    }
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let parts: Vec<String> = pairs
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    Some(parts.join("   "))
}
