//! Current planar design intent, in metres, with stable IDs and parameter-driven constraints.
//! Exports Design, Point, Line, Constraint and Parameter; solver, persistence and UI share
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
    pub id: Uuid,
    pub name: String,
    pub expression: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConstraintKind {
    Horizontal { line: Uuid },
    Vertical { line: Uuid },
    Fixed { point: Uuid, xy: [f64; 2] },
    DistanceX { points: [Uuid; 2], parameter: Uuid },
    DistanceY { points: [Uuid; 2], parameter: Uuid },
    Length { line: Uuid, parameter: Uuid },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Constraint {
    pub id: Uuid,
    pub kind: ConstraintKind,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extrusion {
    pub id: Uuid,
    pub depth: Uuid,
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
    pub points: Vec<Point>,
    pub lines: Vec<Line>,
    pub constraints: Vec<Constraint>,
    pub parameters: Vec<Parameter>,
    pub extrusion: Option<Extrusion>,
    #[serde(default)]
    pub construction: Vec<ConstructionFeature>,
}
impl Design {
    pub fn ensure_sketch(&mut self) -> Uuid {
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
    /// Normalize supported current feature definitions; changing parameters adds no feature.
    pub fn sync_construction(&mut self) {
        if !self.lines.is_empty() || self.extrusion.is_some() {
            self.ensure_sketch();
        }
        if let Some(extrusion) = &self.extrusion {
            let id = extrusion.id;
            let sketch = self
                .construction
                .iter()
                .find(|f| matches!(f.kind, ConstructionKind::Sketch))
                .unwrap()
                .id;
            self.construction
                .retain(|f| matches!(f.kind, ConstructionKind::Sketch) || f.id == id);
            if !self.construction.iter().any(|f| f.id == id) {
                self.construction.push(ConstructionFeature {
                    id,
                    name: "Extrude 1".into(),
                    kind: ConstructionKind::Extrude { sketch },
                });
            }
        } else {
            self.construction
                .retain(|f| matches!(f.kind, ConstructionKind::Sketch));
        }
    }
    /// Derived evaluation at a construction marker; canonical downstream intent is retained.
    pub fn through_feature(&self, id: Uuid) -> Result<Self, String> {
        let feature = self
            .construction
            .iter()
            .find(|f| f.id == id)
            .ok_or("Missing construction feature")?;
        let mut result = self.clone();
        if matches!(feature.kind, ConstructionKind::Sketch) {
            result.extrusion = None;
            result.construction.retain(|f| f.id == id);
        }
        Ok(result)
    }

    pub fn parameter(&mut self, name: &str, expression: String) -> Uuid {
        if let Some(p) = self.parameters.iter_mut().find(|p| p.name == name) {
            p.expression = expression;
            return p.id;
        }
        let id = Uuid::new_v4();
        self.parameters.push(Parameter {
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
            .find(|p| (p.xy[0] - xy[0]).hypot(p.xy[1] - xy[1]) < 0.0005)
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
            format!("{} mm", (b[0] - a[0]) * 1000.),
        );
        let height = self.parameter(
            &format!(
                "height{}",
                if n == 0 { String::new() } else { n.to_string() }
            ),
            format!("{} mm", (b[1] - a[1]) * 1000.),
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
            || self.lines.len() > 128
            || self.constraints.len() > 256
            || self.parameters.len() > 128
        {
            return Err("Document exceeds current sketch limits".into());
        }
        let mut features = HashSet::new();
        let mut seen_sketch = None;
        let mut seen_extrude = false;
        for f in &self.construction {
            if !features.insert(f.id) || f.name.is_empty() || f.name.len() > 128 {
                return Err("Invalid construction feature".into());
            }
            match f.kind {
                ConstructionKind::Sketch => {
                    if seen_sketch.replace(f.id).is_some() || seen_extrude {
                        return Err("Only one planar sketch is currently supported".into());
                    }
                }
                ConstructionKind::Extrude { sketch } => {
                    if seen_sketch != Some(sketch)
                        || seen_extrude
                        || self.extrusion.as_ref().is_none_or(|e| e.id != f.id)
                    {
                        return Err("Invalid construction dependency".into());
                    }
                    seen_extrude = true;
                }
            }
        }
        let mut all = HashSet::new();
        for id in self
            .points
            .iter()
            .map(|p| p.id)
            .chain(self.lines.iter().map(|l| l.id))
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
        for c in &self.constraints {
            let valid = match &c.kind {
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
            if let ConstraintKind::DistanceX { parameter, .. }
            | ConstraintKind::DistanceY { parameter, .. }
            | ConstraintKind::Length { parameter, .. } = &c.kind
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
        Ok(())
    }
}
