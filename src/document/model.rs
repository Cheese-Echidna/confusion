//! Current sketch ownership, semantic cap attachments, and solid dependency definitions.
use super::schema::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapRole {
    Start,
    End,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SketchPlane {
    #[default]
    Xy,
    NamedFace {
        support: Uuid,
        reference: String,
    },
    /// Resolve the producer's cap in the support feature's evaluated output.
    Face {
        support: Uuid,
        producer: Uuid,
        role: CapRole,
    },
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtrudeOperation {
    #[default]
    NewBody,
    Join,
    Cut,
    CutNewBody,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SketchGeometry {
    pub points: Vec<Point>,
    pub lines: Vec<Line>,
    pub circles: Vec<Circle>,
    pub ellipses: Vec<Ellipse>,
    pub splines: Vec<Spline>,
    pub construction_geometry: Vec<Uuid>,
    pub dimension_positions: std::collections::HashMap<Uuid, [f64; 2]>,
    pub driven_dimensions: Vec<Constraint>,
    pub constraints: Vec<Constraint>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SketchDefinition {
    pub id: Uuid,
    pub name: String,
    pub plane: SketchPlane,
    pub geometry: SketchGeometry,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtrudeFeature {
    pub id: Uuid,
    pub name: String,
    pub sketch: Uuid,
    pub boundary: Vec<Uuid>,
    pub depth: Uuid,
    pub operation: ExtrudeOperation,
    pub target: Option<Uuid>,
}
impl Design {
    /// Bodies available immediately before a construction event, including edits.
    pub(crate) fn bodies_before(&self, id: Uuid) -> std::collections::HashSet<Uuid> {
        let mut timeline = self.clone();
        timeline.sync_construction();
        let features = timeline.solid_features();
        let mut bodies = std::collections::HashSet::new();
        for event in &timeline.construction {
            if event.id == id {
                break;
            }
            if let Some(f) = features.iter().find(|f| f.id == event.id) {
                if let Some(target) = f.target {
                    bodies.remove(&target);
                }
                bodies.insert(f.id);
            } else if let Some(f) = timeline.create_features.iter().find(|f| f.id == event.id) {
                if f.kind.consumes_body() {
                    if let Some(target) = f.target {
                        bodies.remove(&target);
                    }
                    if let Some(target) = f.second_target {
                        bodies.remove(&target);
                    }
                }
                bodies.insert(f.id);
            } else if let Some(e) = timeline.solid_edits.iter().find(|e| e.id == event.id) {
                use crate::model::modify::ModifyKind;
                if matches!(e.kind, ModifyKind::Delete | ModifyKind::Remove) && e.face == 0 {
                    bodies.remove(&e.target);
                }
                if e.kind == ModifyKind::Combine && !e.copy {
                    if let Some(tool) = e.tool {
                        bodies.remove(&tool);
                    }
                }
            }
        }
        bodies
    }
    fn produced_before(&self, id: Uuid) -> std::collections::HashSet<Uuid> {
        let mut d = self.clone();
        d.sync_construction();
        d.construction
            .iter()
            .take_while(|f| f.id != id)
            .filter(|f| {
                matches!(
                    f.kind,
                    ConstructionKind::Extrude { .. } | ConstructionKind::Create
                )
            })
            .map(|f| f.id)
            .collect()
    }
    pub fn primary_sketch_id(&self) -> Option<Uuid> {
        self.construction
            .iter()
            .find(|f| matches!(f.kind, ConstructionKind::Sketch))
            .map(|f| f.id)
    }
    pub fn current_sketch_id(&self) -> Option<Uuid> {
        self.active_sketch.or_else(|| self.primary_sketch_id())
    }
    fn geometry(&self) -> SketchGeometry {
        SketchGeometry {
            points: self.points.clone(),
            lines: self.lines.clone(),
            circles: self.circles.clone(),
            ellipses: self.ellipses.clone(),
            splines: self.splines.clone(),
            construction_geometry: self.construction_geometry.clone(),
            dimension_positions: self.dimension_positions.clone(),
            driven_dimensions: self.driven_dimensions.clone(),
            constraints: self.constraints.clone(),
        }
    }
    fn set_geometry(&mut self, g: SketchGeometry) {
        self.points = g.points;
        self.lines = g.lines;
        self.circles = g.circles;
        self.ellipses = g.ellipses;
        self.splines = g.splines;
        self.construction_geometry = g.construction_geometry;
        self.dimension_positions = g.dimension_positions;
        self.driven_dimensions = g.driven_dimensions;
        self.constraints = g.constraints;
    }
    /// Editing a different sketch exchanges its geometry, preserving a single copy of each sketch.
    pub fn activate_sketch(&mut self, id: Uuid) -> Result<(), String> {
        if self.current_sketch_id() == Some(id) {
            return Ok(());
        }
        let index = self
            .sketches
            .iter()
            .position(|s| s.id == id)
            .ok_or("Missing sketch")?;
        let current = self.ensure_sketch();
        let name = self
            .construction
            .iter()
            .find(|f| f.id == current)
            .ok_or("Missing active sketch construction")?
            .name
            .clone();
        let next = self.sketches.remove(index);
        self.sketches.push(SketchDefinition {
            id: current,
            name,
            plane: self.active_plane.clone(),
            geometry: self.geometry(),
        });
        self.set_geometry(next.geometry);
        self.active_plane = next.plane;
        self.active_sketch = Some(id);
        Ok(())
    }
    pub fn create_sketch(&mut self, plane: SketchPlane) -> Result<Uuid, String> {
        self.validate()?;
        if self.sketches.len() >= 63 {
            return Err("Document exceeds current sketch count limit".into());
        }
        let mut candidate = self.clone();
        candidate.sync_construction();
        candidate.ensure_sketch();
        let id = Uuid::new_v4();
        let name = format!("Sketch {}", candidate.sketches.len() + 2);
        candidate.sketches.push(SketchDefinition {
            id,
            name: name.clone(),
            plane,
            geometry: SketchGeometry::default(),
        });
        candidate.construction.push(ConstructionFeature {
            id,
            name,
            kind: ConstructionKind::Sketch,
        });
        candidate.activate_sketch(id)?;
        candidate.validate()?;
        *self = candidate;
        Ok(id)
    }
    /// Computational input contains only one sketch and shared parameters, never model state.
    pub fn sketch_input(&self, id: Uuid) -> Result<Design, String> {
        let geometry = if self.current_sketch_id() == Some(id) {
            self.geometry()
        } else {
            self.sketches
                .iter()
                .find(|s| s.id == id)
                .ok_or("Missing sketch")?
                .geometry
                .clone()
        };
        let mut d = Design {
            parameters: self.parameters.clone(),
            ..Design::default()
        };
        d.set_geometry(geometry);
        Ok(d)
    }
    pub fn sketch_plane(&self, id: Uuid) -> Result<&SketchPlane, String> {
        if self.current_sketch_id() == Some(id) {
            Ok(&self.active_plane)
        } else {
            self.sketches
                .iter()
                .find(|s| s.id == id)
                .map(|s| &s.plane)
                .ok_or("Missing sketch".into())
        }
    }
    pub fn solid_features(&self) -> Vec<ExtrudeFeature> {
        let mut result = vec![];
        if let (Some(e), Some(sketch)) = (&self.extrusion, self.primary_sketch_id()) {
            result.push(ExtrudeFeature {
                id: e.id,
                name: "Extrude 1".into(),
                sketch,
                boundary: e.boundary.clone(),
                depth: e.depth,
                operation: ExtrudeOperation::NewBody,
                target: None,
            });
        }
        result.extend(self.features.clone());
        result
    }
    pub fn latest_body_feature(&self) -> Option<Uuid> {
        let current = self.bodies_before(Uuid::nil());
        let mut d = self.clone();
        d.sync_construction();
        d.construction
            .iter()
            .rev()
            .find(|f| current.contains(&f.id))
            .map(|f| f.id)
    }
    pub fn validate_model(&self) -> Result<(), String> {
        self.validate_modifications()?;
        use std::collections::HashSet;
        if self.active_sketch.is_some()
            && !self.construction.iter().any(|f| {
                Some(f.id) == self.active_sketch && matches!(f.kind, ConstructionKind::Sketch)
            })
        {
            return Err("Missing active sketch".into());
        }
        if self.extrusion.is_some() && self.primary_sketch_id().is_none() {
            if !self.sketches.is_empty() || !self.features.is_empty() {
                return Err("Missing primary construction sketch".into());
            }
            let mut normalized = self.clone();
            normalized.sync_construction();
            return normalized.validate_model();
        }
        if self.sketches.len() > 63 || self.features.len() > 63 {
            return Err("Document exceeds current modeling limits".into());
        }
        let current = self.current_sketch_id();
        let mut sketch_ids = HashSet::new();
        if let Some(id) = current {
            sketch_ids.insert(id);
        }
        let mut entities: HashSet<_> = self
            .points
            .iter()
            .map(|p| p.id)
            .chain(self.lines.iter().map(|p| p.id))
            .chain(self.circles.iter().map(|p| p.id))
            .chain(self.ellipses.iter().map(|p| p.id))
            .chain(self.splines.iter().map(|p| p.id))
            .chain(self.constraints.iter().map(|p| p.id))
            .chain(self.driven_dimensions.iter().map(|p| p.id))
            .collect();
        for sketch in &self.sketches {
            if !self
                .construction
                .iter()
                .any(|f| f.id == sketch.id && matches!(f.kind, ConstructionKind::Sketch))
            {
                return Err("Missing sketch construction definition".into());
            }
            if !sketch_ids.insert(sketch.id) || sketch.name.is_empty() || sketch.name.len() > 128 {
                return Err("Duplicate or invalid sketch".into());
            }
            let input = self.sketch_input(sketch.id)?;
            input.validate()?;
            for id in input
                .points
                .iter()
                .map(|p| p.id)
                .chain(input.lines.iter().map(|p| p.id))
                .chain(input.circles.iter().map(|p| p.id))
                .chain(input.ellipses.iter().map(|p| p.id))
                .chain(input.splines.iter().map(|p| p.id))
                .chain(input.constraints.iter().map(|p| p.id))
                .chain(input.driven_dimensions.iter().map(|p| p.id))
            {
                if !entities.insert(id) {
                    return Err("Duplicate entity ID across sketches".into());
                }
            }
        }
        let features = self.solid_features();
        if self.extrusion.is_some() && features.is_empty() {
            return Err("Missing base extrusion sketch".into());
        }
        let mut available = HashSet::new();
        let mut consumed = HashSet::new();
        for feature in &features {
            let upstream = self.bodies_before(feature.id);
            if !sketch_ids.contains(&feature.sketch)
                || available.contains(&feature.id)
                || entities.contains(&feature.id)
                || sketch_ids.contains(&feature.id)
                || self.parameters.iter().any(|p| p.id == feature.id)
                || feature.name.is_empty()
                || feature.name.len() > 128
            {
                return Err("Invalid solid feature identity or sketch".into());
            }
            let depth = self
                .parameters
                .iter()
                .find(|p| p.id == feature.depth)
                .ok_or("Missing extrusion depth parameter")?;
            if depth.angular || depth.scalar {
                return Err("Extrusion depth must have length units".into());
            }
            match (feature.operation, feature.target) {
                (ExtrudeOperation::NewBody, None) => {}
                (
                    ExtrudeOperation::Join | ExtrudeOperation::Cut | ExtrudeOperation::CutNewBody,
                    Some(target),
                ) if upstream.contains(&target) && consumed.insert(target) => {}
                _ => return Err("Invalid solid target: choose a current upstream body".into()),
            }
            if let SketchPlane::Face {
                support, producer, ..
            } = self.sketch_plane(feature.sketch)?
                && (!self.produced_before(feature.id).contains(support)
                    || !self.produced_before(feature.id).contains(producer))
            {
                return Err("Sketch face attachment has a missing or cyclic dependency".into());
            }
            if let SketchPlane::NamedFace { support, reference } =
                self.sketch_plane(feature.sketch)?
            {
                if !self.produced_before(feature.id).contains(support)
                    || reference.is_empty()
                    || reference.len() > 4096
                {
                    return Err("Sketch face attachment has a missing or cyclic dependency".into());
                }
            }
            let input = self.sketch_input(feature.sketch)?;
            let curves = crate::sketch::entities::curve_ids(&input);
            let unique: HashSet<_> = feature.boundary.iter().collect();
            if unique.len() != feature.boundary.len()
                || feature.boundary.iter().any(|id| {
                    (!curves.contains(id) && !crate::sketch::regions::is_boundary_token(id))
                        || input.construction_geometry.contains(id)
                })
            {
                return Err("Missing or invalid extrusion boundary".into());
            }
            available.insert(feature.id);
        }
        available.extend(self.create_features.iter().map(|f| f.id));
        // Unconsumed sketches also need resolvable plane dependencies.
        for id in sketch_ids {
            if self.parameters.iter().any(|p| p.id == id) || entities.contains(&id) {
                return Err("Duplicate sketch ID".into());
            }
            if let SketchPlane::NamedFace { support, reference } = self.sketch_plane(id)? {
                if !self.bodies_before(id).contains(support)
                    || reference.is_empty()
                    || reference.len() > 4096
                {
                    return Err("Missing sketch support feature".into());
                }
            }
            if let SketchPlane::Face {
                support, producer, ..
            } = self.sketch_plane(id)?
                && (!self.produced_before(id).contains(support)
                    || !self.produced_before(id).contains(producer))
            {
                return Err("Missing sketch support feature".into());
            }
        }
        Ok(())
    }
}
