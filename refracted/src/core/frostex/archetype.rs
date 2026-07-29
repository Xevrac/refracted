//! Content archetypes for FrostEx (viewing roles, not every RES enum).

use crate::core::frostex::index::{AssetKind, AssetRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Archetype {
    /// Folders / packages / bundles in the tree.
    Container,
    /// Image-like payloads (DxTexture, etc.).
    Picture,
    /// Geometry-like payloads (MeshSet, etc.).
    Model,
    /// Wave / sound resources.
    Audio,
    /// EBX / scripts / configs / generic binary.
    Data,
}

impl Archetype {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Container => "Container",
            Self::Picture => "Picture",
            Self::Model => "Model",
            Self::Audio => "Audio",
            Self::Data => "Data",
        }
    }

    pub fn from_asset(asset: &AssetRef) -> Self {
        match asset.kind {
            AssetKind::Toc | AssetKind::Sb => Self::Container,
            AssetKind::File => {
                let n = asset.name.to_ascii_lowercase();
                if n.ends_with(".cas") || n == "cas.cat" {
                    Self::Container
                } else if looks_image_name(&n) {
                    Self::Picture
                } else {
                    Self::Data
                }
            }
            AssetKind::Chunk => {
                // Chunks are often textures/audio; name hints help until bytes are sniffed.
                let n = asset.name.to_ascii_lowercase();
                if looks_image_name(&n) {
                    Self::Picture
                } else if n.contains("mesh") || n.contains("model") {
                    Self::Model
                } else {
                    Self::Data
                }
            }
            AssetKind::Ebx => {
                let n = asset.name.to_ascii_lowercase();
                if n.contains("mesh") || n.contains("model") || n.contains("geometry") {
                    Self::Model
                } else {
                    Self::Data
                }
            }
            AssetKind::Res => classify_res(asset.res_type, &asset.name),
        }
    }
}

fn classify_res(res_type: Option<u32>, name: &str) -> Archetype {
    if let Some(t) = res_type {
        match t {
            // DxTexture / Dx11Texture / Texture / AtlasTexture / ITexture / MovieTexture / RenderTexture
            0x5C4954A6 | 0xBCC7FB86 | 0x6BDE20BA | 0x957C32B1 | 0xC417BBD3 | 0x31E779A2
            | 0x41D57E10 | 0x2FF88D9E | 0x93BAA23F | 0x921476CA | 0xACD91FE8 => {
                return Archetype::Picture;
            }
            // MeshSet and mesh-adjacent
            0x49B156D4 | 0xBA02FEE0 | 0xC22CF759 | 0x3264E585 | 0x30B4A553 => {
                return Archetype::Model;
            }
            // NewWave / ImpulseResponse / sound-ish
            0xB2C465F6 | 0xC78B9D9D => return Archetype::Audio,
            _ => {}
        }
    }
    let n = name.to_ascii_lowercase();
    if looks_image_name(&n) {
        Archetype::Picture
    } else if n.contains("mesh") || n.contains("model") || n.contains("geometry") {
        Archetype::Model
    } else if n.contains("sound") || n.contains("wave") || n.contains("audio") {
        Archetype::Audio
    } else {
        Archetype::Data
    }
}

fn looks_image_name(n: &str) -> bool {
    n.contains("texture")
        || n.contains("diffuse")
        || n.contains("normal")
        || n.contains("specular")
        || n.contains("lut")
        || n.ends_with(".dds")
        || n.ends_with(".png")
        || n.ends_with(".tga")
}
