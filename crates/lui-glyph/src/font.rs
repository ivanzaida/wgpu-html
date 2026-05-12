//! Font management via cosmic-text's `FontSystem`.
//!
//! Handles system font discovery and provides font metrics.

use cosmic_text::FontSystem;

pub struct FontContext {
    pub(crate) system: FontSystem,
}

impl FontContext {
    pub fn new() -> Self {
        Self { system: FontSystem::new() }
    }
}

impl Default for FontContext {
    fn default() -> Self { Self::new() }
}
