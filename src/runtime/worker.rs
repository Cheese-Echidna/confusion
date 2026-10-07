//! Latest-request background solve and OCCT regeneration, isolated from GPUI entities.
//! Exports Worker/ResultSnapshot. Input revisions reject stale results; one pending
//! snapshot replaces earlier queued edits; one result slot bounds completed output. The worker owns all native evaluation state.
#[cfg(all(feature = "solver", feature = "kernel"))]
mod implementation {
    use crate::kernel::cancellation::EvaluationCancellation;
    use crate::{
        document::schema::Design, evaluation::solid, kernel::bridge::ffi, solver::nonlinear,
    };
    use std::sync::{Arc, Condvar, Mutex};
    struct Pending {
        active: Option<(u64, EvaluationCancellation)>,
        latest_revision: Option<u64>,
        snapshot: Option<(u64, Design)>,
        stop: bool,
    }
    pub struct ResultSnapshot {
        pub revision: u64,
        pub sketches: Vec<(uuid::Uuid, Design)>,
        pub sketch: Option<uuid::Uuid>,
        pub features: Vec<uuid::Uuid>,
        pub solution: Result<nonlinear::Solution, String>,
        pub mesh: Result<Option<ffi::Mesh>, String>,
    }
    pub struct Worker {
        pending: Arc<(Mutex<Pending>, Condvar)>,
        results: Arc<Mutex<Option<ResultSnapshot>>>,
    }
    fn recover_evaluation<T>(evaluate: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(evaluate)).unwrap_or_else(|panic| {
            let message = panic
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("unknown evaluator panic");
            Err(format!("Evaluation failed unexpectedly: {message}"))
        })
    }
    impl Default for Worker {
        fn default() -> Self {
            Self::new()
        }
    }
    impl Worker {
        pub fn new() -> Self {
            let pending = Arc::new((
                Mutex::new(Pending {
                    snapshot: None,
                    active: None,
                    latest_revision: None,
                    stop: false,
                }),
                Condvar::new(),
            ));
            let state = pending.clone();
            let results = Arc::new(Mutex::new(None));
            let output = results.clone();
            std::thread::Builder::new()
                .name("confusion-evaluation".into())
                .spawn(move || {
                    let mut cache = crate::evaluation::cache::EvaluationCache::default();
                    loop {
                        let (revision, design, token) = {
                            let (lock, wake) = &*state;
                            let mut p = lock.lock().unwrap();
                            while p.snapshot.is_none() && !p.stop {
                                p = wake.wait(p).unwrap();
                            }
                            if p.stop {
                                break;
                            }
                            let (revision, design) = p.snapshot.take().unwrap();
                            let token = EvaluationCancellation::default();
                            p.active = Some((revision, token.clone()));
                            (revision, design, token)
                        };
                        let evaluated = recover_evaluation(|| {
                            solid::evaluate_cancellable(&design, &token, &mut cache)
                        });
                        let (solution, mesh, features, sketch, sketches) = match evaluated {
                            Ok(model) => (
                                Ok(model.solution),
                                Ok(model.mesh),
                                model.features,
                                model.sketch,
                                model.sketches,
                            ),
                            Err(error) => (Err(error.clone()), Err(error), vec![], None, vec![]),
                        };
                        // Keep the pending lock through publication so a newer submit
                        // cannot race between the revision check and the result write.
                        let mut pending = state.0.lock().unwrap();
                        pending.active = None;
                        if pending.stop
                            || token.is_cancelled()
                            || pending
                                .snapshot
                                .as_ref()
                                .is_some_and(|(r, _)| *r > revision)
                        {
                            continue;
                        }
                        *output.lock().unwrap() = Some(ResultSnapshot {
                            revision,
                            sketches,
                            features,
                            sketch,
                            solution,
                            mesh,
                        });
                    }
                })
                .expect("Could not start evaluation worker");
            Self { pending, results }
        }
        pub fn submit(&self, revision: u64, design: Design) {
            let mut p = self.pending.0.lock().unwrap();
            if p.stop || p.latest_revision.is_some_and(|r| revision <= r) {
                return;
            }
            p.latest_revision = Some(revision);
            if let Some((_, token)) = &p.active {
                token.cancel();
            }
            // Remove a completed obsolete result immediately; publication uses
            // the same lock order (pending, results).
            *self.results.lock().unwrap() = None;
            p.snapshot = Some((revision, design));
            self.pending.1.notify_one();
        }
        /// Abandon a candidate without publishing it or replacing canonical intent.
        pub fn cancel(&self) {
            let mut pending = self.pending.0.lock().unwrap();
            if let Some((_, token)) = &pending.active {
                token.cancel();
            }
            pending.snapshot = None;
            *self.results.lock().unwrap() = None;
        }
        pub fn poll(&self) -> Option<ResultSnapshot> {
            self.results.lock().unwrap().take()
        }
    }
    impl Drop for Worker {
        fn drop(&mut self) {
            let mut pending = self.pending.0.lock().unwrap();
            pending.stop = true;
            if let Some((_, token)) = &pending.active {
                token.cancel();
            }
            self.pending.1.notify_one();
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::document::{
            model::{CapRole, ExtrudeFeature, ExtrudeOperation, SketchPlane},
            schema::Extrusion,
        };
        use std::time::{Duration, Instant};
        use uuid::Uuid;
        fn await_result(worker: &Worker) -> ResultSnapshot {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                if let Some(result) = worker.poll() {
                    return result;
                }
                assert!(Instant::now() < deadline, "Evaluation did not complete");
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        #[test]
        fn failed_cut_finishes_and_worker_accepts_a_repaired_cut() {
            let mut d = Design::default();
            d.rectangle([0., 0.], [0.08, 0.05]);
            let base = Uuid::new_v4();
            let depth = d.parameter("thickness", "10 mm".into());
            d.extrusion = Some(Extrusion {
                id: base,
                depth,
                boundary: vec![],
            });
            d.sync_construction();
            let sketch = d
                .create_sketch(SketchPlane::Face {
                    support: base,
                    producer: base,
                    role: CapRole::End,
                })
                .unwrap();
            d.rectangle([0.10, 0.10], [0.12, 0.12]);
            let depth = d.parameter("pocketDepth", "3 mm".into());
            d.features.push(ExtrudeFeature {
                id: Uuid::new_v4(),
                name: "Pocket".into(),
                sketch,
                boundary: vec![],
                depth,
                operation: ExtrudeOperation::Cut,
                target: Some(base),
            });
            d.sync_construction();
            let worker = Worker::new();
            worker.submit(1, d.clone());
            let failed = await_result(&worker);
            assert_eq!(failed.revision, 1);
            assert!(failed.solution.is_err());
            assert!(failed.mesh.is_err());
            for point in &mut d.points {
                point.xy[0] -= 0.09;
                point.xy[1] -= 0.09;
            }
            for constraint in &mut d.constraints {
                if let crate::document::schema::ConstraintKind::Fixed { xy, .. } =
                    &mut constraint.kind
                {
                    xy[0] -= 0.09;
                    xy[1] -= 0.09;
                }
            }
            worker.submit(2, d);
            let repaired = await_result(&worker);
            assert_eq!(repaired.revision, 2);
            let mesh = repaired.mesh.unwrap().unwrap();
            assert!((mesh.volume - (0.08 * 0.05 * 0.01 - 0.02 * 0.02 * 0.003)).abs() < 1e-11);
        }
        fn design(pockets: usize) -> Design {
            let mut d = Design::default();
            d.rectangle([0., 0.], [0.08, 0.05]);
            let depth = d.parameter("thickness", "10 mm".into());
            let base = uuid::Uuid::new_v4();
            d.extrusion = Some(Extrusion {
                id: base,
                depth,
                boundary: vec![],
            });
            d.sync_construction();
            let mut target = base;
            for n in 0..pockets {
                let sketch = d
                    .create_sketch(SketchPlane::Face {
                        support: target,
                        producer: base,
                        role: CapRole::End,
                    })
                    .unwrap();
                let x = 0.004 + (n % 8) as f64 * 0.009;
                let y = 0.004 + (n / 8) as f64 * 0.012;
                d.rectangle([x, y], [x + 0.005, y + 0.007]);
                let boundary = crate::sketch::regions::select(
                    &d,
                    &d.points.iter().map(|p| p.xy).collect::<Vec<_>>(),
                    &[],
                )
                .unwrap()
                .boundary;
                let depth = d.parameter(&format!("pocket{n}"), "3 mm".into());
                let id = uuid::Uuid::new_v4();
                d.features.push(ExtrudeFeature {
                    id,
                    name: format!("Pocket {n}"),
                    sketch,
                    boundary,
                    depth,
                    operation: ExtrudeOperation::Cut,
                    target: Some(target),
                });
                d.sync_construction();
                target = id;
            }
            d
        }
        fn wait_for_native(worker: &Worker) -> EvaluationCancellation {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                if let Some((_, token)) = &worker.pending.0.lock().unwrap().active
                    && token.native_polls() > 0
                {
                    return token.clone();
                }
                assert!(
                    Instant::now() < deadline,
                    "Worker did not enter native evaluation"
                );
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        #[test]
        fn newer_revision_cancels_active_native_evaluation_and_publishes_only_latest() {
            let worker = Worker::new();
            worker.submit(1, design(24));
            let obsolete = wait_for_native(&worker);
            worker.submit(2, design(0));
            worker.submit(1, design(1)); // stale submissions cannot replace the pending edit
            assert!(obsolete.is_cancelled());
            let result = await_result(&worker);
            assert_eq!(result.revision, 2);
            assert_eq!(result.features.len(), 1);
            let mesh = result.mesh.unwrap().unwrap();
            assert!((mesh.volume - 0.08 * 0.05 * 0.01).abs() < 1e-12);
            // Older input cannot replace an already completed revision either.
            worker.submit(1, design(0));
            assert!(worker.poll().is_none());
            worker.submit(3, design(1));
            assert_eq!(await_result(&worker).revision, 3);
        }
        #[test]
        fn cancelling_candidate_discards_it_and_allows_a_new_request() {
            let worker = Worker::new();
            worker.submit(1, design(24));
            let token = wait_for_native(&worker);
            worker.cancel();
            assert!(token.is_cancelled());
            assert!(worker.poll().is_none());
            worker.submit(2, design(0));
            let result = await_result(&worker);
            assert_eq!(result.revision, 2);
            assert!(result.mesh.unwrap().is_some());
        }
        #[test]
        fn dropping_worker_cancels_active_native_evaluation() {
            let worker = Worker::new();
            worker.submit(1, design(24));
            let token = wait_for_native(&worker);
            drop(worker);
            assert!(token.is_cancelled());
        }
        #[test]
        fn evaluator_panic_becomes_an_error_without_unwinding_the_worker_loop() {
            let failed = recover_evaluation::<()>(|| panic!("solver failure"));
            assert!(failed.unwrap_err().contains("solver failure"));
            assert_eq!(recover_evaluation(|| Ok(42)).unwrap(), 42);
        }
    }
}
#[cfg(all(feature = "solver", feature = "kernel"))]
pub use implementation::*;
