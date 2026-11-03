//! Loading a GIF into straight-alpha RGBA frames.

use crate::process::SrcFrame;
use anyhow::{Context, bail};
use image::{AnimationDecoder, ImageDecoder, codecs::gif::GifDecoder, imageops::FilterType};
use std::io::Cursor;

pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_DIMENSION: u32 = 2048;
pub const MAX_FRAMES: usize = 2000;
pub const MAX_TOTAL_PIXELS: u64 = 64 * 1024 * 1024;
/// Frames larger than this are shrunk while loading; the screen is only 128 px wide, this leaves room to zoom.
pub const WORKING_DIMENSION: u32 = 480;
/// Browsers treat GIF delays below 20 ms as 100 ms; so do we.
pub const DEFAULT_DELAY_MS: u32 = 100;

#[derive(Clone, Debug)]
pub struct Source {
    pub w: usize,
    pub h: usize,
    pub frames: Vec<SrcFrame>,
}

impl Source {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Source> {
        let meta = std::fs::metadata(path).with_context(|| format!("Could not open {}", path.display()))?;
        if meta.len() > MAX_FILE_BYTES {
            bail!("GIF file is larger than the 64 MiB safety limit");
        }
        let bytes = std::fs::read(path).with_context(|| format!("Could not read {}", path.display()))?;
        Source::decode(&bytes)
    }

    pub fn decode(bytes: &[u8]) -> anyhow::Result<Source> {
        if !(bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a")) {
            bail!("The selected file is not a GIF image");
        }
        let decoder = GifDecoder::new(Cursor::new(bytes)).context("Could not decode GIF header")?;
        let (w, h) = decoder.dimensions();
        if w == 0 || h == 0 || w > MAX_DIMENSION || h > MAX_DIMENSION {
            bail!("GIF dimensions exceed the 2048 x 2048 safety limit");
        }
        let (tw, th) = working_size(w, h);
        let mut frames = Vec::new();
        let mut total = 0u64;
        for frame in decoder.into_frames() {
            if frames.len() >= MAX_FRAMES {
                bail!("GIF exceeds the 2,000-frame safety limit");
            }
            let frame = frame.context("Could not decode a GIF frame")?;
            let (numer, denom) = frame.delay().numer_denom_ms();
            let mut delay = (numer as u64 / denom.max(1) as u64) as u32;
            if delay < 20 {
                delay = DEFAULT_DELAY_MS;
            }
            let img = frame.into_buffer();
            total += u64::from(img.width()) * u64::from(img.height());
            if total > MAX_TOTAL_PIXELS {
                bail!("GIF exceeds the decoded-pixel safety limit");
            }
            let img = if (tw, th) != (img.width(), img.height()) {
                image::imageops::resize(&img, tw, th, FilterType::Triangle)
            } else {
                img
            };
            frames.push(SrcFrame { rgba: img.into_raw(), delay_ms: delay });
        }
        if frames.is_empty() {
            bail!("GIF contains no frames");
        }
        Ok(Source { w: tw as usize, h: th as usize, frames })
    }

    /// Sum of the source delays in milliseconds.
    pub fn duration_ms(&self) -> u64 {
        self.frames.iter().map(|f| f.delay_ms as u64).sum()
    }

    /// Frame `index` shrunk to fit `max_w` × `max_h` (straight alpha RGBA) for the editor's "source" pane.
    pub fn thumbnail(&self, index: usize, max_w: u32, max_h: u32) -> Option<(u32, u32, Vec<u8>)> {
        let f = self.frames.get(index)?;
        // Borrows the stored frame: this runs for every frame of the preview and must not copy half a megabyte.
        let img = image::ImageBuffer::<image::Rgba<u8>, &[u8]>::from_raw(self.w as u32, self.h as u32, f.rgba.as_slice())?;
        let k = (max_w as f32 / self.w as f32).min(max_h as f32 / self.h as f32).min(1.0);
        let (tw, th) = (((self.w as f32 * k).round() as u32).max(1), ((self.h as f32 * k).round() as u32).max(1));
        let pixels = if (tw, th) == (img.width(), img.height()) {
            f.rgba.clone()
        } else {
            image::imageops::resize(&img, tw, th, FilterType::Triangle).into_raw()
        };
        Some((tw, th, pixels))
    }
}

fn working_size(w: u32, h: u32) -> (u32, u32) {
    let longest = w.max(h);
    if longest <= WORKING_DIMENSION {
        return (w, h);
    }
    let k = WORKING_DIMENSION as f32 / longest as f32;
    (((w as f32 * k).round() as u32).max(1), ((h as f32 * k).round() as u32).max(1))
}

#[cfg(test)]
pub(crate) mod testing {
    /// Builds a GIF with the `gif` crate: every frame is a flat colour.
    pub fn gif_bytes(w: u16, h: u16, frames: &[([u8; 3], u16)]) -> Vec<u8> {
        let mut out = Vec::new();
        {
            let mut enc = gif::Encoder::new(&mut out, w, h, &[]).unwrap();
            enc.set_repeat(gif::Repeat::Infinite).unwrap();
            for (rgb, delay_cs) in frames {
                let mut frame = gif::Frame::from_palette_pixels(w, h, vec![0u8; w as usize * h as usize], rgb.to_vec(), None);
                frame.delay = *delay_cs;
                enc.write_frame(&frame).unwrap();
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::testing::gif_bytes;
    use super::*;

    #[test]
    fn decodes_frames_sizes_and_delays() {
        let bytes = gif_bytes(8, 4, &[([255, 0, 0], 5), ([0, 255, 0], 1), ([0, 0, 255], 20)]);
        let src = Source::decode(&bytes).unwrap();
        assert_eq!((src.w, src.h, src.frames.len()), (8, 4, 3));
        assert_eq!(&src.frames[0].rgba[..4], &[255, 0, 0, 255]);
        assert_eq!(&src.frames[2].rgba[..4], &[0, 0, 255, 255]);
        // 5 cs = 50 ms; 1 cs = 10 ms is below 20 ms and becomes 100 ms; 20 cs = 200 ms.
        assert_eq!(src.frames.iter().map(|f| f.delay_ms).collect::<Vec<_>>(), [50, 100, 200]);
        assert_eq!(src.duration_ms(), 350);
    }

    #[test]
    fn big_pictures_are_shrunk_to_the_working_size() {
        let bytes = gif_bytes(960, 480, &[([10, 20, 30], 5)]);
        let src = Source::decode(&bytes).unwrap();
        assert_eq!((src.w, src.h), (480, 240));
        assert_eq!(src.frames[0].rgba.len(), 480 * 240 * 4);
        let (w, h, px) = src.thumbnail(0, 100, 100).unwrap();
        assert_eq!((w, h, px.len()), (100, 50, 100 * 50 * 4));
        assert!(src.thumbnail(1, 10, 10).is_none());
    }

    #[test]
    fn rejects_files_that_are_not_gifs_or_are_broken() {
        assert!(Source::decode(b"not a gif at all").is_err());
        assert!(Source::decode(b"").is_err());
        let mut bytes = gif_bytes(4, 4, &[([1, 2, 3], 5)]);
        bytes.truncate(bytes.len() / 2);
        assert!(Source::decode(&bytes).is_err());
    }

    #[test]
    fn missing_file_is_a_clear_error() {
        let err = Source::load(std::path::Path::new("/definitely/not/here.gif")).unwrap_err();
        assert!(err.to_string().contains("Could not open"), "{err}");
    }
}
