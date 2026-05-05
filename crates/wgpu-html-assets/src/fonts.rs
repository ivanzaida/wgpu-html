use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyleAxis {
    Normal,
    Italic,
    Oblique,
}

const FONT_FAMILIES: &[[&str; 4]] = &[
    [
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\segoeuib.ttf",
        "C:\\Windows\\Fonts\\segoeuii.ttf",
        "C:\\Windows\\Fonts\\segoeuiz.ttf",
    ],
    [
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\arialbd.ttf",
        "C:\\Windows\\Fonts\\ariali.ttf",
        "C:\\Windows\\Fonts\\arialbi.ttf",
    ],
    [
        "C:\\Windows\\Fonts\\calibri.ttf",
        "C:\\Windows\\Fonts\\calibrib.ttf",
        "C:\\Windows\\Fonts\\calibrii.ttf",
        "C:\\Windows\\Fonts\\calibriz.ttf",
    ],
    [
        "C:\\Windows\\Fonts\\tahoma.ttf",
        "C:\\Windows\\Fonts\\tahomabd.ttf",
        "",
        "",
    ],
    [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Oblique.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-BoldOblique.ttf",
    ],
    [
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-Oblique.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-BoldOblique.ttf",
    ],
    [
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans-Oblique.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans-BoldOblique.ttf",
    ],
    ["/System/Library/Fonts/Helvetica.ttc", "", "", ""],
    ["/Library/Fonts/Arial.ttf", "", "", ""],
];

#[derive(Clone)]
pub struct SystemFontVariant {
    pub weight: u16,
    pub style: FontStyleAxis,
    pub data: Arc<[u8]>,
}

pub fn system_font_variants() -> &'static [SystemFontVariant] {
    static FACES: OnceLock<Vec<SystemFontVariant>> = OnceLock::new();
    FACES
        .get_or_init(|| {
            for row in FONT_FAMILIES {
                let regular_path = row[0];
                let Ok(reg_bytes) = std::fs::read(regular_path) else {
                    continue;
                };
                let mut out = vec![SystemFontVariant {
                    weight: 400,
                    style: FontStyleAxis::Normal,
                    data: Arc::from(reg_bytes.into_boxed_slice()),
                }];
                let variants: [(&str, u16, FontStyleAxis); 3] = [
                    (row[1], 700, FontStyleAxis::Normal),
                    (row[2], 400, FontStyleAxis::Italic),
                    (row[3], 700, FontStyleAxis::Italic),
                ];
                for (path, weight, style) in variants {
                    if path.is_empty() {
                        continue;
                    }
                    if let Ok(bytes) = std::fs::read(path) {
                        out.push(SystemFontVariant {
                            weight,
                            style,
                            data: Arc::from(bytes.into_boxed_slice()),
                        });
                    }
                }
                return out;
            }
            Vec::new()
        })
        .as_slice()
}
