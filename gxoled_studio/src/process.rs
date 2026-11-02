//! Source picture -> 1-bit OLED frame.
//!
//! Stage 1 puts the picture on the screen (rotate, mirror, fit, zoom, offset) while turning it into one gray
//! plane, with transparent pixels and the area around the picture taking the background colour.
//! Stage 2 (`pixelate`) applies the tone controls and the dither and packs the result into a [`Bitmap`].
//!
//! The tone and dither maths is a Rust port of the `pixelate` function of GGIF-Studio (MIT, (c) 2026
//! Abdelnour Osman); see THIRD_PARTY_NOTICES.md.

use crate::settings::{Background, Dither, Fit, Settings};
use gxoled_lib::{Bitmap, bitmap::BitVec};

/// One decoded source frame: straight-alpha RGBA, `w * h * 4` bytes.
#[derive(Clone, Debug, PartialEq)]
pub struct SrcFrame {
    pub rgba: Vec<u8>,
    pub delay_ms: u32,
}

/// 4×4 Bayer matrix thresholds, `(v + 0.5) * 16`.
const BAYER4: [[f32; 4]; 4] = {
    let m = [[0u8, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    let mut out = [[0.0f32; 4]; 4];
    let mut y = 0;
    while y < 4 {
        let mut x = 0;
        while x < 4 {
            out[y][x] = (m[y][x] as f32 + 0.5) * 16.0;
            x += 1;
        }
        y += 1;
    }
    out
};

/// Luminance (0..=255) of every source pixel after the background fill and the optional input inversion.
fn gray_plane(rgba: &[u8], pixels: usize, s: &Settings) -> Vec<f32> {
    let bg = match s.background {
        Background::Black => 0.0f32,
        Background::White => 255.0,
    };
    let mut out = Vec::with_capacity(pixels);
    for px in rgba.chunks_exact(4).take(pixels) {
        let luma = 0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32;
        let luma = if s.input_invert { 255.0 - luma } else { luma };
        let a = px[3] as f32 / 255.0;
        out.push(luma * a + bg * (1.0 - a));
    }
    out
}

/// Bilinear sample with texel centres at `i + 0.5`; everything outside the picture is `bg`.
fn sample(gray: &[f32], sw: usize, sh: usize, x: f32, y: f32, bg: f32) -> f32 {
    if x < 0.0 || y < 0.0 || x > sw as f32 || y > sh as f32 {
        return bg;
    }
    let fx = x - 0.5;
    let fy = y - 0.5;
    let x0 = fx.floor();
    let y0 = fy.floor();
    let tx = fx - x0;
    let ty = fy - y0;
    let at = |ix: f32, iy: f32| -> f32 {
        let ix = (ix.max(0.0) as usize).min(sw - 1);
        let iy = (iy.max(0.0) as usize).min(sh - 1);
        gray[iy * sw + ix]
    };
    let a = at(x0, y0);
    let b = at(x0 + 1.0, y0);
    let c = at(x0, y0 + 1.0);
    let d = at(x0 + 1.0, y0 + 1.0);
    (a * (1.0 - tx) + b * tx) * (1.0 - ty) + (c * (1.0 - tx) + d * tx) * ty
}

/// Places the source on a `w` × `h` screen and returns the gray plane (before tone and dither).
pub fn place(rgba: &[u8], sw: usize, sh: usize, s: &Settings, w: usize, h: usize) -> Vec<f32> {
    let bg = if s.background == Background::White { 255.0 } else { 0.0 };
    if sw == 0 || sh == 0 || w == 0 || h == 0 || rgba.len() < sw * sh * 4 {
        return vec![bg; w * h];
    }
    let gray = gray_plane(rgba, sw * sh, s);
    let rot = (s.rotate % 360) / 90 * 90;
    let (rw, rh) = if rot == 90 || rot == 270 { (sh as f32, sw as f32) } else { (sw as f32, sh as f32) };
    let (fw, fh) = (w as f32, h as f32);
    let (bx, by) = match s.fit {
        Fit::Contain => {
            let k = (fw / rw).min(fh / rh);
            (k, k)
        }
        Fit::Cover => {
            let k = (fw / rw).max(fh / rh);
            (k, k)
        }
        Fit::Stretch => (fw / rw, fh / rh),
    };
    let (kx, ky) = (bx * s.zoom, by * s.zoom);
    // Source pixels per screen pixel decides how many sub-samples average a screen pixel (box filter).
    let minify = (1.0 / kx).max(1.0 / ky);
    let n = (minify.ceil() as usize).clamp(1, 4);
    let step = 1.0 / n as f32;
    let mut out = Vec::with_capacity(w * h);
    for py in 0..h {
        for px in 0..w {
            let mut acc = 0.0;
            for sy in 0..n {
                for sx in 0..n {
                    let mut u = px as f32 + (sx as f32 + 0.5) * step - fw / 2.0 - s.offset_x as f32;
                    let mut v = py as f32 + (sy as f32 + 0.5) * step - fh / 2.0 - s.offset_y as f32;
                    if s.flip_h {
                        u = -u;
                    }
                    if s.flip_v {
                        v = -v;
                    }
                    u /= kx;
                    v /= ky;
                    // Inverse of a clockwise rotation (screen coordinates, y down).
                    let (xs, ys) = match rot {
                        90 => (v, -u),
                        180 => (-u, -v),
                        270 => (-v, u),
                        _ => (u, v),
                    };
                    acc += sample(&gray, sw, sh, xs + sw as f32 / 2.0, ys + sh as f32 / 2.0, bg);
                }
            }
            out.push(acc / (n * n) as f32);
        }
    }
    out
}

/// Tone controls, dither and output inversion on a gray plane (0..=255) of `w` × `h`.
pub fn pixelate(mut gray: Vec<f32>, w: usize, h: usize, s: &Settings) -> Bitmap {
    let n = w * h;
    gray.resize(n, 0.0);

    if s.normalize {
        let (mut lo, mut hi) = (255.0f32, 0.0f32);
        for &g in &gray {
            lo = lo.min(g);
            hi = hi.max(g);
        }
        let range = hi - lo;
        if range > 1.0 {
            let k = 255.0 / range;
            for g in &mut gray {
                *g = (*g - lo) * k;
            }
        }
    }

    let bri = s.brightness as f32 * 2.55;
    let c = s.contrast as f32 / 100.0;
    let cf = (1.0 + c) / (1.0 - c + 1e-6);
    let inv_gamma = 1.0 / s.gamma.max(0.1);
    for g in &mut gray {
        let v = 255.0 * (*g / 255.0).max(0.0).powf(inv_gamma) + bri;
        let v = (v - 128.0) * cf + 128.0;
        *g = v.clamp(0.0, 255.0);
    }

    let t = s.threshold as f32;
    let mut mono = vec![false; n];
    match s.dither {
        Dither::FloydSteinberg | Dither::Atkinson => {
            let atkinson = s.dither == Dither::Atkinson;
            let mut buf = gray;
            for y in 0..h {
                for x in 0..w {
                    let i = y * w + x;
                    let on = buf[i] >= t;
                    mono[i] = on;
                    let err = buf[i] - if on { 255.0 } else { 0.0 };
                    if atkinson {
                        let e = err / 8.0;
                        if x + 1 < w {
                            buf[i + 1] += e;
                        }
                        if x + 2 < w {
                            buf[i + 2] += e;
                        }
                        if y + 1 < h {
                            if x > 0 {
                                buf[i + w - 1] += e;
                            }
                            buf[i + w] += e;
                            if x + 1 < w {
                                buf[i + w + 1] += e;
                            }
                        }
                        if y + 2 < h {
                            buf[i + 2 * w] += e;
                        }
                    } else {
                        if x + 1 < w {
                            buf[i + 1] += err * 7.0 / 16.0;
                        }
                        if y + 1 < h {
                            if x > 0 {
                                buf[i + w - 1] += err * 3.0 / 16.0;
                            }
                            buf[i + w] += err * 5.0 / 16.0;
                            if x + 1 < w {
                                buf[i + w + 1] += err / 16.0;
                            }
                        }
                    }
                }
            }
        }
        Dither::Ordered => {
            for y in 0..h {
                for x in 0..w {
                    mono[y * w + x] = gray[y * w + x] >= BAYER4[y & 3][x & 3];
                }
            }
        }
        Dither::Threshold | Dither::None => {
            // "None" ignores the threshold control and cuts at the middle, like GGIF.
            let cut = if s.dither == Dither::None { 128.0 } else { t };
            for i in 0..n {
                mono[i] = gray[i] >= cut;
            }
        }
    }
    if s.invert {
        for m in &mut mono {
            *m = !*m;
        }
    }
    Bitmap { w, h, data: mono.into_iter().collect::<BitVec>() }
}

/// The whole pipeline for one source frame.
pub fn render_frame(rgba: &[u8], sw: usize, sh: usize, s: &Settings, w: usize, h: usize) -> Bitmap {
    pixelate(place(rgba, sw, sh, s, w, h), w, h, s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    fn solid(w: usize, h: usize, rgba: [u8; 4]) -> Vec<u8> {
        (0..w * h).flat_map(|_| rgba).collect()
    }

    fn set(img: &mut [u8], w: usize, x: usize, y: usize, rgba: [u8; 4]) {
        img[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&rgba);
    }

    fn lit(b: &Bitmap, x: usize, y: usize) -> bool {
        b.data[y * b.w + x]
    }

    fn count(b: &Bitmap) -> usize {
        b.data.iter().filter(|v| *v).count()
    }

    fn plain() -> Settings {
        Settings { dither: Dither::Threshold, ..Settings::default() }
    }

    // ── tone and dither (the GGIF pixelate port) ──

    #[test]
    fn threshold_cuts_at_the_chosen_level() {
        let g = vec![99.0, 100.0, 101.0, 255.0];
        let s = Settings { threshold: 100, dither: Dither::Threshold, ..Settings::default() };
        let b = pixelate(g, 4, 1, &s);
        assert_eq!((0..4).map(|x| lit(&b, x, 0)).collect::<Vec<_>>(), [false, true, true, true]);
    }

    #[test]
    fn dither_none_ignores_the_threshold_and_cuts_at_128() {
        let s = Settings { threshold: 10, dither: Dither::None, ..Settings::default() };
        let b = pixelate(vec![127.0, 128.0], 2, 1, &s);
        assert_eq!((lit(&b, 0, 0), lit(&b, 1, 0)), (false, true));
    }

    #[test]
    fn brightness_moves_and_invert_flips_the_result() {
        let base = plain();
        assert!(!lit(&pixelate(vec![100.0], 1, 1, &base), 0, 0));
        // +20 brightness = +51 levels: 100 -> 151, then the contrast pivot at 128 keeps it.
        assert!(lit(&pixelate(vec![100.0], 1, 1, &Settings { brightness: 20, ..base }), 0, 0));
        assert!(lit(&pixelate(vec![100.0], 1, 1, &Settings { invert: true, ..base }), 0, 0));
        assert!(!lit(&pixelate(vec![200.0], 1, 1, &Settings { invert: true, ..base }), 0, 0));
    }

    #[test]
    fn contrast_pushes_values_away_from_the_middle() {
        let base = plain();
        // factor (1 + 0.5) / (1 - 0.5) = 3: 140 -> 164 stays lit, 120 -> 96 stays dark; with threshold 150 only the first survives.
        let s = Settings { contrast: 50, threshold: 150, ..base };
        let b = pixelate(vec![140.0, 120.0], 2, 1, &s);
        assert_eq!((lit(&b, 0, 0), lit(&b, 1, 0)), (true, false));
        let flat = pixelate(vec![140.0, 120.0], 2, 1, &Settings { threshold: 150, ..base });
        assert_eq!((lit(&flat, 0, 0), lit(&flat, 1, 0)), (false, false));
    }

    #[test]
    fn gamma_above_one_brightens_the_midtones() {
        let base = plain();
        // 255 * (128/255)^(1/2.2) ≈ 186
        assert!(!lit(&pixelate(vec![100.0], 1, 1, &Settings { threshold: 150, ..base }), 0, 0));
        assert!(lit(&pixelate(vec![100.0], 1, 1, &Settings { threshold: 150, gamma: 2.2, ..base }), 0, 0));
    }

    #[test]
    fn normalize_stretches_a_dull_picture() {
        let base = plain();
        let dull = vec![60.0, 90.0];
        assert_eq!(count(&pixelate(dull.clone(), 2, 1, &base)), 0);
        let b = pixelate(dull, 2, 1, &Settings { normalize: true, ..base });
        assert_eq!((lit(&b, 0, 0), lit(&b, 1, 0)), (false, true));
        // A flat picture has no range to stretch and must not blow up.
        assert_eq!(count(&pixelate(vec![50.0, 50.0], 2, 1, &Settings { normalize: true, ..base })), 0);
    }

    #[test]
    fn ordered_dither_of_mid_gray_lights_half_of_each_4x4_block() {
        let s = Settings { dither: Dither::Ordered, ..Settings::default() };
        let b = pixelate(vec![128.0; 16], 4, 4, &s);
        assert_eq!(count(&b), 8);
    }

    #[test]
    fn error_diffusion_keeps_the_average_brightness() {
        for dither in [Dither::FloydSteinberg, Dither::Atkinson] {
            let s = Settings { dither, ..Settings::default() };
            let b = pixelate(vec![128.0; 64 * 64], 64, 64, &s);
            let lit = count(&b) as f32 / (64.0 * 64.0);
            // Atkinson drops a quarter of the error, so it drifts a little further from 50 %.
            assert!((0.40..=0.60).contains(&lit), "{dither:?}: {lit}");
        }
        // Black stays black and white stays white.
        let s = Settings::default();
        assert_eq!(count(&pixelate(vec![0.0; 100], 10, 10, &s)), 0);
        assert_eq!(count(&pixelate(vec![255.0; 100], 10, 10, &s)), 100);
    }

    #[test]
    fn floyd_steinberg_matches_a_hand_computed_row() {
        // Single row of 3 pixels at 100, threshold 128: p0 off (err 100 -> +43.75 to p1), p1 = 143.75 on
        // (err -111.25 -> -77.8 to p2), p2 = 22.2 off.
        let s = Settings { dither: Dither::FloydSteinberg, ..Settings::default() };
        let b = pixelate(vec![100.0; 3], 3, 1, &s);
        assert_eq!((lit(&b, 0, 0), lit(&b, 1, 0), lit(&b, 2, 0)), (false, true, false));
    }

    // ── geometry ──

    /// A 4×2 picture: white pixel at the left end of the top row, everything else black.
    fn marker() -> Vec<u8> {
        let mut img = solid(4, 2, [0, 0, 0, 255]);
        set(&mut img, 4, 0, 0, [255, 255, 255, 255]);
        img
    }

    fn only_lit(b: &Bitmap) -> Vec<(usize, usize)> {
        let mut v = Vec::new();
        for y in 0..b.h {
            for x in 0..b.w {
                if lit(b, x, y) {
                    v.push((x, y));
                }
            }
        }
        v
    }

    #[test]
    fn identity_keeps_the_marker_in_place() {
        let b = render_frame(&marker(), 4, 2, &plain(), 4, 2);
        assert_eq!(only_lit(&b), [(0, 0)]);
    }

    #[test]
    fn mirrors_move_the_marker_on_the_screen() {
        let h = render_frame(&marker(), 4, 2, &Settings { flip_h: true, ..plain() }, 4, 2);
        assert_eq!(only_lit(&h), [(3, 0)]);
        let v = render_frame(&marker(), 4, 2, &Settings { flip_v: true, ..plain() }, 4, 2);
        assert_eq!(only_lit(&v), [(0, 1)]);
    }

    #[test]
    fn rotation_turns_clockwise() {
        // 90° clockwise: the top-left corner goes to the top-right; the 4×2 picture needs a 2×4 screen.
        let r90 = render_frame(&marker(), 4, 2, &Settings { rotate: 90, ..plain() }, 2, 4);
        assert_eq!(only_lit(&r90), [(1, 0)]);
        let r180 = render_frame(&marker(), 4, 2, &Settings { rotate: 180, ..plain() }, 4, 2);
        assert_eq!(only_lit(&r180), [(3, 1)]);
        let r270 = render_frame(&marker(), 4, 2, &Settings { rotate: 270, ..plain() }, 2, 4);
        assert_eq!(only_lit(&r270), [(0, 3)]);
    }

    #[test]
    fn fit_modes_on_a_square_picture_and_a_wide_screen() {
        let white = solid(8, 8, [255, 255, 255, 255]);
        // Contain: 8x8 on 16x8 -> an 8-wide white block centred (columns 4..12).
        let c = render_frame(&white, 8, 8, &plain(), 16, 8);
        assert_eq!(count(&c), 64);
        assert!(!lit(&c, 3, 4) && lit(&c, 4, 4) && lit(&c, 11, 4) && !lit(&c, 12, 4));
        // Cover: fills the screen.
        let v = render_frame(&white, 8, 8, &Settings { fit: Fit::Cover, ..plain() }, 16, 8);
        assert_eq!(count(&v), 128);
        // Stretch: fills the screen too.
        let t = render_frame(&white, 8, 8, &Settings { fit: Fit::Stretch, ..plain() }, 16, 8);
        assert_eq!(count(&t), 128);
    }

    #[test]
    fn stretch_changes_the_aspect_and_contain_keeps_it() {
        // Left half white, right half black, 2:1 picture shown on a square screen.
        let mut img = solid(8, 4, [0, 0, 0, 255]);
        for y in 0..4 {
            for x in 0..4 {
                set(&mut img, 8, x, y, [255, 255, 255, 255]);
            }
        }
        let contain = render_frame(&img, 8, 4, &plain(), 8, 8);
        // Letterboxed: rows 2..6 carry the picture, the rest is background (black).
        assert!(!lit(&contain, 1, 0) && lit(&contain, 1, 3) && !lit(&contain, 6, 3) && !lit(&contain, 1, 7));
        let stretch = render_frame(&img, 8, 4, &Settings { fit: Fit::Stretch, ..plain() }, 8, 8);
        assert!(lit(&stretch, 1, 0) && lit(&stretch, 1, 7) && !lit(&stretch, 6, 0));
    }

    #[test]
    fn zoom_and_offset_move_the_picture() {
        let white = solid(4, 4, [255, 255, 255, 255]);
        let half = render_frame(&white, 4, 4, &Settings { zoom: 0.5, ..plain() }, 8, 8);
        assert_eq!(count(&half), 16, "half size = 4x4 on an 8x8 screen");
        let moved = render_frame(&white, 4, 4, &Settings { zoom: 0.5, offset_x: 2, offset_y: -2, ..plain() }, 8, 8);
        assert_eq!(only_lit(&moved).first().copied(), Some((4, 0)));
        assert_eq!(count(&moved), 16);
        let big = render_frame(&marker(), 4, 2, &Settings { zoom: 2.0, ..plain() }, 4, 2);
        assert!(count(&big) < 8, "only the middle of a zoomed picture stays visible");
    }

    #[test]
    fn transparent_pixels_take_the_background_colour() {
        let clear = solid(2, 2, [0, 0, 0, 0]);
        let black = render_frame(&clear, 2, 2, &plain(), 2, 2);
        assert_eq!(count(&black), 0);
        let white = render_frame(&clear, 2, 2, &Settings { background: Background::White, ..plain() }, 2, 2);
        assert_eq!(count(&white), 4);
        // Black-on-transparent artwork becomes visible with a white background + output inversion.
        let mut art = solid(2, 2, [0, 0, 0, 0]);
        set(&mut art, 2, 0, 0, [0, 0, 0, 255]);
        let s = Settings { background: Background::White, invert: true, ..plain() };
        assert_eq!(only_lit(&render_frame(&art, 2, 2, &s, 2, 2)), [(0, 0)]);
    }

    #[test]
    fn source_inversion_does_not_invert_the_background() {
        let white = solid(2, 2, [255, 255, 255, 255]);
        let s = Settings { input_invert: true, ..plain() };
        assert_eq!(count(&render_frame(&white, 2, 2, &s, 2, 2)), 0);
        let clear = solid(2, 2, [255, 255, 255, 0]);
        assert_eq!(count(&render_frame(&clear, 2, 2, &s, 2, 2)), 0, "transparent stays background (black)");
    }

    #[test]
    fn large_sources_are_averaged_not_point_sampled() {
        // 64x64 checkerboard of single pixels shrunk to 8x8 must come out as mid gray, not as aliasing noise.
        let mut img = solid(64, 64, [0, 0, 0, 255]);
        for y in 0..64usize {
            for x in 0..64usize {
                if (x + y).is_multiple_of(2) {
                    set(&mut img, 64, x, y, [255, 255, 255, 255]);
                }
            }
        }
        let g = place(&img, 64, 64, &Settings::default(), 8, 8);
        assert!(g.iter().all(|v| (100.0..156.0).contains(v)), "{:?}", &g[..8]);
    }

    #[test]
    fn bad_input_is_handled_without_panicking() {
        let s = Settings::default();
        assert_eq!(count(&render_frame(&[], 0, 0, &s, 8, 8)), 0);
        assert_eq!(count(&render_frame(&[1, 2, 3], 4, 4, &s, 8, 8)), 0);
        assert_eq!(render_frame(&solid(2, 2, [255; 4]), 2, 2, &s, 0, 0).data.len(), 0);
    }
}
