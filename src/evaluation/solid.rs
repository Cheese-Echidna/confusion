//! Immutable sketch solves and dependency-ordered exact solid evaluation.
#[cfg(all(feature = "solver", feature = "kernel"))]
mod implementation {
    use crate::{
        document::{
            model::{CapRole, ExtrudeOperation, SketchPlane},
            schema::Design,
        },
        kernel::bridge::ffi,
        parameters::expression,
        sketch::regions,
        solver::nonlinear,
    };
    use std::collections::HashMap;
    use uuid::Uuid;
    pub struct EvaluatedModel {
        pub solution: nonlinear::Solution,
        pub mesh: Option<ffi::Mesh>,
        pub features: Vec<Uuid>,
    }
    pub fn evaluate(
        design: &Design,
        cancelled: impl Fn() -> bool,
    ) -> Result<EvaluatedModel, String> {
        design.validate()?;
        let mut design = design.clone();
        design.sync_construction();
        design.validate()?;
        let parameters = expression::evaluate(&design)?;
        let mut solutions = HashMap::new();
        for sketch in design
            .construction
            .iter()
            .filter(|f| matches!(f.kind, crate::document::schema::ConstructionKind::Sketch))
        {
            if cancelled() {
                return Err("Evaluation superseded".into());
            }
            let input = design.sketch_input(sketch.id)?;
            let s = nonlinear::solve_cancellable(&input, &parameters, &cancelled)
                .map_err(|e| format!("{}: {e}", sketch.name))?;
            solutions.insert(sketch.id, s);
        }
        let solution = if let Some(id) = design.current_sketch_id() {
            solutions
                .get(&id)
                .ok_or("Missing active sketch solution")?
                .clone()
        } else {
            nonlinear::solve_cancellable(&design, &parameters, &cancelled)?
        };
        let features = design.solid_features();
        let ids: Vec<_> = features.iter().map(|f| f.id).collect();
        let index = |id: Uuid| {
            ids.iter()
                .position(|i| *i == id)
                .map(|i| i as i32)
                .ok_or("Missing feature dependency".to_string())
        };
        let mut edges = vec![];
        let mut steps = vec![];
        for feature in &features {
            if cancelled() {
                return Err("Evaluation superseded".into());
            }
            let input = design.sketch_input(feature.sketch)?;
            let s = solutions
                .get(&feature.sketch)
                .ok_or("Missing feature sketch solution")?;
            if !s.conflicts.is_empty() {
                return Err(format!(
                    "{}: {} conflicting constraints",
                    feature.name,
                    s.conflicts.len()
                ));
            }
            let region = regions::select(&input, &s.points, &feature.boundary)
                .map_err(|e| format!("{}: {e}", feature.name))?;
            let start = edges.len() as u32;
            for (wire, w) in region.wires.iter().enumerate() {
                for e in w {
                    edges.push(ffi::ProfileEdge {
                        wire: wire as u32,
                        sx: e.start[0],
                        sy: e.start[1],
                        ex: e.end[0],
                        ey: e.end[1],
                        cx: e.center[0],
                        cy: e.center[1],
                        sweep: e.sweep,
                    });
                }
            }
            let (support, producer, role) = match design.sketch_plane(feature.sketch)? {
                SketchPlane::Xy => (-1, -1, 0),
                SketchPlane::Face {
                    support,
                    producer,
                    role,
                } => (
                    index(*support)?,
                    index(*producer)?,
                    if *role == CapRole::Start { 1 } else { 2 },
                ),
            };
            steps.push(ffi::ModelStep {
                edge_start: start,
                edge_count: edges.len() as u32 - start,
                depth: parameters[&feature.depth],
                operation: match feature.operation {
                    ExtrudeOperation::NewBody => 0,
                    ExtrudeOperation::Join => 1,
                    ExtrudeOperation::Cut => 2,
                },
                target: feature.target.map(&index).transpose()?.unwrap_or(-1),
                support,
                producer,
                role,
            });
        }
        let mut planes = vec![];
        for sketch in design
            .construction
            .iter()
            .filter(|f| matches!(f.kind, crate::document::schema::ConstructionKind::Sketch))
        {
            if let SketchPlane::Face {
                support,
                producer,
                role,
            } = design.sketch_plane(sketch.id)?
            {
                planes.push(ffi::FaceRequest {
                    support: index(*support)?,
                    producer: index(*producer)?,
                    role: if *role == CapRole::Start { 1 } else { 2 },
                });
            }
        }
        let mesh = if steps.is_empty() {
            None
        } else {
            Some(ffi::evaluate_model(&edges, &steps, &planes).map_err(|e| e.to_string())?)
        };
        if cancelled() {
            return Err("Evaluation superseded".into());
        }
        Ok(EvaluatedModel {
            solution,
            mesh,
            features: ids,
        })
    }
}
#[cfg(all(feature = "solver", feature = "kernel"))]
pub use implementation::*;
