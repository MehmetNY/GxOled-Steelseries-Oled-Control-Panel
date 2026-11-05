//! Everything the user can change about how a GIF becomes a 1-bit OLED animation.
//!
//! The option set and its ranges follow the processing model of GGIF-Studio (MIT, (c) 2026 Abdelnour Osman):
//! fit / rotate / flip / zoom / offset, background fill, source inversion, brightness / contrast / gamma /
//! normalize / threshold, five dither modes, output inversion, and speed / minimum frame time / trim / loop.

use serde::{Deserialize, Deserializer, Serialize};

// Settings are read from files that people edit by hand. A whole number that is out of range (`threshold = 300`,
// `rotate = -90`) must be repaired, not rejected: one rejected value would make the application drop the entire
// settings file and fall back to the defaults for everything.
fn whole_number<'de, D: Deserializer<'de>>(d: D, lo: i64, hi: i64) -> Result<i64, D::Error> {
    Ok(i64::deserialize(d)?.clamp(lo, hi))
}

fn de_rotate<'de, D: Deserializer<'de>>(d: D) -> Result<u16, D::Error> {
    Ok(i64::deserialize(d)?.rem_euclid(360) as u16)
}

fn de_threshold<'de, D: Deserializer<'de>>(d: D) -> Result<u8, D::Error> {
    Ok(whole_number(d, 0, 255)? as u8)
}

fn de_min_frame_ms<'de, D: Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    Ok(whole_number(d, 10, MIN_FRAME_MS_MAX as i64)? as u32)
}

fn de_frame_index<'de, D: Deserializer<'de>>(d: D) -> Result<usize, D::Error> {
    Ok(whole_number(d, 0, MAX_FRAME_INDEX as i64)? as usize)
}

fn de_optional_frame_index<'de, D: Deserializer<'de>>(d: D) -> Result<Option<usize>, D::Error> {
    Ok(Option::<i64>::deserialize(d)?.map(|v| v.clamp(0, MAX_FRAME_INDEX as i64) as usize))
}

/// Longest "minimum frame time" (ms) the Min-frame slider offers.
const MIN_FRAME_MS_MAX: u32 = 200;
/// Far above any GIF the application accepts; only keeps hand-edited numbers from growing absurd.
const MAX_FRAME_INDEX: usize = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Dither {
    #[default]
    FloydSteinberg,
    Atkinson,
    Ordered,
    Threshold,
    None,
}

impl Dither {
    pub const ALL: [Dither; 5] = [Dither::FloydSteinberg, Dither::Atkinson, Dither::Ordered, Dither::Threshold, Dither::None];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Fit {
    /// Whole picture visible, empty bands filled with the background.
    #[default]
    Contain,
    /// Screen completely covered, the picture is cropped.
    Cover,
    /// Picture is stretched to the screen, aspect ratio is not kept.
    Stretch,
}

impl Fit {
    pub const ALL: [Fit; 3] = [Fit::Contain, Fit::Cover, Fit::Stretch];
}

/// What transparent pixels (and the area around the picture) become.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Background {
    #[default]
    Black,
    White,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    // geometry
    pub fit: Fit,
    /// Clockwise quarter turns in degrees: 0, 90, 180 or 270.
    #[serde(deserialize_with = "de_rotate")]
    pub rotate: u16,
    /// Mirrors are applied after the rotation, as seen on the screen.
    pub flip_h: bool,
    pub flip_v: bool,
    /// Zoom on top of `fit`, 0.2 ..= 3.
    pub zoom: f32,
    /// Shift in screen pixels, -64 ..= 64.
    pub offset_x: i32,
    pub offset_y: i32,

    // source
    pub background: Background,
    /// Invert the picture's colours before everything else (the background is not inverted).
    pub input_invert: bool,

    // tone
    pub brightness: i32,
    pub contrast: i32,
    pub gamma: f32,
    pub normalize: bool,
    #[serde(deserialize_with = "de_threshold")]
    pub threshold: u8,

