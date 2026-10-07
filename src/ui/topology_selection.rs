// Pick exact kernel edges/vertices using their projected curves, never tessellation diagonals.
impl WorkspaceView {
    fn pick_topology(&mut self, position: Point<Pixels>, additive: bool) -> bool {
        let Some(bounds) = self.bounds.get() else {
            return false;
        };
        let size = [
            f64::from(bounds.size.width) as u32,
            f64::from(bounds.size.height) as u32,
        ];
        let cursor = [
            f64::from(position.x - bounds.origin.x),
            f64::from(position.y - bounds.origin.y),
        ];
        let project = |p: &crate::kernel::bridge::ffi::SpatialPoint| {
            self.camera
                .project(nalgebra::Point3::new(p.x * 25., p.y * 25., p.z * 25.), size)
        };
        let visible = |body: u32, part: u32| {
            self.evaluated_features
                .get(body as usize)
                .is_some_and(|id| !self.hidden_bodies.contains(&(*id, part)))
                && !self.hidden.contains("bodies")
        };
        let edge_tool = self.panel == Some(Panel::SolidModify)
            && matches!(
                self.solid_editor.kind,
                ModifyKind::Fillet | ModifyKind::Chamfer
            );
        if !edge_tool && self.panel != Some(Panel::Extrude) {
            let hit = self
                .body_vertices
                .iter()
                .enumerate()
                .filter(|(_, v)| visible(v.body, v.part))
                .filter_map(|(i, v)| {
                    let at = project(&v.point)?;
                    let distance = (at[0] - cursor[0]).hypot(at[1] - cursor[1]);
                    (distance < 9.).then_some((i, distance))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((i, _)) = hit {
                let measuring =
                    self.panel == Some(Panel::Inspect) && self.inspection_action == Action::Measure;
                if !additive && (!measuring || self.selected_vertices.len() >= 2) {
                    self.selected_vertices.clear();
                }
                if let Some(at) = self.selected_vertices.iter().position(|v| *v == i) {
                    self.selected_vertices.remove(at);
                } else {
                    self.selected_vertices.push(i);
                }
                self.selected_edges.clear();
                self.selected = 0;
                self.status = format!("{} body vertices selected", self.selected_vertices.len());
                return true;
            }
        }
        if self.panel == Some(Panel::Extrude) {
            return false;
        }
        let hit = self
            .body_edges
            .iter()
            .enumerate()
            .filter(|(_, e)| visible(e.body, e.part))
            .filter_map(|(i, e)| {
                let distance = e
                    .points
                    .windows(2)
                    .filter_map(|pair| {
                        let a = project(&pair[0])?;
                        let b = project(&pair[1])?;
                        let delta = [b[0] - a[0], b[1] - a[1]];
                        let length = delta[0].powi(2) + delta[1].powi(2);
                        let t = if length > 1e-8 {
                            ((cursor[0] - a[0]) * delta[0] + (cursor[1] - a[1]) * delta[1]) / length
                        } else {
                            0.
                        }
                        .clamp(0., 1.);
                        Some(
                            (cursor[0] - a[0] - delta[0] * t)
                                .hypot(cursor[1] - a[1] - delta[1] * t),
                        )
                    })
                    .fold(f64::INFINITY, f64::min);
                (distance < 8.).then_some((i, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((i, _)) = hit {
            let edge = &self.body_edges[i];
            if !additive && !edge_tool {
                self.selected_edges.clear();
            }
            // A modifying operation is confined to one body.
            if self
                .selected_edges
                .first()
                .is_some_and(|previous| self.body_edges[*previous].body != edge.body)
            {
                self.selected_edges.clear();
            }
            if let Some(at) = self
                .selected_edges
                .iter()
                .position(|selected| *selected == i)
            {
                self.selected_edges.remove(at);
            } else {
                self.selected_edges.push(i);
            }
            self.selected_vertices.clear();
            self.selected = edge.faces.first().copied().unwrap_or(0);
            if edge_tool {
                self.solid_editor.target = self.evaluated_features.get(edge.body as usize).copied();
                self.solid_editor.reference = None;
            }
            self.status = format!("{} edge(s) selected", self.selected_edges.len());
            return true;
        }
        false
    }
    fn topology_overlay(&self) -> impl IntoElement {
        let camera = self.camera.clone();
        let curves: Vec<Vec<[f64; 3]>> = self
            .selected_edges
            .iter()
            .filter_map(|i| self.body_edges.get(*i))
            .map(|e| e.points.iter().map(|p| [p.x, p.y, p.z]).collect())
            .collect();
        let vertices: Vec<_> = self
            .selected_vertices
            .iter()
            .filter_map(|i| self.body_vertices.get(*i))
            .map(|v| [v.point.x, v.point.y, v.point.z])
            .collect();
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let project = |p: [f64; 3]| {
                    camera
                        .project(
                            nalgebra::Point3::from(p).map(|v| v * 25.),
                            [
                                f64::from(bounds.size.width) as u32,
                                f64::from(bounds.size.height) as u32,
                            ],
                        )
                        .map(|p| bounds.origin + point(px(p[0] as f32), px(p[1] as f32)))
                };
                for curve in &curves {
                    for pair in curve.windows(2) {
                        if let (Some(a), Some(b)) = (project(pair[0]), project(pair[1])) {
                            let mut path = PathBuilder::stroke(px(4.));
                            path.move_to(a);
                            path.line_to(b);
                            if let Ok(path) = path.build() {
                                window.paint_path(path, rgb(t::WARNING));
                            }
                        }
                    }
                }
                for vertex in &vertices {
                    if let Some(at) = project(*vertex) {
                        window.paint_quad(fill(
                            Bounds::new(at - point(px(5.), px(5.)), size(px(10.), px(10.))),
                            rgb(t::WARNING),
                        ));
                    }
                }
            },
        )
        .absolute()
        .inset_0()
    }
    fn pick_sketch_geometry(
        &mut self,
        position: Point<Pixels>,
        additive: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.panel == Some(Panel::SolidModify) {
            return false;
        }
        let Some(bounds) = self.bounds.get() else {
            return false;
        };
        let pixel = [
            f64::from(position.x - bounds.origin.x),
            f64::from(position.y - bounds.origin.y),
        ];
        let size = [
            f64::from(bounds.size.width) as u32,
            f64::from(bounds.size.height) as u32,
        ];
        let Some((origin, direction)) = self.camera.ray(pixel, size) else {
            return false;
        };
        let mut sketches = self.world_sketches.clone();
        if let Some(id) = self.design.current_sketch_id() {
            sketches.push((id, self.display_design(), self.active_frame()));
        }
        for (id, d, frame) in sketches {
            if self.hidden_sketches.contains(&id) || self.hidden.contains("sketches") {
                continue;
            }
            let Some(at) = frame.intersect(nalgebra::Point3::from(origin.coords / 25.), direction)
            else {
                continue;
            };
            let hit = d.points.iter().any(|p| {
                self.camera
                    .project(nalgebra::Point3::from(frame.world(p.xy).coords * 25.), size)
                    .is_some_and(|p| (p[0] - pixel[0]).hypot(p[1] - pixel[1]) < 8.)
            }) || crate::sketch::entities::closest(&d, at, 8. / self.scale).is_some()
                || (self.panel == Some(Panel::Extrude)
                    && crate::sketch::regions::at(
                        &d,
                        &d.points.iter().map(|p| p.xy).collect::<Vec<_>>(),
                        at,
                    )
                    .ok()
                    .flatten()
                    .is_some());
            if !hit {
                continue;
            }
            if self.design.current_sketch_id() != Some(id) {
                if self.design.activate_sketch(id).is_err() {
                    continue;
                }
                self.solved.clear();
                self.selection.clear();
            }
            self.extrusion_face = None;
            self.selected_edges.clear();
            self.selected_vertices.clear();
            self.selected = 0;
            self.sketch_click(position, additive, true, cx);
            self.sketch_drag = None;
            if self.panel == Some(Panel::Extrude) {
                self.camera
                    .set_direction(frame.x - frame.y + frame.normal * 0.65, frame.normal);
                if let Some(preview) = self.extrusion_preview(cx) {
                    self.camera.target =
                        nalgebra::Point3::from(preview.frame.world(preview.center).coords * 25.);
                }
                self.changed_camera();
            }
            return true;
        }
        false
    }
}
