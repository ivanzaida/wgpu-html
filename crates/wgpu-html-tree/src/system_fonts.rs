pub use wgpu_html_assets::fonts::{SystemFontVariant, system_font_variants};

use crate::{FontFace, FontStyleAxis, Tree};

pub fn register_system_fonts(tree: &mut Tree, family: &str) -> usize {
    let variants = system_font_variants();
    for face in variants {
        let style = match face.style {
            wgpu_html_assets::FontStyleAxis::Normal => FontStyleAxis::Normal,
            wgpu_html_assets::FontStyleAxis::Italic => FontStyleAxis::Italic,
            wgpu_html_assets::FontStyleAxis::Oblique => FontStyleAxis::Oblique,
        };
        tree.register_font(FontFace {
            family: family.to_owned(),
            weight: face.weight,
            style,
            data: face.data.clone(),
        });
    }
    variants.len()
}