    // output
    pub dither: Dither,
    pub invert: bool,

    // playback
    pub looping: bool,
    pub speed: f32,
    #[serde(deserialize_with = "de_min_frame_ms")]
    pub min_frame_ms: u32,
    /// First frame that plays (inclusive).
    #[serde(deserialize_with = "de_frame_index")]
    pub trim_start: usize,
    /// Last frame that plays (inclusive); `None` = the last frame of the source.
    #[serde(deserialize_with = "de_optional_frame_index")]
    pub trim_end: Option<usize>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fit: Fit::Contain,
            rotate: 0,
            flip_h: false,
            flip_v: false,
            zoom: 1.0,
            offset_x: 0,
            offset_y: 0,
            background: Background::Black,
            input_invert: false,
            brightness: 0,
            contrast: 0,
            gamma: 1.0,
            normalize: false,
            threshold: 128,
            dither: Dither::FloydSteinberg,
            invert: false,
            looping: true,
            speed: 1.0,
            min_frame_ms: 30,
            trim_start: 0,
            trim_end: None,
        }
    }
}

impl Settings {
    /// Forces every value into its valid range (settings come from files that people can edit by hand).
    pub fn clamped(mut self) -> Self {
        let fin = |v: f32, d: f32| if v.is_finite() { v } else { d };
        self.rotate = (self.rotate % 360) / 90 * 90;
        self.zoom = fin(self.zoom, 1.0).clamp(0.2, 3.0);
        self.offset_x = self.offset_x.clamp(-64, 64);
        self.offset_y = self.offset_y.clamp(-64, 64);
        self.brightness = self.brightness.clamp(-100, 100);
        self.contrast = self.contrast.clamp(-100, 100);
        self.gamma = fin(self.gamma, 1.0).clamp(0.1, 3.0);
        self.speed = fin(self.speed, 1.0).clamp(0.1, 5.0);
        self.min_frame_ms = self.min_frame_ms.clamp(10, MIN_FRAME_MS_MAX);
        if let Some(end) = self.trim_end
            && end < self.trim_start
        {
            self.trim_end = Some(self.trim_start);
        }
        self
    }

    /// `true` when both settings produce the same 1-bit pictures (playback-only options may differ).
    pub fn same_look(&self, other: &Settings) -> bool {
        self.fit == other.fit
            && self.rotate == other.rotate
            && self.flip_h == other.flip_h
            && self.flip_v == other.flip_v
            && self.zoom == other.zoom
            && self.offset_x == other.offset_x
            && self.offset_y == other.offset_y
            && self.background == other.background
            && self.input_invert == other.input_invert
            && self.brightness == other.brightness
            && self.contrast == other.contrast
            && self.gamma == other.gamma
            && self.normalize == other.normalize
            && self.threshold == other.threshold
            && self.dither == other.dither
            && self.invert == other.invert
    }
}

/// A numeric setting that the window edits with a slider. Values are integers (percent for fractions).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Param {
    Zoom,
    OffsetX,
    OffsetY,
    Brightness,
    Contrast,
    Gamma,
    Threshold,
    Speed,
    MinFrame,
    TrimStart,
    TrimEnd,
}

impl Param {
    pub const ALL: [Param; 11] = [
        Param::Zoom,
        Param::OffsetX,
        Param::OffsetY,
        Param::Brightness,
        Param::Contrast,
        Param::Gamma,
        Param::Threshold,
        Param::Speed,
        Param::MinFrame,
        Param::TrimStart,
        Param::TrimEnd,
    ];

    /// Slider range; the trim sliders depend on how many frames the source has.
    pub fn range(self, frames: usize) -> (i32, i32) {
        let last = frames.saturating_sub(1) as i32;
        match self {
            Param::Zoom => (20, 300),
            Param::OffsetX | Param::OffsetY => (-64, 64),
            Param::Brightness | Param::Contrast => (-100, 100),
            Param::Gamma => (10, 300),
            Param::Threshold => (0, 255),
            Param::Speed => (10, 500),
            Param::MinFrame => (10, MIN_FRAME_MS_MAX as i32),
            Param::TrimStart | Param::TrimEnd => (0, last),
        }
    }

