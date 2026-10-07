// Included in the desktop workspace. Keep Create controls isolated from other menus.
struct CreateEditor {
    kind: crate::model::solid_create::CreateKind,
    fields: Vec<Entity<TextInput>>,
    sketch: Option<Uuid>,
    second_sketch: Option<Uuid>,
    target: Option<Uuid>,
    second_target: Option<Uuid>,
    editing: Option<Uuid>,
}
impl WorkspaceView {
    fn open_create(
        &mut self,
        kind: crate::model::solid_create::CreateKind,
        editing: Option<Uuid>,
        cx: &mut Context<Self>,
    ) {
        if self.pending_candidate.is_some() {
            self.rebuild(cx);
        }
        self.error = None;
        let existing = editing
            .and_then(|id| self.design.create_features.iter().find(|f| f.id == id))
            .cloned();
        let bodies = self.design.current_create_bodies();
        let selected_body = self
            .body_faces
            .get(&self.selected)
            .map(|b| b.0)
            .filter(|id| bodies.iter().any(|b| b.0 == *id));
        let fields = kind
            .fields()
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let expression = existing
                    .as_ref()
                    .and_then(|f| f.parameters.get(i))
                    .and_then(|id| self.design.parameters.iter().find(|p| p.id == *id))
                    .map_or(field.default, |p| p.expression.as_str());
                cx.new(|cx| TextInput::new(expression, cx))
            })
            .collect();
        let sketch = if kind.uses_profile() {
            existing
                .as_ref()
                .and_then(|f| f.sketch)
                .or(self.design.current_sketch_id())
        } else {
            None
        };
        let second_sketch = existing.as_ref().and_then(|f| f.second_sketch).or_else(|| {
            self.design
                .construction
                .iter()
                .find(|f| matches!(f.kind, ConstructionKind::Sketch) && Some(f.id) != sketch)
                .map(|f| f.id)
        });
        self.create_editor = Some(CreateEditor {
            kind,
            fields,
            sketch,
            second_sketch,
            target: if kind.uses_body() {
                existing
                    .as_ref()
                    .and_then(|f| f.target)
                    .or(selected_body)
                    .or_else(|| bodies.last().map(|b| b.0))
            } else {
                None
            },
            second_target: existing
                .as_ref()
                .and_then(|f| f.second_target)
                .or_else(|| bodies.first().map(|b| b.0)),
            editing,
        });
        self.mode = Mode::Solid;
        self.sketch = false;
        self.tool = Tool::Select;
        self.panel = Some(Panel::Create);
        self.construction_cursor = None;
        self.before_construction = false;
        self.refresh_mesh();
        cx.notify();
    }
    fn apply_create(&mut self, cx: &mut Context<Self>) {
        if self.pending_candidate.is_some() {
            return;
        }
        use crate::model::solid_create::{CreateFeature, CreateKind, Unit};
        let Some(editor) = &self.create_editor else {
            return;
        };
        let mut candidate = self.design.clone();
        let existing = editor
            .editing
            .and_then(|id| candidate.create_features.iter().find(|f| f.id == id))
            .cloned();
        let id = editor.editing.unwrap_or_else(Uuid::new_v4);
        let mut parameters = vec![];
        for (i, (field, input)) in editor.kind.fields().iter().zip(&editor.fields).enumerate() {
            let text = input.read(cx).content.to_string();
            let expression = if field.unit == Unit::Number {
                text
            } else {
                crate::parameters::expression::dimension_input(&text, field.unit == Unit::Angle)
            };
            let pid =
                if let Some(pid) = existing.as_ref().and_then(|f| f.parameters.get(i)).copied() {
                    if let Some(p) = candidate.parameters.iter_mut().find(|p| p.id == pid) {
                        p.expression = expression;
                    }
                    pid
                } else {
                    let tool = editor.kind.name().to_ascii_lowercase().replace(' ', "_");
                    let label: String = field
                        .label
                        .to_ascii_lowercase()
                        .chars()
                        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                        .collect();
                    let mut suffix = candidate.create_features.len() + 1;
                    let mut name = format!("{tool}_{suffix}_{label}");
                    while candidate.parameters.iter().any(|p| p.name == name) {
                        suffix += 1;
                        name = format!("{tool}_{suffix}_{label}");
                    }
                    let pid = candidate.parameter(&name, expression);
                    let p = candidate
                        .parameters
                        .iter_mut()
                        .find(|p| p.id == pid)
                        .unwrap();
                    p.angular = field.unit == Unit::Angle;
                    p.scalar = field.unit == Unit::Number;
                    pid
                };
            parameters.push(pid);
        }
        let feature = CreateFeature {
            id,
            name: existing.as_ref().map_or_else(
                || {
                    format!(
                        "{} {}",
                        editor.kind.name(),
                        candidate.create_features.len() + 1
                    )
                },
                |f| f.name.clone(),
            ),
            kind: editor.kind,
            parameters,
            sketch: editor.sketch,
            second_sketch: if editor.kind == CreateKind::Loft {
                editor.second_sketch
            } else {
                None
            },
            boundary: existing
                .as_ref()
                .filter(|f| f.sketch == editor.sketch)
                .map_or_else(
                    || {
                        self.sketch_region
                            .as_ref()
                            .map_or_else(|| self.selection.clone(), |r| r.boundary.clone())
                    },
                    |f| f.boundary.clone(),
                ),
            target: editor.target,
            second_target: if editor.kind == CreateKind::BoundaryFill {
                editor.second_target
            } else {
                None
            },
        };
        // Sketch selection may also contain points/constraints; retain only curves.
        let mut feature = feature;
        if let Some(sketch) = feature.sketch {
            if let Ok(input) = candidate.sketch_input(sketch) {
                let curves = crate::sketch::entities::curve_ids(&input);
                feature.boundary.retain(|id| {
                    curves.contains(id) || crate::sketch::regions::is_boundary_token(id)
                });
            }
        } else {
            feature.boundary.clear();
        }
        if let Some(index) = candidate.create_features.iter().position(|f| f.id == id) {
            candidate.create_features[index] = feature;
        } else {
            candidate.create_features.push(feature);
        }
        candidate.sync_construction();
        self.validate_candidate(candidate, cx);
    }
    fn create_controls(&self, cx: &mut Context<Self>) -> AnyElement {
        use crate::model::solid_create::CreateKind;
        let Some(editor) = &self.create_editor else {
            return div().into_any_element();
        };
        let mut content = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_color(rgb(t::MUTED)).child(editor.kind.help()));
        for (field, input) in editor.kind.fields().iter().zip(&editor.fields) {
            content = content.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(field.label)
                    .child(input.clone()),
            );
        }
        if editor.kind.uses_profile() {
            content = content.child("Profile sketch");
            for sketch in self
                .design
                .construction
                .iter()
                .filter(|f| matches!(f.kind, ConstructionKind::Sketch))
            {
                let id = sketch.id;
                content = content.child(
                    text_button(
                        SharedString::from(format!("create-profile-{id}")),
                        sketch.name.clone(),
                        "Choose profile sketch",
                    )
                    .when(editor.sketch == Some(id), |el| el.bg(rgb(t::SELECTED)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(e) = &mut this.create_editor {
                            e.sketch = Some(id);
                        }
                        this.selection.clear();
                        cx.notify();
                    })),
                );
            }
        }
        if editor.kind == CreateKind::Loft {
            content = content.child("Second section sketch");
            for sketch in self.design.construction.iter().filter(|f| {
                matches!(f.kind, ConstructionKind::Sketch) && Some(f.id) != editor.sketch
            }) {
                let id = sketch.id;
                content = content.child(
                    text_button(
                        SharedString::from(format!("create-second-{id}")),
                        sketch.name.clone(),
                        "Choose second section",
                    )
                    .when(editor.second_sketch == Some(id), |el| {
                        el.bg(rgb(t::SELECTED))
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(e) = &mut this.create_editor {
                            e.second_sketch = Some(id);
                        }
                        cx.notify();
                    })),
                );
            }
        }
        if editor.kind.uses_body() {
            content = content.child("Target body");
            let mut bodies = self.design.current_create_bodies();
            if let Some(id) = editor.editing {
                if let Some(f) = self.design.create_features.iter().find(|f| f.id == id) {
                    for target in [f.target, f.second_target].into_iter().flatten() {
                        if !bodies.iter().any(|b| b.0 == target) {
                            bodies.push((target, "Original input body".into()));
                        }
                    }
                }
                bodies.retain(|b| b.0 != id);
            }
            for (id, name) in &bodies {
                let id = *id;
                content = content.child(
                    text_button(
                        SharedString::from(format!("create-target-{id}")),
                        name.clone(),
                        "Choose target body",
                    )
                    .when(editor.target == Some(id), |el| el.bg(rgb(t::SELECTED)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(e) = &mut this.create_editor {
                            e.target = Some(id);
                        }
                        cx.notify();
                    })),
                );
            }
            if bodies.is_empty() {
                content = content.child("Create a body first.");
            }
            if editor.kind == CreateKind::BoundaryFill {
                content = content.child("Second tool body");
                for (id, name) in bodies.iter().filter(|b| Some(b.0) != editor.target) {
                    let id = *id;
                    content = content.child(
                        text_button(
                            SharedString::from(format!("create-tool-{id}")),
                            name.clone(),
                            "Choose second body",
                        )
                        .when(editor.second_target == Some(id), |el| {
                            el.bg(rgb(t::SELECTED))
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(e) = &mut this.create_editor {
                                e.second_target = Some(id);
                            }
                            cx.notify();
                        })),
                    );
                }
            }
        }
        content
            .child(
                text_button(
                    "confirm-create",
                    if self.pending_candidate.is_some() {
                        "Validating…"
                    } else {
                        "Apply"
                    },
                    "Create solid feature · Enter",
                )
                .on_click(cx.listener(|this, _, window, cx| this.confirm_editor(window, cx))),
            )
            .into_any_element()
    }
}
