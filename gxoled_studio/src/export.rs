//! 1-bit frames -> an ordinary two-colour GIF.

use gxoled_lib::Bitmap;

/// Encodes `frames` (all the same size) as a looping or play-once GIF with a black/white palette.
pub fn encode_gif(frames: &[(&Bitmap, u32)], looping: bool) -> anyhow::Result<Vec<u8>> {
    let Some((first, _)) = frames.first() else { anyhow::bail!("There are no frames to export") };
    let (w, h) = (first.w, first.h);
    if w == 0 || h == 0 || w > u16::MAX as usize || h > u16::MAX as usize {
        anyhow::bail!("Unsupported picture size {w}×{h}");
    }
    let mut out = Vec::new();
    {
        let mut enc = gif::Encoder::new(&mut out, w as u16, h as u16, &[0, 0, 0, 255, 255, 255])?;
        enc.set_repeat(if looping { gif::Repeat::Infinite } else { gif::Repeat::Finite(0) })?;
        for (bitmap, delay_ms) in frames {
            if (bitmap.w, bitmap.h) != (w, h) {
                anyhow::bail!("All frames must have the same size");
            }
            let pixels: Vec<u8> = bitmap.data.iter().map(|on| on as u8).collect();
            let mut frame = gif::Frame::from_indexed_pixels(w as u16, h as u16, pixels, None);
            frame.delay = ((*delay_ms as f32) / 10.0).round().clamp(2.0, u16::MAX as f32) as u16;
            enc.write_frame(&frame)?;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{AnimationDecoder, codecs::gif::GifDecoder};
    use std::io::Cursor;

    fn checker(w: usize, h: usize, phase: usize) -> Bitmap {
        let mut b = Bitmap::new(w, h, false);
        for y in 0..h {
            for x in 0..w {
                b.data.set(y * w + x, (x + y + phase).is_multiple_of(2));
            }
        }
        b
    }

    #[test]
    fn exported_frames_decode_back_to_the_same_pixels_and_delays() {
        let (a, b) = (checker(16, 8, 0), checker(16, 8, 1));
        let bytes = encode_gif(&[(&a, 100), (&b, 250)], true).unwrap();
        let frames: Vec<_> = GifDecoder::new(Cursor::new(&bytes)).unwrap().into_frames().collect::<Result<_, _>>().unwrap();
        assert_eq!(frames.len(), 2);
        for (frame, want) in frames.iter().zip([&a, &b]) {
            let img = frame.buffer();
            assert_eq!((img.width() as usize, img.height() as usize), (16, 8));
            for y in 0..8 {
                for x in 0..16 {
                    let px = img.get_pixel(x as u32, y as u32).0;
                    assert_eq!(px, if want.data[y * 16 + x] { [255, 255, 255, 255] } else { [0, 0, 0, 255] });
                }
            }
        }
        assert_eq!(frames[0].delay().numer_denom_ms().0, 100);
        assert_eq!(frames[1].delay().numer_denom_ms().0, 250);
    }

    #[test]
    fn export_then_import_is_lossless_for_the_studio_loader() {
        let a = checker(128, 64, 0);
        let bytes = encode_gif(&[(&a, 100)], false).unwrap();
        let src = crate::source::Source::decode(&bytes).unwrap();
        assert_eq!((src.w, src.h, src.frames.len()), (128, 64, 1));
        let back = crate::process::render_frame(&src.frames[0].rgba, src.w, src.h, &crate::settings::Settings { dither: crate::settings::Dither::Threshold, ..Default::default() }, 128, 64);
        assert_eq!(back, a);
    }

    #[test]
    fn loop_flag_is_written() {
        let a = checker(4, 4, 0);
        let looping = encode_gif(&[(&a, 100)], true).unwrap();
        let once = encode_gif(&[(&a, 100)], false).unwrap();
        // NETSCAPE2.0 application extension: loop count 0 = forever.
        let find = |bytes: &[u8]| bytes.windows(11).position(|w| w == b"NETSCAPE2.0").map(|p| (bytes[p + 11 + 2], bytes[p + 11 + 3]));
        assert_eq!(find(&looping), Some((0, 0)));
        assert_ne!(find(&once), Some((0, 0)));
    }

    #[test]
    fn bad_input_is_rejected() {
        assert!(encode_gif(&[], true).is_err());
        let (a, b) = (Bitmap::new(4, 4, false), Bitmap::new(5, 4, false));
        assert!(encode_gif(&[(&a, 100), (&b, 100)], true).is_err());
        assert!(encode_gif(&[(&Bitmap::new(0, 0, false), 100)], true).is_err());
    }
}
