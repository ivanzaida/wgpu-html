//! Font face types — `FontStyleAxis`, `FontFace`, `FontHandle`.

use std::sync::Arc;

/// CSS `font-style` axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontStyleAxis {
    Normal,
    Italic,
    Oblique,
}

impl Default for FontStyleAxis {
    fn default() -> Self { Self::Normal }
}

/// One physical font face: `(family, weight, style)` backed by font bytes.
#[derive(Debug, Clone)]
pub struct FontFace {
    pub family: String,
    pub weight: u16,
    pub style: FontStyleAxis,
    pub data: Arc<[u8]>,
}

impl FontFace {
    pub fn new(family: impl Into<String>, weight: u16, style: FontStyleAxis, data: Arc<[u8]>) -> Self {
        Self { family: family.into(), weight, style, data }
    }

    pub fn regular(family: impl Into<String>, data: Arc<[u8]>) -> Self {
        Self::new(family, 400, FontStyleAxis::Normal, data)
    }

    pub fn bold(family: impl Into<String>, data: Arc<[u8]>) -> Self {
        Self::new(family, 700, FontStyleAxis::Normal, data)
    }

    pub fn italic(family: impl Into<String>, data: Arc<[u8]>) -> Self {
        Self::new(family, 400, FontStyleAxis::Italic, data)
    }

    pub fn bold_italic(family: impl Into<String>, data: Arc<[u8]>) -> Self {
        Self::new(family, 700, FontStyleAxis::Italic, data)
    }
}

/// Stable handle returned when registering a face.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FontHandle(pub usize);
