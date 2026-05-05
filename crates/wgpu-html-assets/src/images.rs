use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
  sync::{Arc, OnceLock},
  time::Instant,
};

// Public types

#[derive(Debug, Clone)]
pub struct ImageData {
  pub image_id: u64,
  pub data: Arc<Vec<u8>>,
  pub width: u32,
  pub height: u32,
  pub frames: Option<Arc<Vec<ImageFrame>>>,
}

#[derive(Debug, Clone)]
pub struct ImageFrame {
  pub image_id: u64,
  pub data: Arc<Vec<u8>>,
  pub delay_ms: u32,
}

// Internal types (pub(crate) for io.rs)

#[derive(Debug, Clone)]
pub(crate) struct DecodedFrameRaw {
  rgba: Arc<Vec<u8>>,
  delay_ms: u32,
}

#[derive(Debug, Clone)]
pub(crate) enum DecodedAsset {
  Still {
    rgba: Arc<Vec<u8>>,
    w: u32,
    h: u32,
  },
  Animated {
    frames: Arc<Vec<DecodedFrameRaw>>,
    w: u32,
    h: u32,
  },
}

pub(crate) type SizedKey = (String, Option<u32>, Option<u32>);

// Decode

fn is_webp(bytes: &[u8]) -> bool {
  bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
}

pub(crate) fn decode_asset(bytes: &[u8]) -> Option<DecodedAsset> {
  if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
    if let Some(asset) = decode_animated_gif(bytes) {
      return Some(asset);
    }
  }
  if is_webp(bytes) {
    if let Some(asset) = decode_animated_webp(bytes) {
      return Some(asset);
    }
  }
  let dyn_img = image::load_from_memory(bytes).ok()?;
  let rgba = dyn_img.to_rgba8();
  let (w, h) = (rgba.width(), rgba.height());
  Some(DecodedAsset::Still {
    rgba: Arc::new(rgba.into_raw()),
    w,
    h,
  })
}

fn decode_animated_webp(bytes: &[u8]) -> Option<DecodedAsset> {
  use image::{codecs::webp::WebPDecoder, AnimationDecoder};

  let decoder = WebPDecoder::new(std::io::Cursor::new(bytes)).ok()?;
  let frames = decoder.into_frames().collect_frames().ok()?;
  if frames.is_empty() {
    return None;
  }
  let (w, h) = {
    let first = frames.first()?;
    (first.buffer().width(), first.buffer().height())
  };
  if frames.len() == 1 {
    let buffer = frames.into_iter().next()?.into_buffer();
    return Some(DecodedAsset::Still {
      rgba: Arc::new(buffer.into_raw()),
      w,
      h,
    });
  }
  let decoded: Vec<DecodedFrameRaw> = frames
    .into_iter()
    .map(|f| {
      let (numer, denom) = f.delay().numer_denom_ms();
      let delay = if denom == 0 { 100 } else { numer / denom.max(1) };
      DecodedFrameRaw {
        rgba: Arc::new(f.into_buffer().into_raw()),
        delay_ms: delay.max(10),
      }
    })
    .collect();
  Some(DecodedAsset::Animated {
    frames: Arc::new(decoded),
    w,
    h,
  })
}

fn decode_animated_gif(bytes: &[u8]) -> Option<DecodedAsset> {
  use image::{codecs::gif::GifDecoder, AnimationDecoder};

  let decoder = GifDecoder::new(std::io::Cursor::new(bytes)).ok()?;
  let frames = decoder.into_frames().collect_frames().ok()?;
  if frames.is_empty() {
    return None;
  }
  let (w, h) = {
    let first = frames.first()?;
    (first.buffer().width(), first.buffer().height())
  };
  if frames.len() == 1 {
    let buffer = frames.into_iter().next()?.into_buffer();
    return Some(DecodedAsset::Still {
      rgba: Arc::new(buffer.into_raw()),
      w,
      h,
    });
  }
  let decoded: Vec<DecodedFrameRaw> = frames
    .into_iter()
    .map(|f| {
      let (numer, denom) = f.delay().numer_denom_ms();
      let delay = if denom == 0 { 100 } else { numer / denom.max(1) };
      DecodedFrameRaw {
        rgba: Arc::new(f.into_buffer().into_raw()),
        delay_ms: delay.max(10),
      }
    })
    .collect();
  Some(DecodedAsset::Animated {
    frames: Arc::new(decoded),
    w,
    h,
  })
}

