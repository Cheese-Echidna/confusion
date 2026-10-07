// Included inside the desktop workspace implementation; uses existing inspector controls.
use crate::model::modify::{BodyStyle, Material, ModifyKind, SolidEdit};
struct SolidEditor {
    kind: ModifyKind,
    editing: Option<Uuid>,
    target: Option<Uuid>,
    tool: Option<Uuid>,
    fields: Vec<Entity<TextInput>>,
    labels: Vec<&'static str>,
    copy: bool,
    mode: u32,
    material_action: Action,
}
impl SolidEditor {
    fn new(_: &mut Context<WorkspaceView>) -> Self {
        Self {
            kind: ModifyKind::PressPull,
            editing: None,
            target: None,
            tool: None,
            fields: vec![],
            labels: vec![],
            copy: false,
            mode: 0,
            material_action: Action::PhysicalMaterial,
        }
    }
}
impl WorkspaceView {
    fn selected_body(&self) -> Option<Uuid> {
        self.body_faces.get(&self.selected).map(|(id, _)| *id)
    }
    fn face_color(&self, face: u32) -> [f32; 3] {
        self.body_faces
            .get(&face)
            .and_then(|(id, _)| self.design.body_styles.iter().find(|s| s.body == *id))
            .map_or([0.227, 0.376, 0.314], |s| s.color)
    }
    fn solid_fields(&mut self, fields: &[(&'static str, &str)], cx: &mut Context<Self>) {
        self.solid_editor.labels = fields.iter().map(|(label, _)| *label).collect();
        self.solid_editor.fields = fields
            .iter()
            .map(|(_, value)| cx.new(|cx| TextInput::new(value, cx)))
            .collect();
    }
    fn open_solid_modify(&mut self, kind: ModifyKind, cx: &mut Context<Self>) {
        if self.pending_candidate.is_some() {
            self.rebuild(cx);
        }
        self.mode = Mode::Solid;
        self.sketch = false;
        self.construction_cursor = None;
        self.before_construction = false;
        self.solid_editor.kind = kind;
        self.solid_editor.editing = None;
        self.solid_editor.target = self.selected_body().or_else(|| {
            self.design
                .current_modify_bodies()
                .last()
                .map(|(id, _)| *id)
        });
        self.solid_editor.tool = None;
        self.solid_editor.copy = false;
        self.solid_editor.mode = 0;
        let fields: &[(&str, &str)] = match kind {
            ModifyKind::Fillet => &[("Radius", "1 mm")],
            ModifyKind::Chamfer => &[("Distance", "1 mm")],
            ModifyKind::Shell => &[("Wall thickness", "1 mm")],
            ModifyKind::PressPull | ModifyKind::OffsetFace => &[("Signed distance", "1 mm")],
            ModifyKind::Draft => &[("Angle", "3 deg"), ("Neutral plane Z", "0 mm")],
            ModifyKind::Scale => &[
                ("Scale factor", "1.1"),
                ("Center X", "0 mm"),
                ("Center Y", "0 mm"),
                ("Center Z", "0 mm"),
            ],
            ModifyKind::MoveCopy => &[
                ("Translate X", "10 mm"),
                ("Translate Y", "0 mm"),
                ("Translate Z", "0 mm"),
                ("Rotate about Z", "0 deg"),
            ],
            ModifyKind::SplitBody | ModifyKind::SplitFace => &[
                ("Plane distance from origin", "5 mm"),
                ("Normal X", "0"),
                ("Normal Y", "0"),
                ("Normal Z", "1"),
            ],
            ModifyKind::ReplaceFace => &[("Replacement face number", "1")],
            _ => &[],
        };
        self.solid_fields(fields, cx);
        self.panel = Some(Panel::SolidModify);
        self.tool = Tool::Select;
    }
    fn open_modify_edit(&mut self, id: Uuid, cx: &mut Context<Self>) {
        let Some(edit) = self.design.solid_edits.iter().find(|e| e.id == id).cloned() else {
            return;
        };
        self.open_solid_modify(edit.kind, cx);
        self.solid_editor.editing = Some(id);
        self.solid_editor.target = Some(edit.target);
        self.solid_editor.tool = edit.tool;
        self.solid_editor.copy = edit.copy;
        self.solid_editor.mode = edit.mode;
        self.selected = edit.face;
        for (i, field) in self.solid_editor.fields.iter().enumerate() {
            let slot = if edit.kind == ModifyKind::ReplaceFace {
                1
            } else {
                i
            };
            let expression = edit.parameters[slot]
                .and_then(|pid| self.design.parameters.iter().find(|p| p.id == pid))
                .map(|p| p.expression.clone())
                .unwrap_or_else(|| {
                    let angle = (edit.kind == ModifyKind::Draft && i == 0)
                        || (edit.kind == ModifyKind::MoveCopy && i == 3);
                    let scalar = (edit.kind == ModifyKind::Scale && i == 0)
                        || (matches!(edit.kind, ModifyKind::SplitBody | ModifyKind::SplitFace)
                            && i > 0)
                        || edit.kind == ModifyKind::ReplaceFace;
                    if scalar {
                        edit.values[slot].to_string()
                    } else if angle {
                        format!("{} deg", edit.values[slot].to_degrees())
                    } else {
                        format!("{} mm", edit.values[slot] * 1000.)
                    }
                });
            field.update(cx, |input, cx| input.set_content(expression, cx));
        }
        cx.notify();
    }
    fn open_materials(&mut self, action: Action, cx: &mut Context<Self>) {
        self.solid_editor.material_action = action;
        self.solid_editor.target = self.selected_body().or_else(|| {
            self.design
                .current_modify_bodies()
                .last()
                .map(|(id, _)| *id)
        });
        self.solid_fields(
            &[
                ("Material name", "Aluminium"),
                ("Density (kg/m³)", "2700"),
                ("Red (0–1)", "0.65"),
                ("Green (0–1)", "0.68"),
                ("Blue (0–1)", "0.72"),
            ],
            cx,
        );
        if let Some(style) = self
            .design
            .body_styles
            .iter()
            .find(|s| Some(s.body) == self.solid_editor.target)
        {
            let values = [
                style.material.clone(),
                style.density.to_string(),
                style.color[0].to_string(),
                style.color[1].to_string(),
                style.color[2].to_string(),
            ];
            self.solid_editor.fields = values
                .iter()
                .map(|value| cx.new(|cx| TextInput::new(value, cx)))
                .collect();
        }
        self.panel = Some(Panel::Materials);
    }
    fn solid_value(
        &self,
        index: usize,
        angular: bool,
        scalar: bool,
        cx: &Context<Self>,
    ) -> Result<f64, String> {
        let text = self.solid_editor.fields[index].read(cx).content.to_string();
        let mut d = Design {
            parameters: self.design.parameters.clone(),
            ..Design::default()
        };
        let id = d.parameter(
            &format!("modify_preview_{}", Uuid::new_v4().simple()),
            if scalar {
                text
            } else {
                crate::parameters::expression::dimension_input(&text, angular)
            },
        );
        let parameter = d.parameters.iter_mut().find(|p| p.id == id).unwrap();
        parameter.angular = angular;
        parameter.scalar = scalar;
        crate::parameters::expression::evaluate(&d).map(|values| values[&id])
    }
    fn apply_solid_modify(&mut self, cx: &mut Context<Self>) {
        if self.pending_candidate.is_some() {
            return;
        }
        let result = (|| {
            let kind = self.solid_editor.kind;
            let target = self
                .solid_editor
                .target
                .ok_or("Create or choose a body first")?;
            let mut values = [0.; 4];
            for (i, value) in values
                .iter_mut()
                .enumerate()
                .take(self.solid_editor.fields.len())
            {
                let angular = (kind == ModifyKind::Draft && i == 0)
                    || (kind == ModifyKind::MoveCopy && i == 3);
                let scalar = (kind == ModifyKind::Scale && i == 0)
                    || (matches!(kind, ModifyKind::SplitBody | ModifyKind::SplitFace) && i > 0)
                    || kind == ModifyKind::ReplaceFace;
                *value = self.solid_value(i, angular, scalar, cx)?;
            }
            if kind == ModifyKind::ReplaceFace {
                values[1] = values[0];
                values[0] = 0.;
            }
            let mut candidate = self.design.clone();
            let id = self.solid_editor.editing.unwrap_or_else(Uuid::new_v4);
            let mut bindings = [None; 4];
            for (i, field) in self.solid_editor.fields.iter().enumerate() {
                let angular = (kind == ModifyKind::Draft && i == 0)
                    || (kind == ModifyKind::MoveCopy && i == 3);
                let scalar = (kind == ModifyKind::Scale && i == 0)
                    || (matches!(kind, ModifyKind::SplitBody | ModifyKind::SplitFace) && i > 0)
                    || kind == ModifyKind::ReplaceFace;
                let text = field.read(cx).content.to_string();
                let expression = if scalar {
                    text
                } else {
                    crate::parameters::expression::dimension_input(&text, angular)
                };
                let slot = if kind == ModifyKind::ReplaceFace {
                    1
                } else {
                    i
                };
                let existing = candidate
                    .solid_edits
                    .iter()
                    .find(|e| e.id == id)
                    .and_then(|e| e.parameters[slot]);
                let pid = if let Some(pid) = existing {
                    candidate
                        .parameters
                        .iter_mut()
                        .find(|p| p.id == pid)
                        .ok_or("Missing solid edit parameter")?
                        .expression = expression;
                    pid
                } else {
                    candidate.parameter(&format!("modify_{}_{}", id.simple(), i), expression)
                };
                let p = candidate
                    .parameters
                    .iter_mut()
                    .find(|p| p.id == pid)
                    .unwrap();
                p.angular = angular;
                p.scalar = scalar;
                bindings[if kind == ModifyKind::ReplaceFace {
                    1
                } else {
                    i
                }] = Some(pid);
            }
            let edit = SolidEdit {
                id,
                kind,
                target,
                tool: self.solid_editor.tool,
                face: self.selected,
                values,
                parameters: bindings,
                copy: self.solid_editor.copy,
                mode: self.solid_editor.mode,
            };
            edit.validate()?;
            if let Some(index) = candidate.solid_edits.iter().position(|e| e.id == id) {
                candidate.solid_edits[index] = edit;
            } else {
                candidate.solid_edits.push(edit);
            }
            candidate.sync_construction();
            candidate.validate()?;
            Ok(candidate)
        })();
        match result {
            Ok(candidate) => self.validate_candidate(candidate, cx),
            Err(error) => self.error = Some(error),
        }
    }
    fn apply_material(&mut self, cx: &mut Context<Self>) {
        let result = (|| {
            let name = self.solid_editor.fields[0].read(cx).content.to_string();
            let density = self.solid_value(1, false, true, cx)?;
            let color = [
                self.solid_value(2, false, true, cx)? as f32,
                self.solid_value(3, false, true, cx)? as f32,
                self.solid_value(4, false, true, cx)? as f32,
            ];
            let mut candidate = self.design.clone();
            if self.solid_editor.material_action == Action::ManageMaterials {
                if let Some(material) = candidate.materials.iter_mut().find(|m| m.name == name) {
                    material.density = density;
                    material.color = color;
                } else {
                    candidate.materials.push(Material {
                        name,
                        density,
                        color,
                    });
                }
            } else {
                let body = self.solid_editor.target.ok_or("Choose a body first")?;
                let existing = candidate
                    .body_styles
                    .iter()
                    .find(|s| s.body == body)
                    .cloned();
                candidate.body_styles.retain(|s| s.body != body);
                let style = if self.solid_editor.material_action == Action::Appearance {
                    BodyStyle {
                        body,
                        material: existing.as_ref().map_or(name, |s| s.material.clone()),
                        density: existing.map_or(density, |s| s.density),
                        color,
                    }
                } else {
                    BodyStyle {
                        body,
                        material: name,
                        density,
                        color,
                    }
                };
                candidate.body_styles.push(style);
            }
            candidate.validate()?;
            Ok(candidate)
        })();
        match result {
            Ok(candidate) => {
                self.checkpoint();
                self.design = candidate;
                self.rebuild(cx);
            }
            Err(error) => self.error = Some(error),
        }
    }
    fn solid_modify_content(
        &self,
        mut content: Stateful<Div>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let materials = self.panel == Some(Panel::Materials);
        let kind = self.solid_editor.kind;
        let needs_tool = !materials
            && matches!(
                kind,
                ModifyKind::Combine
                    | ModifyKind::ReplaceFace
                    | ModifyKind::Align
                    | ModifyKind::SilhouetteSplit
            );
        if !materials || self.solid_editor.material_action != Action::ManageMaterials {
            content = content.child("Target body");
            for (id, name) in self.design.current_modify_bodies() {
                content = content.child(
                    text_button(
                        SharedString::from(format!("modify-target-{id}")),
                        format!(
                            "{}{}",
                            if self.solid_editor.target == Some(id) {
                                "● "
                            } else {
                                ""
                            },
                            name
                        ),
                        "Choose target body",
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.solid_editor.target = Some(id);
                        this.selected = 0;
                        cx.notify();
                    })),
                );
                if needs_tool {
                    content = content.child(
                        text_button(
                            SharedString::from(format!("modify-tool-{id}")),
                            format!(
                                "{}Tool: {}",
                                if self.solid_editor.tool == Some(id) {
                                    "● "
                                } else {
                                    ""
                                },
                                name
                            ),
                            "Choose tool body",
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.solid_editor.tool = Some(id);
                            cx.notify();
                        })),
                    );
                }
            }
        }
        if !materials {
            content = content.child(format!("Selected face: {}", self.selected));
            let help = match kind {
                ModifyKind::Fillet | ModifyKind::Chamfer => {
                    "Select a face to edit its boundary edges, or clear selection for all edges."
                }
                ModifyKind::PressPull | ModifyKind::OffsetFace => {
                    "Select a planar face. Positive distance adds material; negative removes it."
                }
                ModifyKind::Shell => "Select the face to remove for the shell opening.",
                ModifyKind::Draft => {
                    "Select a side face. Pull direction is +Z; the neutral plane is horizontal."
                }
                ModifyKind::ReplaceFace => {
                    "Select a planar face. Choose a tool body and a parallel planar replacement face."
                }
                ModifyKind::SilhouetteSplit => {
                    "Select a planar face and a tool body. Its visible outline projects along the face normal."
                }
                ModifyKind::Align => {
                    "Align the target body's center of mass with the tool body's center of mass."
                }
                ModifyKind::Delete | ModifyKind::Remove | ModifyKind::Simplify => {
                    "Select a face to remove and heal its feature. Clear selection to operate on the whole body."
                }
                _ => "Select the target body and enter the edit values.",
            };
            content = content.child(
                div()
                    .text_size(px(12.))
                    .text_color(rgb(t::MUTED))
                    .child(help),
            );
            content = content.child(
                text_button(
                    "clear-modify-face",
                    "Clear face selection",
                    "Operate on the whole body",
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.selected = 0;
                    cx.notify();
                })),
            );
        }
        if materials
            && let Some(style) = self
                .design
                .body_styles
                .iter()
                .find(|s| Some(s.body) == self.solid_editor.target)
        {
            content = content.child(format!(
                "Assigned: {} · {} kg/m³",
                style.material, style.density
            ));
            if self.design.current_modify_bodies().len() == 1
                && let Some(volume) = self.volume
            {
                content = content.child(format!("Mass: {:.3} g", volume * style.density * 1000.));
            }
        }
        for (label, field) in self
            .solid_editor
            .labels
            .iter()
            .zip(&self.solid_editor.fields)
        {
            content = content.child(*label).child(field.clone());
        }
        if !materials
            && matches!(
                kind,
                ModifyKind::MoveCopy | ModifyKind::Scale | ModifyKind::Align | ModifyKind::Combine
            )
        {
            content = content.child(
                text_button(
                    "modify-copy",
                    if self.solid_editor.copy {
                        "● Keep original / tool"
                    } else {
                        "Keep original / tool"
                    },
                    "Toggle keeping the original or tool body",
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.solid_editor.copy = !this.solid_editor.copy;
                    cx.notify();
                })),
            );
        }
        if !materials && kind == ModifyKind::Combine {
            for (mode, label) in [(0, "Join"), (1, "Cut"), (2, "Intersect")] {
                content = content.child(
                    text_button(
                        ("combine-mode", mode),
                        format!(
                            "{}{}",
                            if self.solid_editor.mode == mode {
                                "● "
                            } else {
                                ""
                            },
                            label
                        ),
                        "Choose Boolean operation",
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.solid_editor.mode = mode;
                        cx.notify();
                    })),
                );
            }
        }
        if materials {
            let presets = Material::presets();
            for material in self.design.materials.iter().chain(
                presets
                    .iter()
                    .filter(|p| !self.design.materials.iter().any(|m| m.name == p.name)),
            ) {
                let material = material.clone();
                let remove_name = material.name.clone();
                content = content.child(
                    text_button(
                        SharedString::from(format!("saved-material-{}", material.name)),
                        material.name.clone(),
                        "Load saved material",
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let values = [
                            material.name.clone(),
                            material.density.to_string(),
                            material.color[0].to_string(),
                            material.color[1].to_string(),
                            material.color[2].to_string(),
                        ];
                        this.solid_editor.fields = values
                            .iter()
                            .map(|value| cx.new(|cx| TextInput::new(value, cx)))
                            .collect();
                        cx.notify();
                    })),
                );
                if self.solid_editor.material_action == Action::ManageMaterials
                    && self.design.materials.iter().any(|m| m.name == remove_name)
                {
                    let name = remove_name;
                    content = content.child(
                        text_button(
                            SharedString::from(format!("remove-material-{name}")),
                            format!("Remove {name}"),
                            "Remove this material from the document library",
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.checkpoint();
                            this.design.materials.retain(|m| m.name != name);
                            this.rebuild(cx);
                        })),
                    );
                }
            }
        }
        content.child(
            text_button(
                "apply-solid-edit",
                if self.pending_candidate.is_some() {
                    "Validating…"
                } else {
                    "Apply"
                },
                "Confirm edit · Enter",
            )
            .on_click(cx.listener(|this, _, window, cx| this.confirm_editor(window, cx))),
        )
    }
}
