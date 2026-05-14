use std::sync::Arc;

use lui_glyph::{FontFace, FontStyleAxis};

pub fn dummy_font_data() -> Arc<[u8]> {
  Arc::from(vec![0u8; 16].into_boxed_slice())
}

pub fn dummy_face(family: &str) -> FontFace {
  FontFace::regular(family, dummy_font_data())
}

pub fn dummy_face_weighted(family: &str, weight: u16) -> FontFace {
  FontFace::new(family, weight, FontStyleAxis::Normal, dummy_font_data())
}

pub fn dummy_face_styled(family: &str, weight: u16, style: FontStyleAxis) -> FontFace {
  FontFace::new(family, weight, style, dummy_font_data())
}
