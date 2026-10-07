//! Latest-request background solve and OCCT regeneration, isolated from GPUI entities.
//! Exports Worker/ResultSnapshot. Input revisions reject stale results; one pending
//! snapshot replaces earlier queued edits; one result slot bounds completed output. The worker owns all native evaluation state.
#[cfg(all(feature = "solver", feature = "kernel"))]
mod implementation {
    use crate::{
        document::schema::Design, kernel::bridge::ffi, parameters::expression, sketch::profiles,
        solver::nonlinear,
    };
    use std::sync::{Arc, Condvar, Mutex};
    struct Pending {
        snapshot: Option<(u64, Design)>,
        stop: bool,
    }
    pub struct ResultSnapshot {
        pub revision: u64,
        pub solution: Result<nonlinear::Solution, String>,
        pub mesh: Result<Option<ffi::Mesh>, String>,
    }
    pub struct Worker {
        pending: Arc<(Mutex<Pending>, Condvar)>,
        results: Arc<Mutex<Option<ResultSnapshot>>>,
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
                        let parameters = expression::evaluate(&design);
                        let solution = parameters.as_ref().map_err(Clone::clone).and_then(|p| {
                            nonlinear::solve_cancellable(&design, p, || {
                                let pending = state.0.lock().unwrap();
                                pending.stop
                                    || pending
                                        .snapshot
                                        .as_ref()
                                        .is_some_and(|(r, _)| *r > revision)
                            })
                        });
                        let mesh = match (&parameters, &solution) {
                            (_, Err(e)) | (Err(e), _) => Err(e.clone()),
                            (Ok(parameters), Ok(solution)) => {
                                if !solution.conflicts.is_empty() {
                                    Err(format!(
                                        "{} conflicting constraints",
                                        solution.conflicts.len()
                                    ))
                                } else if let Some(extrusion) = &design.extrusion {
                                    profiles::closed_profile(&design, &solution.points).and_then(
                                        |p| {
                                            let points: Vec<_> = p
                                                .into_iter()
                                                .map(|p| ffi::Point2 { x: p[0], y: p[1] })
                                                .collect();
                                            ffi::extrude(&points, parameters[&extrusion.depth])
                                                .map(Some)
                                                .map_err(|e| e.to_string())
                                        },
                                    )
                                } else {
                                    Ok(None)
                                }
                            }
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
}
#[cfg(all(feature = "solver", feature = "kernel"))]
pub use implementation::*;
