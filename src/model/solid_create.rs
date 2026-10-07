//! Persistent Solid/Create intent and the compact, unit-aware parameter catalog.
use crate::document::schema::Design;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u32)]
pub enum CreateKind {
    Revolve,
    Sweep,
    Loft,
    Rib,
    Web,
    Emboss,
    Hole,
    Thread,
    Box,
    Cylinder,
    Sphere,
    Torus,
    Coil,
    Pipe,
    RectangularPattern,
    CircularPattern,
    PatternOnPath,
    Mirror,
    Thicken,
    BoundaryFill,
    Gear,
}
#[derive(Clone, Copy)]
pub struct Field {
    pub label: &'static str,
    pub default: &'static str,
    pub unit: Unit,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Unit {
    Length,
    Angle,
    Number,
}
impl CreateKind {
    pub const ALL: [Self; 21] = [
        Self::Revolve,
        Self::Sweep,
        Self::Loft,
        Self::Rib,
        Self::Web,
        Self::Emboss,
        Self::Hole,
        Self::Thread,
        Self::Box,
        Self::Cylinder,
        Self::Sphere,
        Self::Torus,
        Self::Coil,
        Self::Pipe,
        Self::RectangularPattern,
        Self::CircularPattern,
        Self::PatternOnPath,
        Self::Mirror,
        Self::Thicken,
        Self::BoundaryFill,
        Self::Gear,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Revolve => "Revolve",
            Self::Sweep => "Sweep",
            Self::Loft => "Loft",
            Self::Rib => "Rib",
            Self::Web => "Web",
            Self::Emboss => "Emboss",
            Self::Hole => "Hole",
            Self::Thread => "Thread",
            Self::Box => "Box",
            Self::Cylinder => "Cylinder",
            Self::Sphere => "Sphere",
            Self::Torus => "Torus",
            Self::Coil => "Coil",
            Self::Pipe => "Pipe",
            Self::RectangularPattern => "Rectangular pattern",
            Self::CircularPattern => "Circular pattern",
            Self::PatternOnPath => "Pattern on path",
            Self::Mirror => "Mirror",
            Self::Thicken => "Thicken",
            Self::BoundaryFill => "Boundary fill",
            Self::Gear => "Gear",
        }
    }
    pub fn uses_profile(self) -> bool {
        matches!(
            self,
            Self::Revolve | Self::Sweep | Self::Loft | Self::Emboss
        )
    }
    pub fn uses_body(self) -> bool {
        matches!(
            self,
            Self::Rib
                | Self::Web
                | Self::Emboss
                | Self::Hole
                | Self::Thread
                | Self::RectangularPattern
                | Self::CircularPattern
                | Self::PatternOnPath
                | Self::Mirror
                | Self::Thicken
                | Self::BoundaryFill
        )
    }
    pub fn consumes_body(self) -> bool {
        matches!(
            self,
            Self::Rib
                | Self::Web
                | Self::Emboss
                | Self::Hole
                | Self::Thread
                | Self::Thicken
                | Self::BoundaryFill
        )
    }
    pub fn help(self) -> &'static str {
        match self {
            Self::Revolve => "Revolve a closed XY sketch about a Y axis at the chosen X position.",
            Self::Sweep => "Sweep a closed sketch along a straight 3D vector (X, Y, Z).",
            Self::Loft => {
                "Loft the active closed sketch to a second sketch, translated along Z. One outer wire per section."
            }
            Self::Rib => {
                "Join a vertical rectangular rib to the chosen body. Set its size and XY position."
            }
            Self::Web => {
                "Join a horizontal rectangular web to the chosen body. Set its size and XYZ position."
            }
            Self::Emboss => {
                "Raise a closed XY sketch from the chosen Z height and join it to the body. Planar support only."
            }
            Self::Hole => {
                "Cut a cylindrical hole along Z through the chosen depth, starting at the chosen XYZ position."
            }
            Self::Thread => {
                "Cut a rounded helical groove in a cylindrical body about the origin Z axis. Radius is the groove centre radius."
            }
            Self::Coil => "Create a helical coil about the origin Z axis with a circular section.",
            Self::Pipe => "Create a straight hollow pipe along Z.",
            Self::RectangularPattern => "Copy a body in an XY grid. Counts include the original.",
            Self::CircularPattern => {
                "Copy a body about the origin Z axis. Count includes the original; angle is the total span."
            }
            Self::PatternOnPath => {
                "Copy a body along a straight 3D path. Count includes the original; vector is the total displacement."
            }
            Self::Mirror => {
                "Copy a body across a principal plane: 0 = YZ, 1 = XZ, 2 = XY. Offset locates the plane."
            }
            Self::Thicken => {
                "Create a closed wall around a solid using an outward offset and subtracting its original volume."
            }
            Self::BoundaryFill => {
                "Create the common enclosed cell of two overlapping solid bodies; both inputs are consumed."
            }
            Self::Gear => {
                "Create a spur gear with sampled involute teeth (20° pressure angle), optional centre bore, and Z thickness."
            }
            _ => "Create a new solid at the origin. Lengths accept units and named parameters.",
        }
    }
    pub fn fields(self) -> Vec<Field> {
        use Unit::*;
        let l = |label, default| Field {
            label,
            default,
            unit: Length,
        };
        let n = |label, default| Field {
            label,
            default,
            unit: Number,
        };
        let a = |label, default| Field {
            label,
            default,
            unit: Angle,
        };
        match self {
            Self::Box => vec![
                l("Width", "20 mm"),
                l("Depth", "20 mm"),
                l("Height", "20 mm"),
            ],
            Self::Cylinder => vec![l("Radius", "10 mm"), l("Height", "20 mm")],
            Self::Sphere => vec![l("Radius", "10 mm")],
            Self::Torus => vec![l("Major radius", "20 mm"), l("Tube radius", "5 mm")],
            Self::Revolve => vec![a("Angle", "360 deg"), l("Axis X", "0 mm")],
            Self::Sweep => vec![
                l("Path X", "0 mm"),
                l("Path Y", "0 mm"),
                l("Path Z", "20 mm"),
            ],
            Self::Loft => vec![l("Section spacing", "20 mm")],
            Self::Rib => vec![
                l("Length", "20 mm"),
                l("Thickness", "2 mm"),
                l("Height", "10 mm"),
                l("X", "0 mm"),
                l("Y", "0 mm"),
                l("Z", "0 mm"),
            ],
            Self::Web => vec![
                l("Length", "20 mm"),
                l("Width", "10 mm"),
                l("Thickness", "2 mm"),
                l("X", "0 mm"),
                l("Y", "0 mm"),
                l("Z", "0 mm"),
            ],
            Self::Emboss => vec![l("Height", "2 mm"), l("Start Z", "0 mm")],
            Self::Hole => vec![
                l("Radius", "3 mm"),
                l("Depth", "20 mm"),
                l("X", "0 mm"),
                l("Y", "0 mm"),
                l("Start Z", "0 mm"),
            ],
            Self::Thread | Self::Coil => vec![
                l("Helix radius", "10 mm"),
                l("Pitch", "5 mm"),
                n("Turns", "3"),
                l("Section radius", "1 mm"),
            ],
            Self::Pipe => vec![
                l("Outer radius", "10 mm"),
                l("Wall thickness", "2 mm"),
                l("Length", "20 mm"),
            ],
            Self::RectangularPattern => vec![
                n("Columns", "3"),
                n("Rows", "2"),
                l("X spacing", "30 mm"),
                l("Y spacing", "30 mm"),
            ],
            Self::CircularPattern => vec![n("Count", "4"), a("Span", "360 deg")],
            Self::PatternOnPath => vec![
                n("Count", "3"),
                l("Path X", "60 mm"),
                l("Path Y", "0 mm"),
                l("Path Z", "0 mm"),
            ],
            Self::Mirror => vec![n("Plane (0, 1, 2)", "0"), l("Plane offset", "0 mm")],
            Self::Thicken => vec![l("Wall thickness", "2 mm")],
            Self::BoundaryFill => vec![],
            Self::Gear => vec![
                l("Module", "2 mm"),
                n("Teeth", "20"),
                l("Thickness", "5 mm"),
                l("Bore radius", "3 mm"),
            ],
        }
    }
    pub fn validate_values(self, v: &[f64]) -> Result<(), String> {
        if v.len() != self.fields().len() || v.iter().any(|v| !v.is_finite() || v.abs() > 1000.) {
            return Err("Invalid feature dimensions".into());
        }
        let positive = |i: usize| {
            if v[i] > 1e-7 {
                Ok(())
            } else {
                Err(format!("{} must be positive", self.fields()[i].label))
            }
        };
        let count = |i: usize, min, max| {
            if v[i].fract() == 0. && v[i] >= min && v[i] <= max {
                Ok(())
            } else {
                Err(format!(
                    "{} must be an integer between {min} and {max}",
                    self.fields()[i].label
                ))
            }
        };
        match self {
            Self::Box | Self::Rib | Self::Web => {
                positive(0)?;
                positive(1)?;
                positive(2)?;
            }
            Self::Cylinder | Self::Torus | Self::Pipe => {
                positive(0)?;
                positive(1)?;
                if matches!(self, Self::Torus | Self::Pipe) && v[1] >= v[0] {
                    return Err(
                        "Secondary radius/thickness must be smaller than the outer radius".into(),
                    );
                }
                if self == Self::Pipe {
                    positive(2)?;
                }
            }
            Self::Sphere | Self::Loft | Self::Emboss | Self::Thicken => positive(0)?,
            Self::Hole => {
                positive(0)?;
                positive(1)?;
            }
            Self::Revolve => {
                positive(0)?;
                if v[0] > std::f64::consts::TAU + 1e-8 {
                    return Err("Angle must be at most 360 degrees".into());
                }
            }
            Self::Sweep => {
                if v.iter().map(|x| x * x).sum::<f64>() < 1e-14 {
                    return Err("Path must have nonzero length".into());
                }
            }
            Self::Thread | Self::Coil => {
                positive(0)?;
                positive(1)?;
                count(2, 1., 32.)?;
                positive(3)?;
                if v[3] * 2. >= v[1] || v[3] >= v[0] {
                    return Err("Section diameter must be smaller than pitch and section radius smaller than helix radius".into());
                }
            }
            Self::RectangularPattern => {
                count(0, 1., 32.)?;
                count(1, 1., 32.)?;
                if v[0] * v[1] > 128. || v[0] * v[1] < 2. {
                    return Err("Pattern requires 2–128 instances".into());
                }
            }
            Self::CircularPattern => {
                count(0, 2., 128.)?;
                positive(1)?;
                if v[1] > std::f64::consts::TAU + 1e-8 {
                    return Err("Span must be at most 360 degrees".into());
                }
            }
            Self::PatternOnPath => {
                count(0, 2., 128.)?;
                if v[1..].iter().map(|x| x * x).sum::<f64>() < 1e-14 {
                    return Err("Path must have nonzero length".into());
                }
            }
            Self::Mirror => count(0, 0., 2.)?,
            Self::Gear => {
                positive(0)?;
                count(1, 6., 128.)?;
                positive(2)?;
                if v[3] < 0. || v[3] >= v[0] * (v[1] / 2. - 1.25) {
                    return Err("Bore must fit inside the gear root circle".into());
                }
            }
            Self::BoundaryFill => {}
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateFeature {
    pub id: Uuid,
    pub name: String,
    pub kind: CreateKind,
    pub parameters: Vec<Uuid>,
    pub sketch: Option<Uuid>,
    pub second_sketch: Option<Uuid>,
    pub boundary: Vec<Uuid>,
    pub target: Option<Uuid>,
    pub second_target: Option<Uuid>,
}
impl Design {
    pub fn validate_create_features(&self) -> Result<(), String> {
        use std::collections::HashSet;
        if self.create_features.len() > 64 {
            return Err("Too many Create features".into());
        }
        let mut available: HashSet<_> = self.solid_features().iter().map(|f| f.id).collect();
        let mut consumed: HashSet<_> = self
            .solid_features()
            .iter()
            .filter_map(|f| f.target)
            .collect();
        let mut identities = HashSet::new();
        for sketch in self
            .construction
            .iter()
            .filter(|c| matches!(c.kind, crate::document::schema::ConstructionKind::Sketch))
        {
            identities.insert(sketch.id);
            let input = self.sketch_input(sketch.id)?;
            identities.extend(input.points.iter().map(|p| p.id));
            identities.extend(crate::sketch::entities::curve_ids(&input));
            identities.extend(
                input
                    .constraints
                    .iter()
                    .chain(&input.driven_dimensions)
                    .map(|c| c.id),
            );
        }
        identities.extend(self.parameters.iter().map(|p| p.id));
        for f in &self.create_features {
            if !identities.insert(f.id)
                || available.contains(&f.id)
                || self.parameters.iter().any(|p| p.id == f.id)
                || f.name.is_empty()
                || f.name.len() > 128
                || f.parameters.len() != f.kind.fields().len()
            {
                return Err("Invalid Create feature identity or parameters".into());
            }
            for (id, field) in f.parameters.iter().zip(f.kind.fields()) {
                let p = self
                    .parameters
                    .iter()
                    .find(|p| p.id == *id)
                    .ok_or("Missing Create parameter")?;
                if p.angular != (field.unit == Unit::Angle)
                    || p.scalar != (field.unit == Unit::Number)
                {
                    return Err("Incorrect Create parameter units".into());
                }
            }
            if f.kind.uses_profile() != f.sketch.is_some() {
                return Err("Choose a profile sketch".into());
            }
            if let Some(id) = f.sketch {
                let input = self.sketch_input(id)?;
                let curves = crate::sketch::entities::curve_ids(&input);
                let unique: HashSet<_> = f.boundary.iter().collect();
                if unique.len() != f.boundary.len()
                    || f.boundary.iter().any(|id| {
                        (!curves.contains(id) && !crate::sketch::regions::is_boundary_token(id))
                            || input.construction_geometry.contains(id)
                    })
                {
                    return Err("Invalid Create profile boundary".into());
                }
            } else if !f.boundary.is_empty() {
                return Err("This Create feature does not use a profile".into());
            }
            if f.kind != CreateKind::Loft && f.second_sketch.is_some() {
                return Err("Unexpected second profile".into());
            }
            if f.kind != CreateKind::BoundaryFill && f.second_target.is_some() {
                return Err("Unexpected second body".into());
            }
            if f.kind == CreateKind::Loft {
                let id = f.second_sketch.ok_or("Choose a second loft sketch")?;
                if Some(id) == f.sketch {
                    return Err("Loft requires two different sketches".into());
                }
                self.sketch_input(id)?;
            }
            if f.kind.uses_body() != f.target.is_some() {
                return Err("Choose a body".into());
            }
            if let Some(id) = f.target {
                if !available.contains(&id) || consumed.contains(&id) {
                    return Err("Choose a current upstream body".into());
                }
                if f.kind.consumes_body() {
                    consumed.insert(id);
                }
            }
            if f.kind == CreateKind::BoundaryFill {
                let id = f.second_target.ok_or("Choose a second body")?;
                if Some(id) == f.target || !available.contains(&id) || consumed.contains(&id) {
                    return Err("Choose a different current upstream body".into());
                }
                consumed.insert(id);
            }
            available.insert(f.id);
        }
        Ok(())
    }
    pub fn current_create_bodies(&self) -> Vec<(Uuid, String)> {
        let consumed: std::collections::HashSet<_> = self
            .solid_features()
            .iter()
            .filter_map(|f| f.target)
            .chain(
                self.create_features
                    .iter()
                    .filter(|f| f.kind.consumes_body())
                    .flat_map(|f| [f.target, f.second_target].into_iter().flatten()),
            )
            .collect();
        self.solid_features()
            .iter()
            .map(|f| (f.id, f.name.clone()))
            .chain(self.create_features.iter().map(|f| (f.id, f.name.clone())))
            .filter(|(id, _)| !consumed.contains(id))
            .collect()
    }
}
