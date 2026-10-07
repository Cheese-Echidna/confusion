//! Bounded snapshot scheduling independent of rendering and undo history.
use std::time::{Duration, Instant};
#[derive(Default)]
pub struct RecoverySchedule {
    observed: Option<blake3::Hash>,
    written: Option<blake3::Hash>,
    changed: Option<Instant>,
    last_write: Option<Instant>,
    pending_since: Option<Instant>,
}
impl RecoverySchedule {
    pub fn observe(&mut self, fingerprint: blake3::Hash, now: Instant) -> bool {
        if self.observed != Some(fingerprint) {
            self.observed = Some(fingerprint);
            self.changed = Some(now);
            if self.pending_since.is_none() {
                self.pending_since = Some(now);
            }
        }
        self.written != Some(fingerprint)
            && (self
                .changed
                .is_some_and(|t| now.duration_since(t) >= Duration::from_secs(2))
                || self
                    .pending_since
                    .is_some_and(|t| now.duration_since(t) >= Duration::from_secs(30)))
            && self
                .last_write
                .is_none_or(|t| now.duration_since(t) >= Duration::from_secs(10))
    }
    pub fn submitted(&mut self, now: Instant) {
        self.written = self.observed;
        self.last_write = Some(now);
        self.pending_since = None;
    }
    pub fn failed(&mut self) {
        self.written = None;
    }
}
