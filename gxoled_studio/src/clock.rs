//! The clock of the editor's preview: it can be paused with a button, and it stands still while nobody looks
//! at the preview (window hidden, another page selected) so nothing is rendered for it.
//!
//! The caller passes `now` in, which keeps the type free of hidden time and easy to test.

use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct PreviewClock {
    /// Play time gathered before the clock last stopped.
    base: Duration,
    /// When the clock last started; `None` while it is stopped.
    since: Option<Instant>,
    /// The user pressed pause. A page change does not undo that.
    user_paused: bool,
}

impl PreviewClock {
    /// A clock that already runs: a GIF that was just opened plays by itself.
    pub fn started(now: Instant) -> Self {
        Self { base: Duration::ZERO, since: Some(now), user_paused: false }
    }

    /// `true` while the preview advances.
    pub fn playing(&self) -> bool {
        self.since.is_some()
    }

    /// Play time so far, in milliseconds.
    pub fn elapsed_ms(&self, now: Instant) -> u64 {
        let running = self.since.map_or(Duration::ZERO, |s| now.saturating_duration_since(s));
        (self.base + running).as_millis() as u64
    }

    /// The play / pause button.
    pub fn toggle(&mut self, now: Instant) {
        if self.user_paused || self.since.is_none() {
            self.user_paused = false;
            self.start(now);
        } else {
            self.user_paused = true;
            self.stop(now);
        }
    }

    /// The preview went out of sight. The position is kept; a manual pause is remembered.
    pub fn suspend(&mut self, now: Instant) {
        self.stop(now);
    }

    /// The preview is in sight again: it continues unless the user paused it.
    pub fn resume(&mut self, now: Instant) {
        if !self.user_paused {
            self.start(now);
        }
    }

    /// A different GIF was opened: from the beginning, playing.
    pub fn restart(&mut self, now: Instant) {
        *self = Self::started(now);
    }

    /// Back to the first frame without changing whether the clock runs (a play-once GIF that is shown again).
    pub fn rewind(&mut self, now: Instant) {
        self.base = Duration::ZERO;
        if self.since.is_some() {
            self.since = Some(now);
        }
    }

    fn start(&mut self, now: Instant) {
        if self.since.is_none() {
            self.since = Some(now);
        }
    }

    fn stop(&mut self, now: Instant) {
        if let Some(since) = self.since.take() {
            self.base += now.saturating_duration_since(since);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(t0: Instant, ms: u64) -> Instant {
        t0 + Duration::from_millis(ms)
    }

    #[test]
    fn a_new_clock_plays_on_its_own() {
        let t0 = Instant::now();
        let c = PreviewClock::started(t0);
        assert!(c.playing());
        assert_eq!(c.elapsed_ms(at(t0, 250)), 250);
    }

    #[test]
    fn pause_freezes_the_position_and_play_continues_from_it() {
        let t0 = Instant::now();
        let mut c = PreviewClock::started(t0);
        c.toggle(at(t0, 400));
        assert!(!c.playing());
        assert_eq!(c.elapsed_ms(at(t0, 400)), 400);
        assert_eq!(c.elapsed_ms(at(t0, 90_000)), 400, "a paused clock does not advance");
        c.toggle(at(t0, 5_000));
        assert!(c.playing());
        assert_eq!(c.elapsed_ms(at(t0, 5_300)), 700);
    }

    #[test]
    fn leaving_the_page_stops_the_clock_and_coming_back_resumes_it() {
        let t0 = Instant::now();
        let mut c = PreviewClock::started(t0);
        c.suspend(at(t0, 300));
        assert!(!c.playing(), "nothing may run while the preview is out of sight");
        assert_eq!(c.elapsed_ms(at(t0, 60_000)), 300);
        c.resume(at(t0, 60_000));
        assert!(c.playing());
        assert_eq!(c.elapsed_ms(at(t0, 60_100)), 400);
    }

    #[test]
    fn a_manual_pause_survives_leaving_and_returning() {
        let t0 = Instant::now();
        let mut c = PreviewClock::started(t0);
        c.toggle(at(t0, 100));
        c.suspend(at(t0, 200));
        c.resume(at(t0, 300));
        assert!(!c.playing(), "the user's pause is not undone by a page change");
        assert_eq!(c.elapsed_ms(at(t0, 900)), 100);
        c.toggle(at(t0, 1_000));
        assert!(c.playing());
    }

    #[test]
    fn a_new_gif_plays_even_after_a_pause() {
        let t0 = Instant::now();
        let mut c = PreviewClock::started(t0);
        c.toggle(at(t0, 100));
        c.restart(at(t0, 500));
        assert!(c.playing());
        assert_eq!(c.elapsed_ms(at(t0, 530)), 30);
    }

    #[test]
    fn rewind_keeps_the_running_state() {
        let t0 = Instant::now();
        let mut c = PreviewClock::started(t0);
        c.rewind(at(t0, 2_000));
        assert!(c.playing());
        assert_eq!(c.elapsed_ms(at(t0, 2_040)), 40);
        c.toggle(at(t0, 2_100));
        c.rewind(at(t0, 3_000));
        assert!(!c.playing());
        assert_eq!(c.elapsed_ms(at(t0, 4_000)), 0);
    }

    #[test]
    fn time_never_runs_backwards() {
        let t0 = Instant::now() + Duration::from_secs(10);
        let c = PreviewClock::started(t0);
        assert_eq!(c.elapsed_ms(t0 - Duration::from_secs(1)), 0);
    }
}
