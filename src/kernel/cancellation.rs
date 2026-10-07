//! Thread-safe cancellation shared by the UI, worker, and OCCT progress callbacks.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Default)]
pub struct EvaluationCancellation {
    cancelled: Arc<AtomicBool>,
    #[cfg(test)]
    native_polls: Arc<std::sync::atomic::AtomicUsize>,
    #[cfg(test)]
    cancel_after_native_poll: Arc<std::sync::atomic::AtomicUsize>,
}
impl EvaluationCancellation {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
    #[cfg(test)]
    pub(crate) fn native_polls(&self) -> usize {
        self.native_polls.load(Ordering::Relaxed)
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}
// CXX callback: no locks, allocation, or panicking code. OCCT may call concurrently.
pub(crate) fn evaluation_cancelled(token: &EvaluationCancellation) -> bool {
    #[cfg(test)]
    {
        let poll = token.native_polls.fetch_add(1, Ordering::Relaxed) + 1;
        let limit = token.cancel_after_native_poll.load(Ordering::Relaxed);
        if limit != 0 && poll >= limit {
            token.cancel();
        }
    }
    token.is_cancelled()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bridge::ffi;
    fn rectangle(edges: &mut Vec<ffi::ProfileEdge>, x: f64, y: f64, w: f64, h: f64) {
        let points = [[x, y], [x + w, y], [x + w, y + h], [x, y + h]];
        for i in 0..4 {
            let a = points[i];
            let b = points[(i + 1) % 4];
            edges.push(ffi::ProfileEdge {
                wire: 0,
                sx: a[0],
                sy: a[1],
                ex: b[0],
                ey: b[1],
                cx: 0.,
                cy: 0.,
                sweep: 0.,
            });
        }
    }
    #[test]
    fn occt_progress_interrupts_boolean_and_cache_recovers() {
        let mut edges = Vec::new();
        rectangle(&mut edges, 0., 0., 0.08, 0.05);
        rectangle(&mut edges, 0.01, 0.01, 0.02, 0.01);
        let steps = vec![
            ffi::ModelStep {
                edge_start: 0,
                edge_count: 4,
                depth: 0.01,
                operation: 0,
                target: -1,
                support: -1,
                producer: -1,
                role: 0,
            },
            ffi::ModelStep {
                edge_start: 4,
                edge_count: 4,
                depth: 0.003,
                operation: 2,
                target: 0,
                support: 0,
                producer: 0,
                role: 2,
            },
        ];
        let keys = vec!["base".to_string(), "pocket".to_string()];
        let mut cache = ffi::new_model_cache();
        let token = EvaluationCancellation::default();
        // Entry/feature/build checks account for four polls; subsequent polls
        // come from OCCT's Boolean progress indicator.
        token.cancel_after_native_poll.store(10, Ordering::Relaxed);
        let error = ffi::evaluate_cancellable_model(
            cache.pin_mut(),
            &edges,
            &steps,
            &[],
            &[],
            &[],
            &keys,
            &token,
        )
        .err()
        .unwrap();
        assert!(
            error.to_string().contains("Evaluation superseded"),
            "{error}"
        );
        assert!(token.native_polls() >= 10);
        assert!(token.is_cancelled());
        let retry = EvaluationCancellation::default();
        let mesh = ffi::evaluate_cancellable_model(
            cache.pin_mut(),
            &edges,
            &steps,
            &[],
            &[],
            &[],
            &keys,
            &retry,
        )
        .unwrap();
        assert_eq!(cache.reused_features(), 1);
        let expected = 0.08 * 0.05 * 0.01 - 0.02 * 0.01 * 0.003;
        assert!((mesh.volume - expected).abs() < 1e-12);
        // The borrowed native scope must have cleared after the cancelled call.
        let fresh = ffi::evaluate_complete_model(&edges, &steps, &[], &[], &[]).unwrap();
        assert!((fresh.volume - mesh.volume).abs() < 1e-12);
    }
    #[test]
    fn cancellation_is_shared_between_threads() {
        let token = EvaluationCancellation::default();
        let other = token.clone();
        std::thread::spawn(move || other.cancel()).join().unwrap();
        assert!(token.is_cancelled());
    }
}
