pub mod fetcher;
pub mod fonts;
mod io;
pub mod images;

#[cfg(feature = "blocking")]
pub mod blocking;

pub use fetcher::{FetchResponse, Fetcher};
pub use io::AssetIo;
pub use fonts::{FontStyleAxis, SystemFontVariant, system_font_variants};
pub use images::{ImageData, ImageFrame, current_frame};

use std::sync::Arc;
use std::time::Duration;

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