    pub fn default_value(self, frames: usize) -> i32 {
        match self {
            Param::Zoom | Param::Gamma | Param::Speed => 100,
            Param::Threshold => 128,
            Param::MinFrame => 30,
            Param::TrimEnd => frames.saturating_sub(1) as i32,
            _ => 0,
        }
    }

    pub fn get(self, s: &Settings, frames: usize) -> i32 {
        let last = frames.saturating_sub(1);
        match self {
            Param::Zoom => (s.zoom * 100.0).round() as i32,
            Param::OffsetX => s.offset_x,
            Param::OffsetY => s.offset_y,
            Param::Brightness => s.brightness,
            Param::Contrast => s.contrast,
            Param::Gamma => (s.gamma * 100.0).round() as i32,
            Param::Threshold => s.threshold as i32,
            Param::Speed => (s.speed * 100.0).round() as i32,
            Param::MinFrame => s.min_frame_ms as i32,
            Param::TrimStart => s.trim_start.min(last) as i32,
            Param::TrimEnd => s.trim_end.unwrap_or(last).min(last) as i32,
        }
    }

    pub fn set(self, s: &mut Settings, value: i32, frames: usize) {
        let (lo, hi) = self.range(frames);
        let v = value.clamp(lo, hi.max(lo));
        let last = frames.saturating_sub(1);
        match self {
            Param::Zoom => s.zoom = v as f32 / 100.0,
            Param::OffsetX => s.offset_x = v,
            Param::OffsetY => s.offset_y = v,
            Param::Brightness => s.brightness = v,
            Param::Contrast => s.contrast = v,
            Param::Gamma => s.gamma = v as f32 / 100.0,
            Param::Threshold => s.threshold = v as u8,
            Param::Speed => s.speed = v as f32 / 100.0,
            Param::MinFrame => s.min_frame_ms = v as u32,
            Param::TrimStart => {
                s.trim_start = v as usize;
                let end = s.trim_end.unwrap_or(last);
                if end < s.trim_start {
                    s.trim_end = Some(s.trim_start);
                }
            }
            Param::TrimEnd => {
                let end = v as usize;
                s.trim_end = if end >= last { None } else { Some(end) };
                if s.trim_start > end {
                    s.trim_start = end;
                }
            }
        }
    }