// Resize + build sized

fn make_image_id(src: &str, declared_w: Option<u32>, declared_h: Option<u32>, frame_index: Option<usize>) -> u64 {
  let mut hasher = DefaultHasher::new();
  src.hash(&mut hasher);
  declared_w.hash(&mut hasher);
  declared_h.hash(&mut hasher);
  frame_index.hash(&mut hasher);
  hasher.finish()
}

fn resize_rgba(
  src_rgba: Arc<Vec<u8>>,
  decoded_w: u32,
  decoded_h: u32,
  target_w: u32,
  target_h: u32,
) -> Option<Arc<Vec<u8>>> {
  if target_w == decoded_w && target_h == decoded_h {
    return Some(src_rgba);
  }
  let buf = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(decoded_w, decoded_h, (*src_rgba).clone())?;
  let resized = image::imageops::resize(&buf, target_w, target_h, image::imageops::FilterType::Lanczos3);
  Some(Arc::new(resized.into_raw()))
}

pub(crate) fn build_sized(
  src: &str,
  asset: &DecodedAsset,
  declared_w: Option<u32>,
  declared_h: Option<u32>,
) -> Option<ImageData> {
  match asset {
    DecodedAsset::Still { rgba, w, h } => {
      let target_w = declared_w.unwrap_or(*w);
      let target_h = declared_h.unwrap_or(*h);
      let data = resize_rgba(rgba.clone(), *w, *h, target_w, target_h)?;
      Some(ImageData {
        image_id: make_image_id(src, declared_w, declared_h, None),
        data,
        width: target_w,
        height: target_h,
        frames: None,
      })
    }
    DecodedAsset::Animated { frames, w, h } => {
      let target_w = declared_w.unwrap_or(*w);
      let target_h = declared_h.unwrap_or(*h);
      let mut sized = Vec::with_capacity(frames.len());
      for (i, f) in frames.iter().enumerate() {
        let data = resize_rgba(f.rgba.clone(), *w, *h, target_w, target_h)?;
        sized.push(ImageFrame {
          image_id: make_image_id(src, declared_w, declared_h, Some(i)),
          data,
          delay_ms: f.delay_ms,
        });
      }
      let first = sized.first()?;
      let head_id = first.image_id;
      let head_data = first.data.clone();
      Some(ImageData {
        image_id: head_id,
        data: head_data,
        width: target_w,
        height: target_h,
        frames: Some(Arc::new(sized)),
      })
    }
  }
}

// Animation clock

fn animation_clock_origin() -> Instant {
  static ORIGIN: OnceLock<Instant> = OnceLock::new();
  *ORIGIN.get_or_init(Instant::now)
}

fn current_frame_index(frames: &[ImageFrame]) -> usize {
  let total: u64 = frames.iter().map(|f| f.delay_ms as u64).sum();
  if total == 0 {
    return 0;
  }
  let elapsed = animation_clock_origin().elapsed().as_millis() as u64;
  let t = elapsed % total;
  let mut acc = 0u64;
  for (i, f) in frames.iter().enumerate() {
    acc += f.delay_ms as u64;
    if t < acc {
      return i;
    }
  }
  frames.len() - 1
}

pub fn current_frame(data: &ImageData) -> ImageData {
  match &data.frames {
    Some(frames) if !frames.is_empty() => {
      let i = current_frame_index(frames);
      let f = &frames[i];
      ImageData {
        image_id: f.image_id,
        data: f.data.clone(),
        width: data.width,
        height: data.height,
        frames: data.frames.clone(),
      }
    }
    _ => data.clone(),
  }
}
