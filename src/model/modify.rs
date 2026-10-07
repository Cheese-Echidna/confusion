//! Persistent direct solid edits; native evaluation rebuilds these after extrusion evaluation.
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u32)]
pub enum ModifyKind {
    PressPull,
    Fillet,
    Chamfer,
    Shell,
    Draft,
    Scale,
    Combine,
    OffsetFace,
    ReplaceFace,
    SplitBody,
    SplitFace,
    SilhouetteSplit,
    MoveCopy,
    Align,
    Delete,
    Remove,
    Simplify,
}
impl ModifyKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::PressPull => "Press pull",
            Self::Fillet => "Fillet",
            Self::Chamfer => "Chamfer",
            Self::Shell => "Shell",
            Self::Draft => "Draft",
            Self::Scale => "Scale",
            Self::Combine => "Combine",
            Self::OffsetFace => "Offset face",
            Self::ReplaceFace => "Replace face",
            Self::SplitBody => "Split body",
            Self::SplitFace => "Split face",
            Self::SilhouetteSplit => "Silhouette split",
            Self::MoveCopy => "Move / copy",
            Self::Align => "Align",
            Self::Delete => "Delete",
            Self::Remove => "Remove",
            Self::Simplify => "Simplify",
        }
    }
    pub fn uses_face(self) -> bool {
        !matches!(
            self,
            Self::Scale | Self::Combine | Self::SplitBody | Self::MoveCopy | Self::Align
        )
    }
    pub fn code(self) -> u32 {
        self as u32
    }
    /// 0 scalar, 1 length, 2 angle; absent slots cannot bind parameters.
    pub fn parameter_unit(self, slot: usize) -> Option<u8> {
        match (self, slot) {
            (Self::Scale, 0)
            | (Self::ReplaceFace, 1)
            | (Self::SplitBody | Self::SplitFace, 1..=3) => Some(0),
            (Self::Draft, 0) | (Self::MoveCopy, 3) => Some(2),
            (
                Self::PressPull
                | Self::Fillet
                | Self::Chamfer
                | Self::Shell
                | Self::OffsetFace
                | Self::SplitBody
                | Self::SplitFace,
                0,
            )
            | (Self::Draft, 1)
            | (Self::Scale, 1..=3)
            | (Self::MoveCopy, 0..=2) => Some(1),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SolidEdit {
    pub id: Uuid,
    pub kind: ModifyKind,
    pub target: Uuid,
    pub tool: Option<Uuid>,
    /// Face ordinal in the deterministic output before this edit; zero means whole body.
    pub face: u32,
    /// Semantic kernel provenance; ordinals remain only for legacy files and display.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub face_reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_reference: Option<String>,
    pub values: [f64; 4],
    #[serde(default)]
    pub parameters: [Option<Uuid>; 4],
    pub copy: bool,
    /// Combine: 0 join, 1 cut, 2 intersect.
    pub mode: u32,
}
impl SolidEdit {
    pub fn validate(&self) -> Result<(), String> {
        if [&self.face_reference, &self.tool_reference]
            .into_iter()
            .flatten()
            .any(|key| key.is_empty() || key.len() > 4096 || key.chars().any(char::is_control))
        {
            return Err("Invalid solid face reference".into());
        }
        if self
            .values
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 1000.)
            || self.mode > 2
            || self.face > 100000
        {
            return Err("Invalid solid edit parameters".into());
        }
        if matches!(
            self.kind,
            ModifyKind::Fillet | ModifyKind::Chamfer | ModifyKind::Shell | ModifyKind::Scale
        ) && self.values[0] <= 1e-7
        {
            return Err("Enter a positive radius, thickness or scale factor".into());
        }
        if matches!(
            self.kind,
            ModifyKind::Combine
                | ModifyKind::Align
                | ModifyKind::ReplaceFace
                | ModifyKind::SilhouetteSplit
        ) && (self.tool.is_none() || self.tool == Some(self.target))
        {
            return Err("Choose a different tool body".into());
        }
        if matches!(
            self.kind,
            ModifyKind::PressPull
                | ModifyKind::OffsetFace
                | ModifyKind::Shell
                | ModifyKind::Draft
                | ModifyKind::ReplaceFace
                | ModifyKind::SplitFace
                | ModifyKind::SilhouetteSplit
        ) && self.face == 0
        {
            return Err("Select a face in the viewport".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyStyle {
    pub body: Uuid,
    pub material: String,
    pub density: f64,
    pub color: [f32; 3],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Material {
    pub name: String,
    pub density: f64,
    pub color: [f32; 3],
}
impl Material {
    pub fn presets() -> Vec<Self> {
        vec![
            Self {
                name: "Aluminium".into(),
                density: 2700.,
                color: [0.65, 0.68, 0.72],
            },
            Self {
                name: "Steel".into(),
                density: 7850.,
                color: [0.45, 0.47, 0.5],
            },
            Self {
                name: "ABS plastic".into(),
                density: 1040.,
                color: [0.2, 0.3, 0.45],
            },
            Self {
                name: "Copper".into(),
                density: 8960.,
                color: [0.72, 0.34, 0.18],
            },
        ]
    }
}
impl crate::document::schema::Design {
    pub fn current_modify_bodies(&self) -> Vec<(Uuid, String)> {
        let removed: std::collections::HashSet<_> = self
            .solid_edits
            .iter()
            .filter_map(|edit| {
                if matches!(edit.kind, ModifyKind::Delete | ModifyKind::Remove) && edit.face == 0 {
                    Some(edit.target)
                } else if edit.kind == ModifyKind::Combine && !edit.copy {
                    edit.tool
                } else {
                    None
                }
            })
            .collect();
        self.current_create_bodies()
            .into_iter()
            .filter(|(id, _)| !removed.contains(id))
            .collect()
    }
    pub fn validate_modifications(&self) -> Result<(), String> {
        let ids: std::collections::HashSet<_> = self
            .solid_features()
            .iter()
            .map(|f| f.id)
            .chain(self.create_features.iter().map(|f| f.id))
            .collect();
        let mut available: std::collections::HashSet<_> = self
            .current_create_bodies()
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        let mut edits = ids.clone();
        if self.solid_edits.len() > 128 || self.materials.len() > 128 {
            return Err("Too many solid edits or materials".into());
        }
        for edit in &self.solid_edits {
            edit.validate()?;
            for (slot, id) in edit.parameters.iter().enumerate() {
                if let Some(id) = id {
                    let parameter = self
                        .parameters
                        .iter()
                        .find(|p| p.id == *id)
                        .ok_or("Missing solid edit parameter")?;
                    let unit = edit
                        .kind
                        .parameter_unit(slot)
                        .ok_or("Unexpected solid edit parameter")?;
                    if parameter.scalar != (unit == 0) || parameter.angular != (unit == 2) {
                        return Err("Solid edit parameter has incompatible units".into());
                    }
                }
            }
            if !available.contains(&edit.target)
                || edit.tool.is_some_and(|id| !available.contains(&id))
            {
                return Err("Solid edit references a consumed or removed body".into());
            }
            if matches!(edit.kind, ModifyKind::Delete | ModifyKind::Remove) && edit.face == 0 {
                available.remove(&edit.target);
            }
            if edit.kind == ModifyKind::Combine
                && !edit.copy
                && let Some(tool) = edit.tool
            {
                available.remove(&tool);
            }
            if self.parameters.iter().any(|p| p.id == edit.id)
                || self.construction.iter().any(|f| {
                    f.id == edit.id
                        && !matches!(f.kind, crate::document::schema::ConstructionKind::Modify)
                })
            {
                return Err("Duplicate solid edit identity".into());
            }
            if !edits.insert(edit.id)
                || !ids.contains(&edit.target)
                || edit.tool.is_some_and(|id| !ids.contains(&id))
            {
                return Err("Missing solid edit body or duplicate edit identity".into());
            }
        }
        let mut styles = std::collections::HashSet::new();
        for style in &self.body_styles {
            if !styles.insert(style.body) {
                return Err("Duplicate body material assignment".into());
            }
            if !ids.contains(&style.body) {
                return Err("Missing material body".into());
            }
            validate_material(&style.material, style.density, style.color)?;
        }
        let mut names = std::collections::HashSet::new();
        for material in &self.materials {
            if !names.insert(&material.name) {
                return Err("Duplicate material name".into());
            }
            validate_material(&material.name, material.density, material.color)?;
        }
        Ok(())
    }
}
fn validate_material(name: &str, density: f64, color: [f32; 3]) -> Result<(), String> {
    if name.trim().is_empty()
        || name.len() > 128
        || !density.is_finite()
        || density <= 0.
        || density > 1e6
        || color
            .iter()
            .any(|c| !c.is_finite() || !(0. ..=1.).contains(c))
    {
        return Err(
            "Enter a material name, positive density (kg/m³), and RGB values between 0 and 1"
                .into(),
        );
    }
    Ok(())
}
