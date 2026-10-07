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
        pub sketches: Vec<(Uuid, Design)>,
        pub sketch: Option<Uuid>,
        pub mesh: Option<ffi::Mesh>,
        pub features: Vec<Uuid>,
    }
    pub fn evaluate(
        design: &Design,
        cancelled: impl Fn() -> bool,
    ) -> Result<EvaluatedModel, String> {
        evaluate_internal(
            design,
            cancelled,
            &mut crate::evaluation::cache::EvaluationCache::default(),
            false,
            None,
        )
    }
    pub fn evaluate_cached(
        design: &Design,
        cancelled: impl Fn() -> bool,
        cache: &mut crate::evaluation::cache::EvaluationCache,
    ) -> Result<EvaluatedModel, String> {
        evaluate_internal(design, cancelled, cache, true, None)
    }
    /// Cancellable native evaluation; token callbacks are safe on OCCT worker threads.
    pub fn evaluate_cancellable(
        design: &Design,
        token: &crate::kernel::cancellation::EvaluationCancellation,
        cache: &mut crate::evaluation::cache::EvaluationCache,
    ) -> Result<EvaluatedModel, String> {
        evaluate_internal(design, || token.is_cancelled(), cache, true, Some(token))
    }
    /// Evaluate from scratch while retaining exact native topology for exchange.
    pub(crate) fn evaluate_for_export(
        design: &Design,
    ) -> Result<(EvaluatedModel, crate::evaluation::cache::EvaluationCache), String> {
        let mut cache = crate::evaluation::cache::EvaluationCache::default();
        let model = evaluate_internal(design, || false, &mut cache, true, None)?;
        if model.sketches.iter().any(|(id, _)| {
            cache
                .sketches
                .get(id)
                .is_some_and(|(_, s)| !s.conflicts.is_empty())
        }) || !model.solution.conflicts.is_empty()
        {
            return Err("Resolve sketch conflicts before STEP export".into());
        }
        Ok((model, cache))
    }
    fn evaluate_internal(
        design: &Design,
        cancelled: impl Fn() -> bool,
        cache: &mut crate::evaluation::cache::EvaluationCache,
        reuse: bool,
        token: Option<&crate::kernel::cancellation::EvaluationCancellation>,
    ) -> Result<EvaluatedModel, String> {
        design.validate()?;
        let mut design = design.clone();
        design.sync_construction();
        design.validate()?;
        let parameters = expression::evaluate(&design)?;
        cache.reused_sketches = 0;
        cache.reused_features = 0;
        cache
            .sketches
            .retain(|id, _| design.construction.iter().any(|f| f.id == *id));
        let mut solutions = HashMap::new();
        let mut sketches = Vec::new();
        for sketch in design
            .construction
            .iter()
            .filter(|f| matches!(f.kind, crate::document::schema::ConstructionKind::Sketch))
        {
            if cancelled() {
                return Err("Evaluation superseded".into());
            }
            let input = design.sketch_input(sketch.id)?;
            let mut semantic_input = input.clone();
            semantic_input.parameters.clear();
            let values: Vec<_> = input
                .constraints
                .iter()
                .filter_map(|c| c.kind.parameter())
                .map(|id| (id, parameters[&id]))
                .collect();
            let key = crate::evaluation::cache::key(&(semantic_input, values))?;
            let s = if let Some((_, solution)) = cache
                .sketches
                .get(&sketch.id)
                .filter(|(stored, _)| *stored == key)
            {
                cache.reused_sketches += 1;
                solution.clone()
            } else {
                let solution = nonlinear::solve_cancellable(&input, &parameters, &cancelled)
                    .map_err(|e| format!("{}: {e}", sketch.name))?;
                cache.sketches.insert(sketch.id, (key, solution.clone()));
                solution
            };
            let mut display = input;
            for (point, xy) in display.points.iter_mut().zip(&s.points) {
                point.xy = *xy;
            }
            sketches.push((sketch.id, display));
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
        let ids: Vec<_> = features
            .iter()
            .map(|f| f.id)
            .chain(design.create_features.iter().map(|f| f.id))
            .collect();
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
                    edges.push(e.native(wire as u32));
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
                identity: feature.id.to_string(),
                edge_start: start,
                edge_count: edges.len() as u32 - start,
                depth: parameters[&feature.depth],
                operation: match feature.operation {
                    ExtrudeOperation::NewBody => 0,
                    ExtrudeOperation::Join => 1,
                    ExtrudeOperation::Cut => 2,
                    ExtrudeOperation::CutNewBody => 3,
                },
                target: feature.target.map(&index).transpose()?.unwrap_or(-1),
                support,
                producer,
                role,
            });
        }
        let mut creates = vec![];
        for feature in &design.create_features {
            if cancelled() {
                return Err("Evaluation superseded".into());
            }
            let values: Vec<_> = feature.parameters.iter().map(|id| parameters[id]).collect();
            feature
                .kind
                .validate_values(&values)
                .map_err(|e| format!("{}: {e}", feature.name))?;
            let mut add_profile =
                |sketch: Option<Uuid>, boundary: &[Uuid]| -> Result<(u32, u32), String> {
                    let start = edges.len() as u32;
                    if let Some(id) = sketch {
                        if !matches!(design.sketch_plane(id)?, SketchPlane::Xy) {
                            return Err("Create profiles currently require an XY sketch".into());
                        }
                        let input = design.sketch_input(id)?;
                        let solved = solutions.get(&id).ok_or("Missing profile solution")?;
                        if !solved.conflicts.is_empty() {
                            return Err("Resolve profile sketch conflicts first".into());
                        }
                        let region = regions::select(&input, &solved.points, boundary)?;
                        for (wire, w) in region.wires.iter().enumerate() {
                            for e in w {
                                edges.push(e.native(wire as u32));
                            }
                        }
                    }
                    Ok((start, edges.len() as u32 - start))
                };
            let (edge_start, edge_count) = add_profile(feature.sketch, &feature.boundary)?;
            let (second_start, second_count) = add_profile(feature.second_sketch, &[])?;
            creates.push(ffi::CreateStep {
                identity: feature.id.to_string(),
                kind: feature.kind as u32,
                edge_start,
                edge_count,
                second_start,
                second_count,
                target: feature.target.map(&index).transpose()?.unwrap_or(-1),
                second_target: feature.second_target.map(&index).transpose()?.unwrap_or(-1),
                values,
            });
        }
        let mut planes = vec![];
        for sketch in design
            .construction
            .iter()
            .filter(|f| matches!(f.kind, crate::document::schema::ConstructionKind::Sketch))
        {
            let (support, producer, role) = match design.sketch_plane(sketch.id)? {
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
            planes.push(ffi::FaceRequest {
                support,
                producer,
                role,
            });
        }
        let mesh = if steps.is_empty() && creates.is_empty() {
            cache.mesh = None;
            cache.native = ffi::new_model_cache();
            None
        } else {
            let edits: Vec<_> = design
                .solid_edits
                .iter()
                .map(|e| {
                    let mut values = e.values;
                    for (i, id) in e.parameters.iter().enumerate() {
                        if let Some(id) = id {
                            values[i] =
                                *parameters.get(id).ok_or("Missing solid edit parameter")?;
                        }
                    }
                    let mut resolved = e.clone();
                    resolved.values = values;
                    resolved.validate()?;
                    Ok(ffi::ModifyStep {
                        identity: e.id.to_string(),
                        face_reference: e.face_reference.clone().unwrap_or_default(),
                        tool_reference: e.tool_reference.clone().unwrap_or_default(),
                        kind: e.kind.code(),
                        target: index(e.target)? as u32,
                        tool: e.tool.map(&index).transpose()?.unwrap_or(-1),
                        face: e.face,
                        a: values[0],
                        b: values[1],
                        c: values[2],
                        d: values[3],
                        copy: e.copy,
                        mode: e.mode,
                    })
                })
                .collect::<Result<_, String>>()?;
            Some(if reuse {
                let keys = crate::evaluation::invalidation::feature_keys(
                    &edges, &steps, &creates, &edits,
                )?;
                let plane_keys: Vec<_> = planes
                    .iter()
                    .map(|p| (p.support, p.producer, p.role))
                    .collect();
                let mesh_key = crate::evaluation::cache::key(&(&keys, plane_keys))?;
                if let Some((_, mesh)) = cache.mesh.as_ref().filter(|(key, _)| *key == mesh_key) {
                    cache.reused_features = keys.len();
                    mesh.clone()
                } else {
                    if cancelled() {
                        return Err("Evaluation superseded".into());
                    }
                    let mesh = if let Some(token) = token {
                        ffi::evaluate_cancellable_model(
                            cache.native.pin_mut(),
                            &edges,
                            &steps,
                            &planes,
                            &edits,
                            &creates,
                            &keys,
                            token,
                        )
                    } else {
                        ffi::evaluate_cached_model(
                            cache.native.pin_mut(),
                            &edges,
                            &steps,
                            &planes,
                            &edits,
                            &creates,
                            &keys,
                        )
                    }
                    .map_err(|e| e.to_string())?;
                    if cancelled() {
                        return Err("Evaluation superseded".into());
                    }
                    cache.reused_features = cache.native.reused_features();
                    cache.mesh = Some((mesh_key, mesh.clone()));
                    mesh
                }
            } else {
                ffi::evaluate_complete_model(&edges, &steps, &planes, &edits, &creates)
                    .map_err(|e| e.to_string())?
            })
        };
        if cancelled() {
            return Err("Evaluation superseded".into());
        }
        Ok(EvaluatedModel {
            sketches,
            sketch: design.current_sketch_id(),
            solution,
            mesh,
            features: ids,
        })
    }
}
#[cfg(all(feature = "solver", feature = "kernel"))]
pub use implementation::*;
