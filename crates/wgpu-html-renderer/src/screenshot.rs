//! Read back the rendered surface texture and save it as a PNG.
//!
//! wgpu requires `bytes_per_row` of any texture-to-buffer copy to be a
//! multiple of [`wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`] (256), so the
//! staging buffer layout has padding that we strip on the way out.
//!
//! Surface format is whichever sRGB format the swapchain picked
//! (typically `Bgra8UnormSrgb` on Windows, `Rgba8UnormSrgb` on others).
//! We swizzle BGRA → RGBA before writing.

use std::path::Path;

const ALIGN: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

#[derive(Debug)]
pub enum ScreenshotError {
    UnsupportedFormat(wgpu::TextureFormat),
    Map(wgpu::BufferAsyncError),
    Encode(image::ImageError),
}

impl std::fmt::Display for ScreenshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedFormat(fmt) => write!(f, "unsupported surface format: {fmt:?}"),
            Self::Map(e) => write!(f, "buffer map failed: {e:?}"),
            Self::Encode(e) => write!(f, "PNG encode failed: {e}"),
        }
    }
}

impl std::error::Error for ScreenshotError {}

pub(crate) struct StagingBuffer {
    buffer: wgpu::Buffer,
    bytes_per_row_padded: u32,
    bytes_per_row_unpadded: u32,
}

/// Append a copy of `frame` into a freshly-allocated staging buffer to
/// `encoder`. Returns the staging buffer to be mapped after submission.
pub(crate) fn begin_capture(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    frame: &wgpu::Texture,
    width: u32,
    height: u32,
) -> StagingBuffer {
    let bytes_per_row_unpadded = 4 * width;
    let bytes_per_row_padded = bytes_per_row_unpadded.div_ceil(ALIGN) * ALIGN;
    let buffer_size = bytes_per_row_padded as u64 * height as u64;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("screenshot staging"),
        size: buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: frame,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row_padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    StagingBuffer {
        buffer,
        bytes_per_row_padded,
        bytes_per_row_unpadded,
    }
}

/// Map the staging buffer, strip padding, swizzle BGRA→RGBA if needed,
/// and write a PNG to `path`. Blocks until the GPU has finished the
/// previously-submitted copy.
pub(crate) fn finish_capture(
    device: &wgpu::Device,
    stg: StagingBuffer,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    path: &Path,
) -> Result<(), ScreenshotError> {
    let needs_bgra_swizzle = match format {
        wgpu::TextureFormat::Bgra8UnormSrgb | wgpu::TextureFormat::Bgra8Unorm => true,
        wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Rgba8Unorm => false,
        other => return Err(ScreenshotError::UnsupportedFormat(other)),
    };

    let slice = stg.buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv().expect("map callback dropped").map_err(ScreenshotError::Map)?;

    let mut rgba = Vec::with_capacity((stg.bytes_per_row_unpadded * height) as usize);
    {
        let data = slice.get_mapped_range();
        for y in 0..height as usize {
            let row_start = y * stg.bytes_per_row_padded as usize;
            let row_end = row_start + stg.bytes_per_row_unpadded as usize;
            rgba.extend_from_slice(&data[row_start..row_end]);
        }
    }
    stg.buffer.unmap();

    if needs_bgra_swizzle {
        for px in rgba.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
    }

    image::save_buffer(path, &rgba, width, height, image::ColorType::Rgba8)
        .map_err(ScreenshotError::Encode)?;

    Ok(())
}