    /// Text for the slider's value read-out ("1.25×", "+12 px", "30 ms", "Frame 4").
    pub fn format(self, value: i32) -> String {
        match self {
            Param::Zoom | Param::Speed => format!("{}.{:02}×", value / 100, value % 100),
            Param::Gamma => format!("{}.{:02}", value / 100, value % 100),
            Param::OffsetX | Param::OffsetY => format!("{value:+} px"),
            Param::Brightness | Param::Contrast => format!("{value:+}"),
            Param::Threshold => value.to_string(),
            Param::MinFrame => format!("{value} ms"),
            Param::TrimStart | Param::TrimEnd => format!("#{}", value + 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_ggif_defaults() {
        let s = Settings::default();
        assert_eq!((s.threshold, s.dither, s.fit, s.min_frame_ms), (128, Dither::FloydSteinberg, Fit::Contain, 30));
        assert_eq!((s.brightness, s.contrast, s.gamma, s.zoom, s.speed), (0, 0, 1.0, 1.0, 1.0));
        assert!(s.looping && !s.normalize && !s.invert && !s.input_invert);
        assert_eq!(s, s.clamped());
    }

    #[test]
    fn clamping_repairs_hand_edited_values() {
        let s = Settings {
            rotate: 450,
            zoom: f32::NAN,
            offset_x: 900,
            offset_y: -900,
            brightness: 500,
            contrast: -500,
            gamma: 99.0,
            speed: 0.0,
            min_frame_ms: 0,
            trim_start: 5,
            trim_end: Some(2),
            ..Settings::default()
        }
        .clamped();
        assert_eq!(s.rotate, 90);
        assert_eq!(s.zoom, 1.0);
        assert_eq!((s.offset_x, s.offset_y, s.brightness, s.contrast), (64, -64, 100, -100));
        assert_eq!((s.gamma, s.speed, s.min_frame_ms), (3.0, 0.1, 10));
        assert_eq!((s.trim_start, s.trim_end), (5, Some(5)));
    }

    #[test]
    fn out_of_range_numbers_in_a_file_are_repaired_not_rejected() {
        // One bad number used to make the whole file unreadable, so every other setting was lost with it.
        let json = r#"{"threshold":300,"rotate":-90,"min_frame_ms":5000,"trim_start":-4,"trim_end":-2,"zoom":9.0,"invert":true}"#;
        let s: Settings = serde_json::from_str(json).expect("an out-of-range number must not reject the file");
        assert_eq!((s.threshold, s.rotate, s.min_frame_ms, s.trim_start, s.trim_end), (255, 270, 200, 0, Some(0)));
        assert!(s.invert, "the other values of the file are kept");
        assert_eq!(s.clamped().zoom, 3.0);
        // In-range values are untouched, and absent ones fall back to the defaults.
        let s: Settings = serde_json::from_str(r#"{"threshold":77,"rotate":180,"trim_end":4}"#).unwrap();
        assert_eq!((s.threshold, s.rotate, s.trim_end, s.min_frame_ms), (77, 180, Some(4), 30));
        // A value that is not a number at all is still an error (it is not a hand-edit slip we can interpret).
        assert!(serde_json::from_str::<Settings>(r#"{"threshold":"high"}"#).is_err());
    }

    #[test]
    fn the_minimum_frame_time_range_is_the_one_of_the_slider() {
        assert_eq!(Param::MinFrame.range(10), (10, 200));
        assert_eq!(Settings { min_frame_ms: 500, ..Settings::default() }.clamped().min_frame_ms, 200);
    }

    #[test]
    fn params_round_trip_and_clamp() {
        let frames = 10;
        for p in Param::ALL {
            let mut s = Settings::default();
            let (lo, hi) = p.range(frames);
            assert_eq!(p.get(&s, frames), p.default_value(frames), "{p:?}");
            p.set(&mut s, hi + 1000, frames);
            assert_eq!(p.get(&s, frames), hi, "{p:?} above");
            p.set(&mut s, lo - 1000, frames);
            assert_eq!(p.get(&s, frames), lo, "{p:?} below");
            assert!(!p.format(lo).is_empty());
        }
    }

    #[test]
    fn trim_sliders_never_cross() {
        let mut s = Settings::default();
        Param::TrimStart.set(&mut s, 6, 10);
        Param::TrimEnd.set(&mut s, 3, 10);
        assert_eq!((s.trim_start, s.trim_end), (3, Some(3)));
        Param::TrimStart.set(&mut s, 8, 10);
        assert_eq!((s.trim_start, s.trim_end), (8, Some(8)));
        Param::TrimEnd.set(&mut s, 9, 10);
        assert_eq!(s.trim_end, None, "the last frame means 'to the end'");
    }

    #[test]
    fn the_look_ignores_playback_options() {
        let a = Settings::default();
        let b = Settings { speed: 2.0, looping: false, min_frame_ms: 80, trim_start: 2, ..a };
        assert!(a.same_look(&b));
        assert!(!a.same_look(&Settings { threshold: 90, ..a }));
        assert!(!a.same_look(&Settings { dither: Dither::Atkinson, ..a }));
    }

    #[test]
    fn value_labels() {
        assert_eq!(Param::Zoom.format(125), "1.25×");
        assert_eq!(Param::OffsetX.format(12), "+12 px");
        assert_eq!(Param::OffsetX.format(-3), "-3 px");
        assert_eq!(Param::MinFrame.format(30), "30 ms");
        assert_eq!(Param::TrimStart.format(0), "#1");
    }
}
