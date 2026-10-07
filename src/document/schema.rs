//! Current planar design intent, in metres, with stable IDs and parameter-driven constraints.
//! Exports Design, Point, Line, Constraint, Parameter and stable ConstructionFeature/Kind;
//! solver, persistence and UI share
//! these values. Meshes, solver results and edit history must never enter this schema.
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub id: Uuid,
    pub xy: [f64; 2],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    pub id: Uuid,
    pub ends: [Uuid; 2],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Parameter {
    #[serde(default)]
    pub angular: bool,
    pub id: Uuid,
    pub name: String,
    pub expression: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConstraintKind {
    ProjectedAlignment {
        points: [Uuid; 2],
        direction: [f64; 2],
    },
    HorizontalPoints {
        points: [Uuid; 2],
    },
    VerticalPoints {
        points: [Uuid; 2],
    },
    LineOffset {
        lines: [Uuid; 2],
        parameter: Uuid,
        side: f64,
    },
    OffsetRadius {
        circles: [Uuid; 2],
        parameter: Uuid,
    },
    Direction {
        line: Uuid,
        angle: f64,
    },
    ProjectedDistance {
        points: [Uuid; 2],
        parameter: Uuid,
        direction: [f64; 2],
    },
    Coincident {
        points: [Uuid; 2],
    },
    Parallel {
        lines: [Uuid; 2],
    },
    Perpendicular {
        lines: [Uuid; 2],
    },
    Equal {
        curves: [Uuid; 2],
    },
    Collinear {
        lines: [Uuid; 2],
    },
    Midpoint {
        point: Uuid,
        line: Uuid,
    },
    PointOnLine {
        point: Uuid,
        line: Uuid,
    },
    PointOnCircle {
        point: Uuid,
        circle: Uuid,
    },
    Concentric {
        circles: [Uuid; 2],
    },
    Tangent {
        curves: [Uuid; 2],
    },
    Symmetry {
        points: [Uuid; 2],
        axis: Uuid,
    },
    Distance {
        points: [Uuid; 2],
        parameter: Uuid,
    },
    LineDistance {
        lines: [Uuid; 2],
        parameter: Uuid,
    },
    Angle {
        lines: [Uuid; 2],
        parameter: Uuid,
    },
    Diameter {
        circle: Uuid,
        parameter: Uuid,
    },
    Radius {
        circle: Uuid,
        parameter: Uuid,
    },
    Horizontal {
        line: Uuid,
    },
    Vertical {
        line: Uuid,
    },
    Fixed {
        point: Uuid,
        xy: [f64; 2],
    },
    DistanceX {
        points: [Uuid; 2],
        parameter: Uuid,
    },
    DistanceY {
        points: [Uuid; 2],
        parameter: Uuid,
    },
    Length {
        line: Uuid,
        parameter: Uuid,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Constraint {
    pub id: Uuid,
    pub kind: ConstraintKind,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extrusion {
    /// Stable IDs of the outer wire; empty for legacy automatic selection.
    #[serde(default)]
    pub boundary: Vec<Uuid>,
    pub id: Uuid,
    pub depth: Uuid,
}
/// A circular curve uses a center and rim point; the rim carries its radius DOF.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Circle {
    pub id: Uuid,
    pub center: Uuid,
    pub rim: Uuid,
    /// An arc ends here, counterclockwise from rim; None represents a full circle.
    #[serde(default)]
    pub end: Option<Uuid>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ellipse {
    pub id: Uuid,
    pub center: Uuid,
    pub major: Uuid,
    pub minor: Uuid,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spline {
    pub id: Uuid,
    pub points: Vec<Uuid>,
    pub fit: bool,
}
/// Persistent current construction features, distinct from transient edit/undo history.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ConstructionKind {
    Sketch,
    Extrude { sketch: Uuid },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructionFeature {
    pub id: Uuid,
    pub name: String,
    pub kind: ConstructionKind,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Design {
    /// The active sketch uses the legacy geometry fields; inactive sketches are stored once here.
    #[serde(default)]
    pub sketches: Vec<super::model::SketchDefinition>,
    #[serde(default)]
    pub active_sketch: Option<Uuid>,
    #[serde(default)]
    pub active_plane: super::model::SketchPlane,
    #[serde(default)]
    pub features: Vec<super::model::ExtrudeFeature>,
    pub points: Vec<Point>,
    pub lines: Vec<Line>,
    #[serde(default)]
    pub circles: Vec<Circle>,
    #[serde(default)]
    pub ellipses: Vec<Ellipse>,
    #[serde(default)]
    pub splines: Vec<Spline>,
    #[serde(default)]
    pub construction_geometry: Vec<Uuid>,
    #[serde(default)]
    pub dimension_positions: std::collections::HashMap<Uuid, [f64; 2]>,
    #[serde(default)]
    pub driven_dimensions: Vec<Constraint>,
    pub constraints: Vec<Constraint>,
    pub parameters: Vec<Parameter>,
    pub extrusion: Option<Extrusion>,
    #[serde(default)]
    pub construction: Vec<ConstructionFeature>,
}
impl Design {
    pub fn ensure_sketch(&mut self) -> Uuid {
        if let Some(id) = self.active_sketch {
            return id;
        }
        if let Some(f) = self
            .construction
            .iter()
            .find(|f| matches!(f.kind, ConstructionKind::Sketch))
        {
            return f.id;
        }
        let id = Uuid::new_v4();
        self.construction.insert(
            0,
            ConstructionFeature {
                id,
                name: "Sketch 1".into(),
                kind: ConstructionKind::Sketch,
            },
        );
        id
    }
    /// Preserve construction dependencies while synchronizing current feature definitions.
    pub fn sync_construction(&mut self) {
        if !self.points.is_empty()
            || self.extrusion.is_some()
            || !self.features.is_empty()
            || !self.sketches.is_empty()
        {
            self.ensure_sketch();
        }
        if let Some(e) = &self.extrusion {
            let id = e.id;
            let Some(sketch) = self.primary_sketch_id() else {
                return;
            };
            if !self.construction.iter().any(|f| f.id == id) {
                self.construction.push(ConstructionFeature {
                    id,
                    name: "Extrude 1".into(),
                    kind: ConstructionKind::Extrude { sketch },
                });
            }
        }
        for e in &self.features {
            if !self.construction.iter().any(|f| f.id == e.id) {
                self.construction.push(ConstructionFeature {
                    id: e.id,
                    name: e.name.clone(),
                    kind: ConstructionKind::Extrude { sketch: e.sketch },
                });
            }
        }
        let ids: std::collections::HashSet<_> =
            self.solid_features().iter().map(|f| f.id).collect();
        self.construction
            .retain(|f| matches!(f.kind, ConstructionKind::Sketch) || ids.contains(&f.id));
    }
    /// Evaluate the dependency prefix at a construction marker, retaining canonical intent elsewhere.
    pub fn through_feature(&self, id: Uuid) -> Result<Self, String> {
        let index = self
            .construction
            .iter()
            .position(|f| f.id == id)
            .ok_or("Missing construction feature")?;
        let mut result = self.clone();
        let kept: std::collections::HashSet<_> =
            self.construction[..=index].iter().map(|f| f.id).collect();
        result.features.retain(|f| kept.contains(&f.id));
        if result
            .extrusion
            .as_ref()
            .is_some_and(|e| !kept.contains(&e.id))
        {
            result.extrusion = None;
        }
        result.sketches.retain(|s| kept.contains(&s.id));
        if result
            .current_sketch_id()
            .is_some_and(|id| !kept.contains(&id))
        {
            let sketch = self.construction[..=index]
                .iter()
                .rev()
                .find(|f| matches!(f.kind, ConstructionKind::Sketch))
                .ok_or("Missing sketch")?
                .id;
            result = self.clone();
            result.activate_sketch(sketch)?;
            result.features.retain(|f| kept.contains(&f.id));
            if result
                .extrusion
                .as_ref()
                .is_some_and(|e| !kept.contains(&e.id))
            {
                result.extrusion = None;
            }
            result.sketches.retain(|s| kept.contains(&s.id));
        }
        result.construction.truncate(index + 1);
        Ok(result)
    }

    pub fn parameter(&mut self, name: &str, expression: String) -> Uuid {
        if let Some(p) = self.parameters.iter_mut().find(|p| p.name == name) {
            p.expression = expression;
            return p.id;
        }
        let id = Uuid::new_v4();
        self.parameters.push(Parameter {
            angular: false,
            id,
            name: name.into(),
            expression,
        });
        id
    }
    pub fn point(&mut self, xy: [f64; 2]) -> Uuid {
        if let Some(p) = self
            .points
            .iter()
            .find(|p| (p.xy[0] - xy[0]).hypot(p.xy[1] - xy[1]) < 1e-9)
        {
            return p.id;
        }
        let id = Uuid::new_v4();
        self.points.push(Point { id, xy });
        id
    }
    pub fn line(&mut self, a: [f64; 2], b: [f64; 2]) -> Uuid {
        self.ensure_sketch();
        let ends = [self.point(a), self.point(b)];
        let id = Uuid::new_v4();
        self.lines.push(Line { id, ends });
        id
    }
    pub fn constrain(&mut self, kind: ConstraintKind) {
        self.constraints.push(Constraint {
            id: Uuid::new_v4(),
            kind,
        });
    }
    pub fn rectangle(&mut self, a: [f64; 2], b: [f64; 2]) {
        self.ensure_sketch();
        let low = [a[0].min(b[0]), a[1].min(b[1])];
        let b = [a[0].max(b[0]), a[1].max(b[1])];
        let a = low;
        let corners = [a, [b[0], a[1]], b, [a[0], b[1]]];
        let ids = corners.map(|p| self.point(p));
        for i in 0..4 {
            let id = Uuid::new_v4();
            self.lines.push(Line {
                id,
                ends: [ids[i], ids[(i + 1) % 4]],
            });
            self.constrain(if i % 2 == 0 {
                ConstraintKind::Horizontal { line: id }
            } else {
                ConstraintKind::Vertical { line: id }
            });
        }
        self.constrain(ConstraintKind::Fixed {
            point: ids[0],
            xy: a,
        });
        let n = self.parameters.len();
        let width = self.parameter(
            &format!(
                "width{}",
                if n == 0 { String::new() } else { n.to_string() }
            ),
            millimetres(b[0] - a[0]),
        );
        let height = self.parameter(
            &format!(
                "height{}",
                if n == 0 { String::new() } else { n.to_string() }
            ),
            millimetres(b[1] - a[1]),
        );
        self.constrain(ConstraintKind::DistanceX {
            points: [ids[0], ids[1]],
            parameter: width,
        });
        self.constrain(ConstraintKind::DistanceY {
            points: [ids[0], ids[3]],
            parameter: height,
        });
    }
    pub fn validate(&self) -> Result<(), String> {
        use std::collections::HashSet;
        if self.points.len() > 128
            || self.lines.len() + self.circles.len() + self.ellipses.len() + self.splines.len()
                > 128
            || self.constraints.len() > 256
            || self.parameters.len() > 128
        {
            return Err("Document exceeds current sketch limits".into());
        }
        let mut features = HashSet::new();
        let mut seen_sketches = HashSet::new();
        let solid = self.solid_features();
        for f in &self.construction {
            if !features.insert(f.id) || f.name.is_empty() || f.name.len() > 128 {
                return Err("Invalid construction feature".into());
            }
            match f.kind {
                ConstructionKind::Sketch => {
                    if Some(f.id) != self.current_sketch_id()
                        && !self.sketches.iter().any(|s| s.id == f.id)
                    {
                        return Err("Missing construction sketch".into());
                    }
                    seen_sketches.insert(f.id);
                }
                ConstructionKind::Extrude { sketch } => {
                    if !seen_sketches.contains(&sketch)
                        || !solid.iter().any(|e| e.id == f.id && e.sketch == sketch)
                    {
                        return Err("Invalid construction dependency".into());
                    }
                }
            }
        }
        let mut all = HashSet::new();
        for id in self
            .points
            .iter()
            .map(|p| p.id)
            .chain(self.lines.iter().map(|l| l.id))
            .chain(self.circles.iter().map(|p| p.id))
            .chain(self.ellipses.iter().map(|p| p.id))
            .chain(self.splines.iter().map(|p| p.id))
            .chain(self.driven_dimensions.iter().map(|p| p.id))
            .chain(self.parameters.iter().map(|p| p.id))
            .chain(self.constraints.iter().map(|c| c.id))
            .chain(self.extrusion.iter().map(|e| e.id))
        {
            if !all.insert(id) {
                return Err("Duplicate entity ID".into());
            }
        }
        let points: HashSet<_> = self.points.iter().map(|p| p.id).collect();
        let lines: HashSet<_> = self.lines.iter().map(|p| p.id).collect();
        let circles: HashSet<_> = self.circles.iter().map(|p| p.id).collect();
        let curves: HashSet<_> = lines
            .union(&circles)
            .copied()
            .chain(self.ellipses.iter().map(|c| c.id))
            .chain(self.splines.iter().map(|c| c.id))
            .collect();
        if self.ellipses.iter().any(|e| {
            [e.center, e.major, e.minor]
                .iter()
                .any(|id| !points.contains(id))
                || e.center == e.major
                || e.center == e.minor
        }) {
            return Err("Invalid ellipse references".into());
        }
        if self.splines.iter().any(|s| {
            s.points.len() < 2
                || s.points.len() > 32
                || s.points.iter().any(|id| !points.contains(id))
        }) {
            return Err("Invalid spline references".into());
        }
        if self.circles.iter().any(|c| {
            !points.contains(&c.center)
                || !points.contains(&c.rim)
                || c.center == c.rim
                || c.end.is_some_and(|e| !points.contains(&e) || e == c.center)
        }) {
            return Err("Invalid circle or arc references".into());
        }
        if self
            .construction_geometry
            .iter()
            .any(|id| !curves.contains(id))
        {
            return Err("Missing construction geometry".into());
        }
        if self
            .dimension_positions
            .values()
            .any(|p| p.iter().any(|v| !v.is_finite() || v.abs() > 1000.))
        {
            return Err("Invalid dimension position".into());
        }
        let params: HashSet<_> = self.parameters.iter().map(|p| p.id).collect();
        if self
            .points
            .iter()
            .any(|p| p.xy.iter().any(|x| !x.is_finite() || x.abs() > 1000.))
        {
            return Err("Invalid point coordinate".into());
        }
        if self
            .lines
            .iter()
            .any(|l| l.ends[0] == l.ends[1] || l.ends.iter().any(|id| !points.contains(id)))
        {
            return Err("Invalid line endpoints".into());
        }
        let mut names = HashSet::new();
        for p in &self.parameters {
            if p.name.is_empty()
                || !p
                    .name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
                || p.name.chars().next().unwrap().is_ascii_digit()
                || !names.insert(&p.name)
                || p.expression.len() > 512
            {
                return Err("Invalid or duplicate parameter name".into());
            }
        }
        for c in self.constraints.iter().chain(&self.driven_dimensions) {
            let valid = match &c.kind {
                ConstraintKind::HorizontalPoints { points: ps }
                | ConstraintKind::VerticalPoints { points: ps } => {
                    ps.iter().all(|p| points.contains(p))
                }
                ConstraintKind::LineOffset {
                    lines: ls, side, ..
                } => ls.iter().all(|l| lines.contains(l)) && side.abs() == 1.,
                ConstraintKind::OffsetRadius { circles: cs, .. } => {
                    cs.iter().all(|c| circles.contains(c))
                }
                ConstraintKind::Direction { line, angle } => {
                    lines.contains(line) && angle.is_finite()
                }
                ConstraintKind::ProjectedAlignment {
                    points: ps,
                    direction,
                }
                | ConstraintKind::ProjectedDistance {
                    points: ps,
                    direction,
                    ..
                } => {
                    ps.iter().all(|p| points.contains(p))
                        && direction.iter().all(|v| v.is_finite())
                        && (direction[0].hypot(direction[1]) - 1.).abs() < 1e-8
                }
                ConstraintKind::Coincident { points: ps }
                | ConstraintKind::Distance { points: ps, .. } => {
                    ps.iter().all(|p| points.contains(p))
                }
                ConstraintKind::Parallel { lines: ls }
                | ConstraintKind::Perpendicular { lines: ls }
                | ConstraintKind::Collinear { lines: ls }
                | ConstraintKind::Angle { lines: ls, .. }
                | ConstraintKind::LineDistance { lines: ls, .. } => {
                    ls[0] != ls[1] && ls.iter().all(|l| lines.contains(l))
                }
                ConstraintKind::Equal { curves: cs } | ConstraintKind::Tangent { curves: cs } => {
                    cs[0] != cs[1]
                        && cs.iter().all(|l| lines.contains(l) || circles.contains(l))
                        && (!matches!(c.kind, ConstraintKind::Equal { .. })
                            || cs.iter().all(|id| lines.contains(id))
                            || cs.iter().all(|id| circles.contains(id)))
                        && (!matches!(c.kind, ConstraintKind::Tangent { .. })
                            || cs.iter().any(|id| circles.contains(id)))
                }
                ConstraintKind::Midpoint { point, line }
                | ConstraintKind::PointOnLine { point, line } => {
                    points.contains(point) && lines.contains(line)
                }
                ConstraintKind::PointOnCircle { point, circle } => {
                    points.contains(point) && circles.contains(circle)
                }
                ConstraintKind::Concentric { circles: cs } => {
                    cs[0] != cs[1] && cs.iter().all(|id| circles.contains(id))
                }
                ConstraintKind::Symmetry { points: ps, axis } => {
                    ps.iter().all(|p| points.contains(p)) && lines.contains(axis)
                }
                ConstraintKind::Diameter { circle, .. } | ConstraintKind::Radius { circle, .. } => {
                    circles.contains(circle)
                }
                ConstraintKind::Horizontal { line }
                | ConstraintKind::Vertical { line }
                | ConstraintKind::Length { line, .. } => lines.contains(line),
                ConstraintKind::Fixed { point, xy } => {
                    points.contains(point) && xy.iter().all(|x| x.is_finite() && x.abs() <= 1000.)
                }
                ConstraintKind::DistanceX { points: ps, .. }
                | ConstraintKind::DistanceY { points: ps, .. } => {
                    ps.iter().all(|p| points.contains(p))
                }
            };
            if !valid {
                return Err("Constraint references a missing entity".into());
            }
            if let ConstraintKind::LineOffset { parameter, .. }
            | ConstraintKind::OffsetRadius { parameter, .. }
            | ConstraintKind::ProjectedDistance { parameter, .. }
            | ConstraintKind::LineDistance { parameter, .. }
            | ConstraintKind::Distance { parameter, .. }
            | ConstraintKind::Angle { parameter, .. }
            | ConstraintKind::Diameter { parameter, .. }
            | ConstraintKind::Radius { parameter, .. }
            | ConstraintKind::DistanceX { parameter, .. }
            | ConstraintKind::DistanceY { parameter, .. }
            | ConstraintKind::Length { parameter, .. } = &c.kind
                && !self
                    .driven_dimensions
                    .iter()
                    .any(|driven| driven.id == c.id)
                && !params.contains(parameter)
            {
                return Err("Missing dimension parameter".into());
            }
        }
        if self
            .extrusion
            .as_ref()
            .is_some_and(|e| !params.contains(&e.depth))
        {
            return Err("Missing extrusion depth parameter".into());
        }
        for c in &self.constraints {
            if let Some(id) = c.kind.parameter() {
                let p = self.parameters.iter().find(|p| p.id == id).unwrap();
                if p.angular != matches!(c.kind, ConstraintKind::Angle { .. }) {
                    return Err("Dimension parameter has incompatible units".into());
                }
            }
        }
        self.validate_model()?;
        Ok(())
    }
}

fn millimetres(value: f64) -> String {
    format!(
        "{} mm",
        format!("{:.9}", value * 1000.)
            .trim_end_matches('0')
            .trim_end_matches('.')
    )
}

impl ConstraintKind {
    pub fn parameter(&self) -> Option<Uuid> {
        match self {
            Self::LineOffset { parameter, .. }
            | Self::OffsetRadius { parameter, .. }
            | Self::ProjectedDistance { parameter, .. }
            | Self::LineDistance { parameter, .. }
            | Self::DistanceX { parameter, .. }
            | Self::DistanceY { parameter, .. }
            | Self::Length { parameter, .. }
            | Self::Distance { parameter, .. }
            | Self::Angle { parameter, .. }
            | Self::Diameter { parameter, .. }
            | Self::Radius { parameter, .. } => Some(*parameter),
            _ => None,
        }
    }
    pub fn references(&self) -> Vec<Uuid> {
        match self {
            Self::Direction { line, .. }
            | Self::Horizontal { line }
            | Self::Vertical { line }
            | Self::Length { line, .. } => {
                vec![*line]
            }
            Self::Fixed { point, .. } => vec![*point],
            Self::ProjectedAlignment { points, .. }
            | Self::HorizontalPoints { points }
            | Self::VerticalPoints { points }
            | Self::ProjectedDistance { points, .. }
            | Self::DistanceX { points, .. }
            | Self::DistanceY { points, .. }
            | Self::Distance { points, .. }
            | Self::Coincident { points } => points.to_vec(),
            Self::Parallel { lines }
            | Self::Perpendicular { lines }
            | Self::Collinear { lines }
            | Self::Angle { lines, .. }
            | Self::LineOffset { lines, .. }
            | Self::LineDistance { lines, .. } => lines.to_vec(),
            Self::Equal { curves } | Self::Tangent { curves } => curves.to_vec(),
            Self::Midpoint { point, line } | Self::PointOnLine { point, line } => {
                vec![*point, *line]
            }
            Self::PointOnCircle { point, circle } => vec![*point, *circle],
            Self::Concentric { circles } | Self::OffsetRadius { circles, .. } => circles.to_vec(),
            Self::Symmetry { points, axis } => vec![points[0], points[1], *axis],
            Self::Diameter { circle, .. } | Self::Radius { circle, .. } => vec![*circle],
        }
    }
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Horizontal { .. } => "line_horizontal",
            Self::Vertical { .. } => "line_vertical",
            Self::Fixed { .. } => "locked",
            Self::Parallel { .. } => "line_parallel",
            Self::Perpendicular { .. } => "line_perpendicular",
            Self::Equal { .. } => "restr_ortho",
            Self::Concentric { .. } => "circle_concentric",
            Self::Tangent { .. } => "line_tangent_pc",
            Self::Symmetry { .. } => "mirror",
            Self::Midpoint { .. } => "snap_middle",
            Self::Collinear { .. } => "line",
            Self::Direction { .. } | Self::Angle { .. } => "dim_angular",
            _ => "snap_endpoints",
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Direction { .. } => "Angle",
            Self::Horizontal { .. } | Self::HorizontalPoints { .. } => "Horizontal",
            Self::Vertical { .. } | Self::VerticalPoints { .. } => "Vertical",
            Self::Fixed { .. } => "Fixed",
            Self::Coincident { .. } => "Coincident",
            Self::Parallel { .. } => "Parallel",
            Self::Perpendicular { .. } => "Perpendicular",
            Self::Equal { .. } => "Equal",
            Self::Collinear { .. } => "Collinear",
            Self::Midpoint { .. } => "Midpoint",
            Self::PointOnLine { .. } | Self::PointOnCircle { .. } => "Coincident",
            Self::Concentric { .. } => "Concentric",
            Self::Tangent { .. } => "Tangent",
            Self::Symmetry { .. } => "Symmetry",
            Self::Angle { .. } => "Angle",
            Self::Diameter { .. } => "Diameter",
            Self::Radius { .. } => "Radius",
            _ => "Distance",
        }
    }
}
