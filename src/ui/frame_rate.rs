//! Smoothed rate of viewport frame submissions, distinct from GPU execution timing.
use std::time::{Duration, Instant};
pub struct FrameRate {
    start: Instant,
    frames: u32,
    pub fps: f64,
    pub milliseconds: f64,
}
impl Default for FrameRate {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            frames: 0,
            fps: 0.,
            milliseconds: 0.,
        }
    }
}
impl FrameRate {
    pub fn tick(&mut self) {
        self.sample(Instant::now());
    }
    fn sample(&mut self, now: Instant) {
        self.frames += 1;
        let elapsed = now.duration_since(self.start);
        if elapsed >= Duration::from_millis(500) {
            self.fps = self.frames as f64 / elapsed.as_secs_f64();
            self.milliseconds = 1000. / self.fps;
            self.start = now;
            self.frames = 0;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn uses_elapsed_time_instead_of_assuming_refresh_rate() {
        let mut rate = FrameRate::default();
        let start = rate.start;
        for i in 1..=30 {
            rate.sample(start + Duration::from_millis(i * 20));
        }
        assert!((rate.fps - 50.).abs() < 1e-8);
        assert!((rate.milliseconds - 20.).abs() < 1e-8);
    }
}
