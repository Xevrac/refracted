fn main() {
    // Auto-increment version on each build
    increment_version();
    copy_runtime_assets();
    
    #[cfg(windows)]
    {
        use image::ImageFormat;
        use std::path::Path;
        println!("cargo:rerun-if-changed=../icon.png");

        let icon_png_path = Path::new("..").join("icon.png");
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let ico_path = Path::new(&out_dir).join("icon.ico");

        match image::open(&icon_png_path) {
            Ok(img) => {
                if let Err(e) = img.save_with_format(&ico_path, ImageFormat::Ico) {
                    eprintln!("Warning: Failed to convert icon PNG to ICO: {}", e);
                } else {
                    let mut res = winres::WindowsResource::new();
                    res.set_icon(&ico_path.to_string_lossy());
                    res.set_manifest_file("manifest.xml");
                    if let Err(e) = res.compile() {
                        eprintln!("Warning: Failed to embed Windows icon: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to load icon image '{}': {}",
                    icon_png_path.display(),
                    e
                );
            }
        }
    }
}

fn copy_runtime_assets() {
    use std::fs;
    use std::path::{Path, PathBuf};

    println!("cargo:rerun-if-changed=resources/default_games.json");
    println!("cargo:rerun-if-changed=src/client/labs/data/js/");
    println!("cargo:rerun-if-changed=src/client/cnc/data/");

    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => PathBuf::from(v),
        Err(_) => return,
    };
    let source_dir = manifest_dir
        .join("src")
        .join("client")
        .join("labs")
        .join("data")
        .join("js");
    if !source_dir.exists() {
        println!("cargo:warning=Runtime asset folder missing: {}", source_dir.display());
        return;
    }

    let out_dir = match std::env::var("OUT_DIR") {
        Ok(v) => PathBuf::from(v),
        Err(_) => return,
    };

    // OUT_DIR: target/<profile>/build/<pkg-hash>/out => profile dir is 3 levels up.
    let profile_dir = out_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf);

    if let Some(profile_dir) = profile_dir {
        let dest_dir = profile_dir
            .join("data")
            .join("client")
            .join("labs")
            .join("js");
        if fs::create_dir_all(&dest_dir).is_ok() {
            if let Ok(entries) = fs::read_dir(&source_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }
                    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                        continue;
                    };
                    if !name.starts_with("photon-bundle-") || !name.ends_with(".js") {
                        continue;
                    }
                    let dest = dest_dir.join(name);
                    if fs::copy(&path, &dest).is_ok() {
                        println!("cargo:warning=Copied runtime asset to {}", dest.display());
                    } else {
                        println!("cargo:warning=Failed to copy runtime asset to {}", dest.display());
                    }
                }
            }
        }

        let cnc_source_dir = manifest_dir
            .join("src")
            .join("client")
            .join("cnc")
            .join("data");
        let cnc_dest_dir = profile_dir
            .join("data")
            .join("client")
            .join("cnc");

        if cnc_dest_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&cnc_dest_dir) {
                println!(
                    "cargo:warning=Failed to clear CNC runtime data dir {}: {}",
                    cnc_dest_dir.display(),
                    e
                );
            }
        }

        if cnc_source_dir.exists() {
            if let Err(e) = copy_dir_recursive(&cnc_source_dir, &cnc_dest_dir) {
                println!(
                    "cargo:warning=Failed to copy CNC runtime data to {}: {}",
                    cnc_dest_dir.display(),
                    e
                );
            } else {
                println!(
                    "cargo:warning=Copied CNC runtime data to {}",
                    cnc_dest_dir.display()
                );
            }
        } else {
            println!(
                "cargo:warning=CNC runtime asset folder missing: {}",
                cnc_source_dir.display()
            );
        }
    }
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    use std::fs;

    fs::create_dir_all(dst).map_err(|e| format!("create_dir_all({}): {}", dst.display(), e))?;

    for entry in fs::read_dir(src).map_err(|e| format!("read_dir({}): {}", src.display(), e))? {
        let entry = entry.map_err(|e| format!("read_dir entry error: {}", e))?;
        let path = entry.path();
        let name = entry.file_name();
        let dest_path = dst.join(name);

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else if path.is_file() {
            fs::copy(&path, &dest_path).map_err(|e| {
                format!(
                    "copy({} -> {}): {}",
                    path.display(),
                    dest_path.display(),
                    e
                )
            })?;
        }
    }

    Ok(())
}

fn increment_version() {
    use std::fs;
    use std::io::Write;
    use std::path::Path;
    
    let cargo_toml_path = Path::new("Cargo.toml");
    
    if let Ok(contents) = fs::read_to_string(cargo_toml_path) {
        // Parse TOML
        if let Ok(mut doc) = toml::from_str::<toml::Value>(&contents) {
            if let Some(package) = doc.get_mut("package").and_then(|p| p.as_table_mut()) {
                if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
                    // Parse version (e.g., "0.1.0")
                    let parts: Vec<&str> = version.split('.').collect();
                    if parts.len() == 3 {
                        if let (Ok(major), Ok(minor), Ok(patch)) = (
                            parts[0].parse::<u32>(),
                            parts[1].parse::<u32>(),
                            parts[2].parse::<u32>(),
                        ) {
                            // Increment patch version
                            let new_patch = patch + 1;
                            let new_version = format!("{}.{}.{}", major, minor, new_patch);
                            
                            // Update the version in the TOML value
                            package.insert("version".to_string(), toml::Value::String(new_version.clone()));
                            
                            // Write back to Cargo.toml
                            if let Ok(toml_string) = toml::to_string_pretty(&doc) {
                                if let Ok(mut file) = fs::File::create(cargo_toml_path) {
                                    if file.write_all(toml_string.as_bytes()).is_ok() {
                                        println!("cargo:warning=Version incremented to {}", new_version);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

