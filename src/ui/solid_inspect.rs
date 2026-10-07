// Included in the desktop workspace implementation to keep inspection UI local.
impl WorkspaceView {
    fn center_of_mass_marker(&self) -> Option<Div> {
        if self.panel != Some(Panel::Inspect) || self.inspection_action != Action::CenterOfMass {
            return None;
        }
        let data = self.inspection.as_ref()?;
        let bounds = self.bounds.get()?;
        let size = [
            f32::from(bounds.size.width).max(1.) as u32,
            f32::from(bounds.size.height).max(1.) as u32,
        ];
        let [x, y] = self.camera.project(
            nalgebra::Point3::new(data.cx * 25., data.cy * 25., data.cz * 25.),
            size,
        )?;
        Some(
            div()
                .absolute()
                .left(px(x as f32 - 8.))
                .top(px(y as f32 - 8.))
                .size(px(16.))
                .rounded_full()
                .border_2()
                .border_color(rgb(t::WARNING))
                .child(
                    div()
                        .absolute()
                        .left(px(5.))
                        .top(px(5.))
                        .size(px(2.))
                        .bg(rgb(t::WARNING)),
                ),
        )
    }
    fn inspection_colors(&self, vertices: &[DemoVertex]) -> Option<Vec<DemoVertex>> {
        if self.panel != Some(Panel::Inspect)
            || !matches!(
                self.inspection_action,
                Action::CurvatureAnalysis | Action::DraftAnalysis
            )
        {
            return None;
        }
        let data = self.inspection.as_ref()?;
        let maximum = data
            .faces
            .iter()
            .filter(|f| f.samples > 0)
            .map(|f| f.max_curvature)
            .fold(0., f64::max);
        Some(
            vertices
                .iter()
                .map(|v| {
                    let mut tinted = *v;
                    if let Some(face) = data
                        .faces
                        .iter()
                        .find(|f| f.face == v.face && f.samples > 0)
                    {
                        tinted.color = if self.inspection_action == Action::CurvatureAnalysis {
                            let fraction = if maximum > 0. {
                                (face.max_curvature / maximum) as f32
                            } else {
                                0.
                            };
                            [0.15 + 0.8 * fraction, 0.35, 0.95 - 0.8 * fraction]
                        } else {
                            let draft = (face.min_draft + face.max_draft) * 0.5;
                            if draft < -1. {
                                [0.9, 0.2, 0.15]
                            } else if draft > 1. {
                                [0.15, 0.75, 0.35]
                            } else {
                                [0.95, 0.7, 0.15]
                            }
                        };
                    }
                    tinted
                })
                .collect(),
        )
    }
    fn inspection_title(&self) -> &'static str {
        match self.inspection_action {
            Action::SectionAnalysis => "Section analysis",
            Action::Interference => "Interference",
            Action::CenterOfMass => "Center of mass",
            Action::CurvatureAnalysis => "Curvature analysis",
            Action::DraftAnalysis => "Draft analysis",
            Action::ValidateSolid => "Validate solid",
            _ => "Measure solid",
        }
    }
    fn inspection_lines(&self) -> Vec<String> {
        if self.inspection_action == Action::Measure && !self.selected_vertices.is_empty() {
            let points: Vec<_> = self
                .selected_vertices
                .iter()
                .filter_map(|i| self.body_vertices.get(*i))
                .map(|v| &v.point)
                .collect();
            if let [a, b] = points.as_slice() {
                return vec![format!(
                    "Distance: {:.4} mm",
                    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt()
                        * 1000.
                )];
            }
            return points
                .iter()
                .map(|p| {
                    format!(
                        "Vertex: ({:.3}, {:.3}, {:.3}) mm",
                        p.x * 1000.,
                        p.y * 1000.,
                        p.z * 1000.
                    )
                })
                .collect();
        }
        let Some(data) = &self.inspection else {
            return vec!["Create a solid and wait for evaluation to finish.".into()];
        };
        match self.inspection_action {
            Action::CenterOfMass => vec![
                "Uniform density · all evaluated bodies".into(),
                format!("X: {:.4} mm", data.cx * 1000.),
                format!("Y: {:.4} mm", data.cy * 1000.),
                format!("Z: {:.4} mm", data.cz * 1000.),
            ],
            Action::Interference => {
                let mut lines = vec!["Exact intersections between evaluated bodies".into()];
                for pair in &data.interference {
                    lines.push(format!(
                        "Bodies {} / {}: {:.4} mm³",
                        pair.first,
                        pair.second,
                        pair.volume * 1e9
                    ));
                }
                if !data.error.is_empty() {
                    lines.push(data.error.clone());
                } else if data.interference.is_empty() {
                    lines.push("No volumetric interference detected".into());
                }
                lines
            }
            Action::ValidateSolid => vec![
                if data.valid {
                    "OCCT geometry check passed".into()
                } else {
                    "OCCT geometry check failed".into()
                },
                format!("{} solids · {} faces", data.solids, data.faces.len()),
                format!("Volume: {:.4} mm³", self.volume.unwrap_or(0.) * 1e9),
            ],
            Action::CurvatureAnalysis | Action::DraftAnalysis => {
                let curvature = self.inspection_action == Action::CurvatureAnalysis;
                let mut lines = vec![if curvature {
                    "Principal curvature magnitudes · 5 × 5 trimmed surface samples per face (1/mm)"
                        .into()
                } else {
                    "Signed draft to global +Z · 5 × 5 trimmed surface samples per face · 0° is vertical".into()
                }];
                for face in data
                    .faces
                    .iter()
                    .filter(|f| self.selected == 0 || f.face == self.selected)
                {
                    lines.push(if face.samples == 0 {
                        format!("Face {}: no regular interior samples", face.face)
                    } else if curvature {
                        format!(
                            "Face {}: {:.6} … {:.6} 1/mm",
                            face.face,
                            face.min_curvature / 1000.,
                            face.max_curvature / 1000.
                        )
                    } else {
                        format!(
                            "Face {}: {:.2}° … {:.2}°",
                            face.face, face.min_draft, face.max_draft
                        )
                    });
                }
                lines.push(if curvature {
                    "Blue: low curvature · red: high curvature (relative to model)".into()
                } else {
                    "Red: negative draft · yellow: within ±1° · green: positive draft (face average)".into()
                });
                lines.push(
                    "Click a face to isolate results. Sample ranges are approximate extrema."
                        .into(),
                );
                lines
            }
            Action::SectionAnalysis => vec![
                "Tessellated cutaway · keeps the negative side of the plane".into(),
                "The cut is open; this preview does not alter the solid.".into(),
            ],
            _ => {
                let mut lines = vec![
                    format!(
                        "Volume (all bodies): {:.4} mm³",
                        self.volume.unwrap_or(0.) * 1e9
                    ),
                    format!("Surface area (all bodies): {:.4} mm²", data.area * 1e6),
                ];
                if let Some(face) = data.faces.iter().find(|f| f.face == self.selected) {
                    lines.push(format!(
                        "Face {} area: {:.4} mm²",
                        face.face,
                        face.area * 1e6
                    ));
                }
                if let Some((vertices, _)) = &self.mesh {
                    let mut min = [f32::INFINITY; 3];
                    let mut max = [f32::NEG_INFINITY; 3];
                    for v in vertices
                        .iter()
                        .filter(|v| self.selected == 0 || v.face == self.selected)
                    {
                        for axis in 0..3 {
                            min[axis] = min[axis].min(v.position[axis]);
                            max[axis] = max[axis].max(v.position[axis]);
                        }
                    }
                    if min[0].is_finite() {
                        lines.push(format!(
                            "{} bounds (mesh): {:.4} × {:.4} × {:.4} mm",
                            if self.selected == 0 { "Model" } else { "Face" },
                            (max[0] - min[0]) * 40.,
                            (max[1] - min[1]) * 40.,
                            (max[2] - min[2]) * 40.
                        ));
                    }
                }
                lines.push("Click a face to measure its area and bounds.".into());
                lines
            }
        }
    }
}
