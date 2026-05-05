pub mod fetcher;
pub mod fonts;
pub mod images;
mod io;

#[cfg(feature = "blocking")]
pub mod blocking;

use std::{sync::Arc, time::Duration};

pub use fetcher::{FetchResponse, Fetcher};
pub use fonts::{system_font_variants, FontStyleAxis, SystemFontVariant};
pub use images::{current_frame, ImageData, ImageFrame};
pub use io::AssetIo;

#[derive(Clone, Debug, Default)]
pub struct FetchConfig {
  pub ttl: Option<Duration>,
}

#[derive(Clone)]
pub enum AssetStatus {
  Ready(Arc<[u8]>),
  Pending,
  Failed,
}
