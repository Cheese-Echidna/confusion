// Viewport pull tabs share the editor's parameter inputs and stay anchored to geometry.
impl WorkspaceView {
    fn parameter_handles(&self, cx: &mut Context<Self>) -> Vec<Stateful<Div>> {
        use crate::model::solid_create::Unit;
        let fields: Vec<(Entity<TextInput>, String, Unit)> = match self.panel {
            Some(Panel::Create) => self
                .create_editor
                .as_ref()
                .map(|e| {
                    e.fields
                        .iter()
                        .zip(e.kind.fields())
                        .map(|(input, field)| (input.clone(), field.label.to_string(), field.unit))
                        .collect()
                })
                .unwrap_or_default(),
            Some(Panel::SolidModify) => self
                .solid_editor
                .fields
                .iter()
                .enumerate()
                .map(|(i, input)| {
                    let slot = if self.solid_editor.kind == ModifyKind::ReplaceFace {
                        1
                    } else {
                        i
                    };
                    let unit = match self.solid_editor.kind.parameter_unit(slot) {
                        Some(2) => Unit::Angle,
                        Some(1) => Unit::Length,
                        _ => Unit::Number,
                    };
                    (input.clone(), self.solid_editor.labels[i].to_string(), unit)
                })
                .collect(),
            Some(Panel::Fillet | Panel::Offset) => vec![(
                self.dimension.clone(),
                if self.panel == Some(Panel::Fillet) {
                    "Radius"
                } else {
                    "Distance"
                }
                .into(),
                Unit::Length,
            )],
            Some(Panel::Transform) => self
                .panel_fields()
                .into_iter()
                .map(|input| {
                    let (label, unit) = if input.entity_id() == self.transform_angle.entity_id() {
                        ("Angle", Unit::Angle)
                    } else if input.entity_id() == self.transform_count.entity_id() {
                        (
                            if self.transform_action == Action::Scale {
                                "Scale"
                            } else {
                                "Count"
                            },
                            Unit::Number,
                        )
                    } else if input.entity_id() == self.transform_rows.entity_id() {
                        ("Rows", Unit::Number)
                    } else if input.entity_id() == self.transform_x.entity_id() {
                        ("X distance", Unit::Length)
                    } else {
                        ("Y distance", Unit::Length)
                    };
                    (input, label.to_string(), unit)
                })
                .collect(),
            _ => vec![],
        };
        let Some(bounds) = self.bounds.get() else {
            return vec![];
        };
        let size = [
            f64::from(bounds.size.width) as u32,
            f64::from(bounds.size.height) as u32,
        ];
        let anchor = self
            .mesh
            .as_ref()
            .and_then(|(vertices, _)| {
                let points: Vec<_> = vertices
                    .iter()
                    .filter(|v| self.selected == 0 || v.face == self.selected)
                    .collect();
                if points.is_empty() {
                    return None;
                }
                let center = std::array::from_fn::<_, 3, _>(|k| {
                    points.iter().map(|v| v.position[k] as f64).sum::<f64>() / points.len() as f64
                });
                self.camera.project(nalgebra::Point3::from(center), size)
            })
            .or_else(|| {
                let d = self.display_design();
                let points: Vec<_> = d
                    .points
                    .iter()
                    .filter(|p| {
                        self.selection.is_empty()
                            || self.selection.contains(&p.id)
                            || self.selection.iter().any(|id| {
                                crate::sketch::entities::curve_points(&d, *id).contains(&p.id)
                            })
                    })
                    .collect();
                if points.is_empty() {
                    return None;
                }
                let center = [
                    points.iter().map(|p| p.xy[0]).sum::<f64>() / points.len() as f64,
                    points.iter().map(|p| p.xy[1]).sum::<f64>() / points.len() as f64,
                ];
                self.camera.project(
                    nalgebra::Point3::from(self.active_frame().world(center).coords * 25.),
                    size,
                )
            })
            .unwrap_or([size[0] as f64 * 0.5, size[1] as f64 * 0.5]);
        fields
            .into_iter()
            .enumerate()
            .map(|(i, (input, label, unit))| {
                let integer = matches!(
                    label.as_str(),
                    "Count" | "Rows" | "Columns" | "Teeth" | "Replacement face number"
                ) || label.starts_with("Plane (");
                let content = input.read(cx).content.to_string();
                div()
                    .id(SharedString::from(format!("parameter-handle-{i}")))
                    .absolute()
                    .left(px(
                        (anchor[0] as f32 + 36.).clamp(8., (size[0] as f32 - 240.).max(8.))
                    ))
                    .top(px((anchor[1] as f32 + i as f32 * 40.)
                        .clamp(8., (size[1] as f32 - 40.).max(8.))))
                    .w(px(210.))
                    .h(px(32.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .bg(rgb(t::PANEL))
                    .border_1()
                    .border_color(rgb(t::WARNING))
                    .rounded_md()
                    .cursor_pointer()
                    .child(
                        div()
                            .relative()
                            .w(px(32.))
                            .h(px(18.))
                            .child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .right_0()
                                    .top(px(8.))
                                    .h(px(2.))
                                    .bg(rgb(t::WARNING)),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .left(px(12.))
                                    .top(px(2.))
                                    .w(px(8.))
                                    .h(px(14.))
                                    .bg(rgb(t::WARNING))
                                    .rounded_sm(),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .child(format!("{label} · {content}")),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                            cx.stop_propagation();
                            let mut d = this.design.clone();
                            let expression = if unit == Unit::Number {
                                input.read(cx).content.to_string()
                            } else {
                                crate::parameters::expression::dimension_input(
                                    &input.read(cx).content,
                                    unit == Unit::Angle,
                                )
                            };
                            let id = d.parameter(
                                &format!("handle_{}", Uuid::new_v4().simple()),
                                expression,
                            );
                            let p = d.parameters.iter_mut().find(|p| p.id == id).unwrap();
                            p.angular = unit == Unit::Angle;
                            p.scalar = unit == Unit::Number;
                            if let Ok(values) = crate::parameters::expression::evaluate(&d) {
                                this.parameter_drag = Some((
                                    input.clone(),
                                    event.position,
                                    values[&id],
                                    unit,
                                    integer,
                                ));
                            }
                        }),
                    )
            })
            .collect()
    }
}
