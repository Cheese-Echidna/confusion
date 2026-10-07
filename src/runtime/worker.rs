//! Latest-request background solve and OCCT regeneration, isolated from GPUI entities.
//! Exports Worker/ResultSnapshot. Input revisions reject stale results; one pending
//! snapshot replaces earlier queued edits; one result slot bounds completed output. The worker owns all native evaluation state.
#[cfg(all(feature = "solver", feature = "kernel"))]
mod implementation {
    use crate::{
        document::schema::Design, evaluation::solid, kernel::bridge::ffi, solver::nonlinear,
    };
    use std::sync::{Arc, Condvar, Mutex};
    struct Pending {
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
                        let (revision, design) = {
                            let (lock, wake) = &*state;
                            let mut p = lock.lock().unwrap();
                            while p.snapshot.is_none() && !p.stop {
                                p = wake.wait(p).unwrap();
                            }
                            if p.stop {
                                break;
                            }
                            p.snapshot.take().unwrap()
                        };
                        let evaluated = recover_evaluation(|| {
                            solid::evaluate_cached(
                                &design,
                                || {
                                    let pending = state.0.lock().unwrap();
                                    pending.stop
                                        || pending
                                            .snapshot
                                            .as_ref()
                                            .is_some_and(|(r, _)| *r > revision)
                                },
                                &mut cache,
                            )
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
                        // Drop results already superseded while a native operation was running.
                        if state
                            .0
                            .lock()
                            .unwrap()
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
            p.snapshot = Some((revision, design));
            self.pending.1.notify_one();
        }
        pub fn poll(&self) -> Option<ResultSnapshot> {
            self.results.lock().unwrap().take()
        }
    }
    impl Drop for Worker {
        fn drop(&mut self) {
            self.pending.0.lock().unwrap().stop = true;
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
