//! Which frames play, for how long, in what order.

use crate::settings::Settings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    /// Index into the source frames.
    pub index: usize,
    pub delay_ms: u32,
}

/// Applies trim, speed and the minimum frame time to the source delays.
///
/// `delay = max(source_delay / speed, min_frame_ms)`: the minimum is a send-rate cap for the OLED link, so it
/// is applied after the speed, exactly like GGIF's "minimum frame time".
pub fn playback_plan(delays_ms: &[u32], s: &Settings) -> Vec<Step> {
    if delays_ms.is_empty() {
        return Vec::new();
    }
    let last = delays_ms.len() - 1;
    let start = s.trim_start.min(last);
    let end = s.trim_end.unwrap_or(last).min(last).max(start);
    let speed = if s.speed.is_finite() { s.speed.clamp(0.1, 5.0) } else { 1.0 };
    (start..=end)
        .map(|index| {
            let scaled = (delays_ms[index] as f32 / speed).round() as u32;
            Step { index, delay_ms: scaled.max(s.min_frame_ms) }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untouched_settings_play_everything_at_source_speed() {
        let plan = playback_plan(&[100, 50, 200], &Settings::default());
        assert_eq!(plan.iter().map(|s| (s.index, s.delay_ms)).collect::<Vec<_>>(), [(0, 100), (1, 50), (2, 200)]);
    }

    #[test]
    fn speed_scales_delays_and_the_minimum_caps_them() {
        let s = Settings { speed: 2.0, ..Settings::default() };
        assert_eq!(playback_plan(&[100, 50, 40], &s).iter().map(|s| s.delay_ms).collect::<Vec<_>>(), [50, 30, 30]);
        let slow = Settings { speed: 0.5, ..Settings::default() };
        assert_eq!(playback_plan(&[100], &slow)[0].delay_ms, 200);
        let fast = Settings { speed: 5.0, min_frame_ms: 80, ..Settings::default() };
        assert_eq!(playback_plan(&[100, 400], &fast).iter().map(|s| s.delay_ms).collect::<Vec<_>>(), [80, 80]);
    }

    #[test]
    fn trim_keeps_the_inclusive_range() {
        let s = Settings { trim_start: 1, trim_end: Some(2), ..Settings::default() };
        assert_eq!(playback_plan(&[10, 20, 30, 40], &s).iter().map(|s| s.index).collect::<Vec<_>>(), [1, 2]);
        let past = Settings { trim_start: 9, trim_end: Some(20), ..Settings::default() };
        assert_eq!(playback_plan(&[10, 20, 30], &past).iter().map(|s| s.index).collect::<Vec<_>>(), [2]);
        let inverted = Settings { trim_start: 2, trim_end: Some(0), ..Settings::default() };
        assert_eq!(playback_plan(&[10, 20, 30], &inverted).iter().map(|s| s.index).collect::<Vec<_>>(), [2]);
        assert!(playback_plan(&[], &Settings::default()).is_empty());
    }

    #[test]
    fn nonsense_speed_cannot_stall_or_explode_the_plan() {
        for speed in [f32::NAN, 0.0, -3.0, f32::INFINITY, 1e9] {
            let s = Settings { speed, ..Settings::default() };
            let d = playback_plan(&[100], &s)[0].delay_ms;
            assert!((30..=1000).contains(&d), "{speed}: {d}");
        }
    }
}
