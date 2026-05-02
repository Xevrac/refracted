//! Labs-only runtime assets (Photon bundles, etc.) under `{exe}/data/client/labs/`.

use std::path::PathBuf;

/// Directory where the update-layer Photon `photon-bundle-*.js` files are installed (see `build.rs`).
pub fn photon_js_runtime_dir() -> PathBuf {
    crate::common::paths::app_data_dir()
        .join("client")
        .join("labs")
        .join("js")
}
