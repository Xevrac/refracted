//! FrostEx tree icons — embedded SVG, rasterized once into egui textures.
//! Icons follow content *archetypes*, not every Frostbite type id.

use egui::{Color32, ColorImage, TextureHandle, TextureOptions, Vec2};
use std::collections::HashMap;

use crate::core::frostex::archetype::Archetype;
use crate::core::frostex::index::TreeNodeKind;

const ICON_PX: u32 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrostIcon {
    ChevronRight,
    ChevronDown,
    Folder,
    FolderOpen,
    Package,
    Bundle,
    Picture,
    Model,
    Audio,
    Data,
    Cas,
}

impl FrostIcon {
    fn svg(self) -> &'static str {
        match self {
            Self::ChevronRight => include_str!("icons/chevron_right.svg"),
            Self::ChevronDown => include_str!("icons/chevron_down.svg"),
            Self::Folder => include_str!("icons/folder.svg"),
            Self::FolderOpen => include_str!("icons/folder_open.svg"),
            Self::Package => include_str!("icons/toc.svg"),
            Self::Bundle => include_str!("icons/bundle.svg"),
            Self::Picture => include_str!("icons/picture.svg"),
            Self::Model => include_str!("icons/model.svg"),
            Self::Audio => include_str!("icons/audio.svg"),
            Self::Data => include_str!("icons/file.svg"),
            Self::Cas => include_str!("icons/cas.svg"),
        }
    }

    pub fn for_tree_node(kind: &TreeNodeKind, expanded: bool, label: &str) -> Self {
        let lower = label.to_ascii_lowercase();
        if lower == "cas.cat" || lower.ends_with(".cas") {
            return Self::Cas;
        }
        match kind {
            TreeNodeKind::Directory => {
                if expanded {
                    Self::FolderOpen
                } else {
                    Self::Folder
                }
            }
            TreeNodeKind::Toc | TreeNodeKind::Sb => Self::Package,
            TreeNodeKind::Bundle => Self::Bundle,
            TreeNodeKind::Ebx | TreeNodeKind::Chunk | TreeNodeKind::File => Self::Data,
            TreeNodeKind::Res => Self::Data,
        }
    }

    pub fn for_archetype(arch: Archetype) -> Self {
        match arch {
            Archetype::Container => Self::Package,
            Archetype::Picture => Self::Picture,
            Archetype::Model => Self::Model,
            Archetype::Audio => Self::Audio,
            Archetype::Data => Self::Data,
        }
    }
}

#[derive(Default)]
pub struct IconAtlas {
    textures: HashMap<FrostIcon, TextureHandle>,
}

impl IconAtlas {
    pub fn ensure(&mut self, ctx: &egui::Context) {
        for icon in [
            FrostIcon::ChevronRight,
            FrostIcon::ChevronDown,
            FrostIcon::Folder,
            FrostIcon::FolderOpen,
            FrostIcon::Package,
            FrostIcon::Bundle,
            FrostIcon::Picture,
            FrostIcon::Model,
            FrostIcon::Audio,
            FrostIcon::Data,
            FrostIcon::Cas,
        ] {
            if self.textures.contains_key(&icon) {
                continue;
            }
            let image = rasterize_svg(icon.svg(), ICON_PX).unwrap_or_else(|_| fallback_icon(icon));
            let tex = ctx.load_texture(
                format!("frostex_icon_{icon:?}"),
                image,
                TextureOptions::LINEAR,
            );
            self.textures.insert(icon, tex);
        }
    }

    pub fn get(&self, icon: FrostIcon) -> Option<&TextureHandle> {
        self.textures.get(&icon)
    }

    pub fn size() -> Vec2 {
        Vec2::splat(ICON_PX as f32)
    }
}

fn rasterize_svg(svg: &str, px: u32) -> Result<ColorImage, String> {
    let opts = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opts).map_err(|e| format!("svg parse: {e}"))?;
    let mut pixmap =
        tiny_skia::Pixmap::new(px, px).ok_or_else(|| "pixmap alloc failed".to_string())?;
    let size = tree.size();
    let sx = px as f32 / size.width().max(1.0);
    let sy = px as f32 / size.height().max(1.0);
    let transform = tiny_skia::Transform::from_scale(sx, sy);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    let mut rgba = Vec::with_capacity((px * px * 4) as usize);
    for pixel in pixmap.pixels() {
        rgba.push(pixel.red());
        rgba.push(pixel.green());
        rgba.push(pixel.blue());
        rgba.push(pixel.alpha());
    }
    Ok(ColorImage::from_rgba_unmultiplied(
        [px as usize, px as usize],
        &rgba,
    ))
}

fn fallback_icon(icon: FrostIcon) -> ColorImage {
    let mut rgba = vec![0u8; (ICON_PX * ICON_PX * 4) as usize];
    let color = match icon {
        FrostIcon::Folder | FrostIcon::FolderOpen => Color32::from_rgb(220, 180, 80),
        FrostIcon::Package => Color32::from_rgb(100, 180, 255),
        FrostIcon::Bundle => Color32::from_rgb(180, 140, 255),
        FrostIcon::Picture => Color32::from_rgb(120, 200, 160),
        FrostIcon::Model => Color32::from_rgb(255, 140, 100),
        FrostIcon::Audio => Color32::from_rgb(140, 160, 255),
        FrostIcon::Cas => Color32::from_rgb(160, 160, 180),
        _ => Color32::from_rgb(180, 180, 180),
    };
    for y in 2..ICON_PX - 2 {
        for x in 2..ICON_PX - 2 {
            let i = ((y * ICON_PX + x) * 4) as usize;
            rgba[i] = color.r();
            rgba[i + 1] = color.g();
            rgba[i + 2] = color.b();
            rgba[i + 3] = 255;
        }
    }
    ColorImage::from_rgba_unmultiplied([ICON_PX as usize, ICON_PX as usize], &rgba)
}
