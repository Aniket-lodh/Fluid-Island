//! Pure stopwatch logic with no GTK/D-Bus dependency. Uses monotonic `Instant`
//! so late UI refreshes cannot corrupt the measured duration.

use std::time::{Duration, Instant};

/// Format a duration as MM:SS,cs with centiseconds.
pub fn format_duration_cs(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    let centis = duration.subsec_millis() / 10;
    format!("{:02}:{:02},{:02}", minutes, seconds, centis)
}

/// Stopwatch with start/pause/resume/reset/laps, backed by `Instant`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stopwatch {
    accumulated: Duration,
    started_at: Option<Instant>,
    laps: Vec<Duration>,
}

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            accumulated: Duration::ZERO,
            started_at: None,
            laps: Vec::new(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        match self.started_at {
            Some(started) => self.accumulated + started.elapsed(),
            None => self.accumulated,
        }
    }

    pub fn is_running(&self) -> bool {
        self.started_at.is_some()
    }

    /// Start the stopwatch from zero. If already running, resets and restarts.
    pub fn start(&mut self) {
        self.accumulated = Duration::ZERO;
        self.started_at = Some(Instant::now());
        self.laps.clear();
    }

    pub fn pause(&mut self) {
        if let Some(started) = self.started_at.take() {
            self.accumulated += started.elapsed();
        }
    }

    /// Resume a paused stopwatch. No-op if already running or stopped (zero).
    pub fn resume(&mut self) {
        if self.started_at.is_none() && self.accumulated > Duration::ZERO {
            self.started_at = Some(Instant::now());
        }
    }

    pub fn reset(&mut self) {
        self.accumulated = Duration::ZERO;
        self.started_at = None;
        self.laps.clear();
    }

    /// Record the current elapsed time as a lap. No-op when stopped at zero.
    pub fn lap(&mut self) -> Option<Duration> {
        let elapsed = self.elapsed();
        if elapsed.is_zero() && self.started_at.is_none() {
            return None;
        }
        self.laps.push(elapsed);
        Some(elapsed)
    }

    /// Lap timestamps in the order recorded.
    pub fn laps(&self) -> &[Duration] {
        &self.laps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn new_stopwatch_is_zero_and_stopped() {
        let s = Stopwatch::new();
        assert_eq!(s.elapsed(), Duration::ZERO);
        assert!(!s.is_running());
    }

    #[test]
    fn start_and_elapse() {
        let mut s = Stopwatch::new();
        s.start();
        assert!(s.is_running());
        assert!(s.elapsed() >= Duration::ZERO);
        std::thread::sleep(Duration::from_millis(10));
        assert!(s.elapsed() >= Duration::from_millis(10));
    }

    #[test]
    fn pause_preserves_elapsed() {
        let mut s = Stopwatch::new();
        s.start();
        std::thread::sleep(Duration::from_millis(50));
        s.pause();
        let paused = s.elapsed();
        assert!(paused >= Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(s.elapsed(), paused);
    }

    #[test]
    fn resume_adds_to_accumulated() {
        let mut s = Stopwatch::new();
        s.start();
        std::thread::sleep(Duration::from_millis(30));
        s.pause();
        let paused = s.elapsed();
        std::thread::sleep(Duration::from_millis(30));
        s.resume();
        assert!(s.is_running());
        assert!(s.elapsed() > paused);
    }

    #[test]
    fn reset_clears_and_stops() {
        let mut s = Stopwatch::new();
        s.start();
        std::thread::sleep(Duration::from_millis(30));
        s.reset();
        assert!(!s.is_running());
        assert_eq!(s.elapsed(), Duration::ZERO);
    }

    #[test]
    fn start_after_reset_restarts_from_zero() {
        let mut s = Stopwatch::new();
        s.start();
        std::thread::sleep(Duration::from_millis(30));
        s.start();
        std::thread::sleep(Duration::from_millis(20));
        assert!(s.elapsed() < Duration::from_millis(50));
    }

    #[test]
    fn pause_when_stopped_is_noop() {
        let mut s = Stopwatch::new();
        s.pause();
        assert_eq!(s.elapsed(), Duration::ZERO);
        assert!(!s.is_running());
    }

    #[test]
    fn resume_when_running_is_noop() {
        let mut s = Stopwatch::new();
        s.start();
        std::thread::sleep(Duration::from_millis(10));
        let before = s.elapsed();
        s.resume();
        let after = s.elapsed();
        assert!(after >= before);
        assert!(s.is_running());
        assert!(after >= Duration::from_millis(10));
    }

    #[test]
    fn resume_when_stopped_is_noop() {
        let mut s = Stopwatch::new();
        s.resume();
        assert_eq!(s.elapsed(), Duration::ZERO);
        assert!(!s.is_running());
    }

    #[test]
    fn lap_records_elapsed_while_running() {
        let mut s = Stopwatch::new();
        s.start();
        std::thread::sleep(Duration::from_millis(20));
        let lap = s.lap();
        assert!(lap.is_some());
        assert_eq!(s.laps().len(), 1);
        assert_eq!(s.laps()[0], lap.unwrap());
    }

    #[test]
    fn lap_is_noop_when_stopped_at_zero() {
        let mut s = Stopwatch::new();
        assert_eq!(s.lap(), None);
        assert!(s.laps().is_empty());
    }

    #[test]
    fn laps_accumulate_in_order() {
        let mut s = Stopwatch::new();
        s.start();
        std::thread::sleep(Duration::from_millis(10));
        let first = s.lap().unwrap();
        std::thread::sleep(Duration::from_millis(20));
        let second = s.lap().unwrap();
        assert_eq!(s.laps(), &[first, second]);
        assert!(second >= first);
    }

    #[test]
    fn reset_clears_laps() {
        let mut s = Stopwatch::new();
        s.start();
        s.lap();
        s.reset();
        assert!(s.laps().is_empty());
    }

    #[test]
    fn start_clears_laps() {
        let mut s = Stopwatch::new();
        s.start();
        s.lap();
        s.start();
        assert!(s.laps().is_empty());
    }

    #[test]
    fn format_cs_matches_reference_style() {
        assert_eq!(
            format_duration_cs(Duration::from_millis(13 * 60_000 + 12_550)),
            "13:12,55"
        );
        assert_eq!(format_duration_cs(Duration::ZERO), "00:00,00");
    }
}
