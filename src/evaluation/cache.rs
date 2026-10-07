//! Disposable, worker-owned evaluation results. Never serialized into a design.
#[cfg(all(feature = "solver", feature = "kernel"))]
use crate::{kernel::bridge::ffi, solver::nonlinear::Solution};
#[cfg(all(feature = "solver", feature = "kernel"))]
use std::collections::HashMap;

#[cfg(all(feature = "solver", feature = "kernel"))]
pub struct EvaluationCache {
    pub(crate) sketches: HashMap<uuid::Uuid, (String, Solution)>,
    pub(crate) native: cxx::UniquePtr<ffi::ModelCache>,
    pub reused_sketches: usize,
    pub(crate) reused_features: usize,
    pub(crate) mesh: Option<(String, ffi::Mesh)>,
}
#[cfg(all(feature = "solver", feature = "kernel"))]
impl Default for EvaluationCache {
    fn default() -> Self {
        Self {
            sketches: HashMap::new(),
            native: ffi::new_model_cache(),
            reused_sketches: 0,
            reused_features: 0,
            mesh: None,
        }
    }
}
#[cfg(all(feature = "solver", feature = "kernel"))]
impl EvaluationCache {
    pub fn reused_features(&self) -> usize {
        self.reused_features
    }
}

/// Hash only semantic inputs. The cache is process-local, so backend versions and
/// tolerances cannot change during its lifetime.
#[cfg(all(feature = "solver", feature = "kernel"))]
pub(crate) fn key(value: &impl serde::Serialize) -> Result<String, String> {
    Ok(
        blake3::hash(&serde_json::to_vec(value).map_err(|e| e.to_string())?)
            .to_hex()
            .to_string(),
    )
}
