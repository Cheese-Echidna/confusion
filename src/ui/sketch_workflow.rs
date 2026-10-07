// Sketch gesture, selection and command coordination, included in viewport's desktop module.
// Geometry edits and residuals remain in the headless sketch/solver modules.
impl WorkspaceView {
    fn sketch_position(&self, position: Point<Pixels>, suppress: bool) -> Option<[f64; 2]> {
        let b = self.bounds.get()?;
        let (origin, direction) = self.camera.ray(
            [
                f64::from(position.x - b.origin.x),
                f64::from(position.y - b.origin.y),
            ],
            [
                f64::from(b.size.width) as u32,
                f64::from(b.size.height) as u32,
            ],
        )?;
        let [x, y] = self
            .active_frame()
            .intersect(nalgebra::Point3::from(origin.coords / 25.), direction)?;
        if suppress {
            return Some([x, y]);
        }
        let point = if self.snap {
            [(x * 1000.).round() * 0.001, (y * 1000.).round() * 0.001]
        } else {
            [x, y]
        };
        let coords = if self.solved.len() == self.design.points.len() {
            self.solved.clone()
        } else {
            self.design.points.iter().map(|p| p.xy).collect()
        };
        Some(
            coords
                .into_iter()
                .find(|p| (p[0] - x).hypot(p[1] - y) * self.scale < 8.)
                .unwrap_or(point),
        )
    }
    fn tool_prompt(&self) -> &'static str {
        match self.tool {
            Tool::Point => "Point · Click to place",
            Tool::Select => "Select · Ctrl-click adds · Drag free points or curves",
            Tool::Line => "Line · Click endpoints · Esc to finish",
            Tool::Rectangle => "Rectangle · Click opposite corners",
            Tool::CenterRectangle => "Center rectangle · Click center and corner",
            Tool::Circle => "Circle · Click center and rim",
            Tool::Circle2 => "Circle · Click diameter endpoints",
            Tool::Circle3 => "Circle · Click three points",
            Tool::Arc => "Arc · Click center, start, end",
            Tool::Arc3 => "Arc · Click start, through point, end",
            Tool::Ellipse => "Ellipse · Click center, major axis, minor axis",
            Tool::Slot => "Slot · Click centerline endpoints, then width",
            Tool::Polygon => "Polygon · Click center and corner",
            Tool::Spline => "Spline · Click fit points · Enter to finish",
            Tool::Dimension => "Dimension · Select geometry · Enter value · Enter to confirm",
            Tool::Measure => "Measure · Select geometry · Ctrl-click a second entity",
            Tool::Break => "Break · Click a point inside a line or arc",
            Tool::Trim => "Trim · Click the segment to remove",
            Tool::Extend => "Extend · Click the end to extend",
        }
    }
    fn display_design(&self) -> Design {
        let mut d = self.design.clone();
        if self.solved.len() == d.points.len() {
            for (p, xy) in d.points.iter_mut().zip(&self.solved) {
                p.xy = *xy
            }
        }
        d
    }
    fn commit_sketch(&mut self, mut candidate: Design, cx: &mut Context<Self>) -> bool {
        candidate.sync_construction();
        let check = crate::parameters::expression::evaluate(&candidate)
            .and_then(|p| crate::solver::nonlinear::solve(&candidate, &p));
        match check {
            Ok(solution) if solution.conflicts.is_empty() => {
                self.checkpoint();
                self.design = candidate;
                self.solved = solution.points;
                self.point_dof = solution.point_dof;
                self.inputs.clear();
                self.rebuild(cx);
                true
            }
            Ok(_) => {
                self.error=Some("This edit would overconstrain the sketch. Remove a conflicting constraint or use a driven dimension.".into());
                cx.notify();
                false
            }
            Err(e) => {
                self.error = Some(e);
                cx.notify();
                false
            }
        }
    }
    fn sketch_click(
        &mut self,
        position: Point<Pixels>,
        additive: bool,
        suppress: bool,
        cx: &mut Context<Self>,
    ) {
        let additive =
            additive || self.pending_constraint.is_some() || self.panel == Some(Panel::Fillet);
        let Some(at) = self.sketch_position(position, suppress) else {
            return;
        };
        let d = self.display_design();
        let point_hit = d
            .points
            .iter()
            .filter_map(|p| {
                let distance = (p.xy[0] - at[0]).hypot(p.xy[1] - at[1]);
                (distance * self.scale < 8.).then_some((p.id, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|p| p.0);
        let curve = crate::sketch::entities::closest(&d, at, 8. / self.scale);
        if matches!(self.tool, Tool::Select | Tool::Dimension | Tool::Measure) {
            // Select a dimension by its label and edit its existing parameter in place.
            if self.tool == Tool::Select
                && self.show_dimensions
                && let Some(c) = d
                    .constraints
                    .iter()
                    .chain(&d.driven_dimensions)
                    .filter(|c| c.kind.parameter().is_some())
                    .find(|c| {
                        let p = self.dimension_anchor(&d, c);
                        self.dimension_hit(&d, c, at)
                            || self
                                .sketch_screen(p)
                                .zip(self.sketch_screen(at))
                                .is_some_and(|(p, at)| {
                                    (p[0] + 7. - at[0]).hypot(p[1] - 8. - at[1]) < 18.
                                })
                    })
            {
                let id = c.id;
                let anchor = self.dimension_anchor(&d, c);
                self.design.dimension_positions.entry(id).or_insert(anchor);
                self.dimension_edit = None;
                self.dimension_position = Some(anchor);
                self.selection = vec![id];
                self.dimension_drag = Some((id, self.design.clone(), false));
                cx.notify();
                return;
            }
            if self.tool == Tool::Dimension
                && !self.selection.is_empty()
                && point_hit.is_none()
                && curve.is_none()
            {
                self.dimension_position = Some(at);
                if let Ok(kind) =
                    crate::sketch::dimensions::kind_at(&d, &self.selection, Uuid::nil(), Some(at))
                    && let Some(v) = crate::sketch::dimensions::value(&d, &kind)
                {
                    let text = if matches!(kind, ConstraintKind::Angle { .. }) {
                        format!("{:.3} deg", v.to_degrees())
                    } else {
                        format!("{:.3} mm", v * 1000.)
                    };
                    self.dimension = cx.new(|cx| TextInput::new(&text, cx));
                }
                self.panel = Some(Panel::Dimension);
                cx.notify();
                return;
            }
            let hit = point_hit.or(curve);
            if self.tool == Tool::Select {
                self.sketch_region = None;
            }
            if !additive
                && self.tool == Tool::Select
                && hit.is_none_or(|id| !self.selection.contains(&id))
            {
                self.selection.clear()
            }
            if let Some(id) = hit {
                self.sketch_region = None;
                if let Some(i) = self.selection.iter().position(|p| *p == id) {
                    if additive {
                        self.selection.remove(i);
                    }
                } else {
                    self.selection.push(id)
                }
                self.line = curve.filter(|id| d.lines.iter().any(|l| l.id == *id));
                if self.tool == Tool::Select && !additive && self.panel != Some(Panel::Extrude) {
                    let mut points = Vec::new();
                    for id in &self.selection {
                        let curve = crate::sketch::entities::curve_points(&d, *id);
                        if curve.is_empty() {
                            points.push(*id)
                        } else {
                            points.extend(curve)
                        }
                    }
                    points.sort();
                    points.dedup();
                    self.sketch_drag = Some(SketchDrag {
                        before: d.clone(),
                        points,
                        start: at,
                        moved: false,
                    });
                }
            } else if self.tool == Tool::Select {
                let xy: Vec<_> = d.points.iter().map(|p| p.xy).collect();
                if let Ok(Some(region)) = crate::sketch::regions::at(&d, &xy, at) {
                    self.sketch_region = Some(region.clone());
                    for id in region.curves {
                        if !self.selection.contains(&id) {
                            self.selection.push(id);
                        }
                    }
                    self.line = None;
                    cx.notify();
                    return;
                }
                self.line = None;
                self.marquee = Some((at, at, additive));
            }
            if let Some(action) = self.pending_constraint {
                self.constrain_selection(action, cx);
            }
            if self.tool == Tool::Dimension {
                self.dimension_edit = None;
                self.panel = Some(Panel::Dimension);
                self.dimension_position = Some(at);
                if let Ok(kind) = crate::sketch::dimensions::kind(&d, &self.selection, Uuid::nil())
                    && let Some(v) = crate::sketch::dimensions::value(&d, &kind)
                {
                    let text = if matches!(kind, ConstraintKind::Angle { .. }) {
                        format!("{:.3} deg", v.to_degrees())
                    } else {
                        format!("{:.3} mm", v * 1000.)
                    };
                    self.dimension = cx.new(|cx| TextInput::new(&text, cx));
                }
            }
            cx.notify();
            return;
        }
        if matches!(self.tool, Tool::Break | Tool::Trim | Tool::Extend) {
            if let Some(id) = curve {
                let mut next = d;
                let result = match self.tool {
                    Tool::Break => crate::sketch::edit::split(&mut next, id, at),
                    Tool::Trim => crate::sketch::edit::trim(&mut next, id, at),
                    _ => crate::sketch::edit::extend(&mut next, id, at),
                };
                match result {
                    Ok(()) => {
                        self.commit_sketch(next, cx);
                        self.selection.clear();
                    }
                    Err(e) => self.error = Some(e),
                }
            }
            cx.notify();
            return;
        }
        if self.tool == Tool::Point {
            let mut next = d;
            next.ensure_sketch();
            next.point(at);
            self.commit_sketch(next, cx);
            return;
        }
        if matches!(
            self.tool,
            Tool::Circle2
                | Tool::Circle3
                | Tool::Arc3
                | Tool::Ellipse
                | Tool::Slot
                | Tool::Spline
                | Tool::Polygon
                | Tool::CenterRectangle
        ) {
            self.tool_points.push(at);
            if self.tool == Tool::Spline {
                cx.notify();
                return;
            }
            let count = if matches!(
                self.tool,
                Tool::Circle2 | Tool::Polygon | Tool::CenterRectangle
            ) {
                2
            } else {
                3
            };
            if self.tool_points.len() < count {
                cx.notify();
                return;
            }
            let points = self.tool_points.clone();
            let mut next = d;
            let result = match self.tool {
                Tool::Circle2 => {
                    let a = points[0];
                    let b = points[1];
                    crate::sketch::edit::circle(
                        &mut next,
                        [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5],
                        b,
                        None,
                    )
                    .map(|_| ())
                }
                Tool::Circle3 | Tool::Arc3 => crate::sketch::edit::circumcenter(
                    points[0], points[1], points[2],
                )
                .and_then(|center| {
                    if self.tool == Tool::Arc3 {
                        let a = points[0];
                        let b = points[1];
                        let c = points[2];
                        let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
                        let (start, end) = if cross > 0. { (a, c) } else { (c, a) };
                        crate::sketch::edit::circle(&mut next, center, start, Some(end)).map(|_| ())
                    } else {
                        crate::sketch::edit::circle(&mut next, center, points[0], None).map(|_| ())
                    }
                }),
                Tool::Ellipse => {
                    crate::sketch::edit::ellipse(&mut next, points[0], points[1], points[2])
                        .map(|_| ())
                }
                Tool::Slot => crate::sketch::edit::slot(&mut next, points[0], points[1], points[2]),
                Tool::Polygon => {
                    let sides = self.transform_count.read(cx).content.parse().unwrap_or(6);
                    crate::sketch::edit::polygon(&mut next, points[0], points[1], sides)
                }
                Tool::CenterRectangle => {
                    let a = points[0];
                    let b = points[1];
                    let corners = [
                        [2. * a[0] - b[0], 2. * a[1] - b[1]],
                        [b[0], 2. * a[1] - b[1]],
                        b,
                        [2. * a[0] - b[0], b[1]],
                    ];
                    for i in 0..4 {
                        let line = next.line(corners[i], corners[(i + 1) % 4]);
                        next.constrain(if i % 2 == 0 {
                            ConstraintKind::Horizontal { line }
                        } else {
                            ConstraintKind::Vertical { line }
                        })
                    }
                    let center = next.point(a);
                    let diagonals = [
                        next.line(corners[0], corners[2]),
                        next.line(corners[1], corners[3]),
                    ];
                    next.construction_geometry.extend(diagonals);
                    for line in diagonals {
                        next.constrain(ConstraintKind::Midpoint {
                            point: center,
                            line,
                        })
                    }
                    if !suppress && a == [0., 0.] {
                        next.constrain(ConstraintKind::Fixed {
                            point: center,
                            xy: a,
                        })
                    }
                    Ok(())
                }
                _ => Ok(()),
            };
            match result {
                Ok(()) => {
                    if self.commit_sketch(next, cx) {
                        self.tool_points.clear();
                    }
                }
                Err(e) => {
                    self.error = Some(e);
                    self.tool_points.pop();
                }
            }
            cx.notify();
            return;
        }
        if let Some(a) = self.anchor {
            if (at[0] - a[0]).hypot(at[1] - a[1]) < 1e-6 {
                return;
            }
            let mut next = d;
            match self.tool {
                Tool::Circle => {
                    if let Err(e) = crate::sketch::edit::circle(&mut next, a, at, None) {
                        self.error = Some(e);
                        return;
                    }
                    if !suppress && a == [0., 0.] {
                        let point = next.point(a);
                        if !next.constraints.iter().any(
                            |c| matches!(c.kind,ConstraintKind::Fixed{point:id,..} if id==point),
                        ) {
                            next.constrain(ConstraintKind::Fixed { point, xy: a })
                        }
                    }
                    if self.commit_sketch(next, cx) {
                        self.anchor = None
                    }
                }
                Tool::Arc => {
                    if let Some(start) = self.arc_start {
                        let radius = (start[0] - a[0]).hypot(start[1] - a[1]);
                        let angle = (at[1] - a[1]).atan2(at[0] - a[0]);
                        let end = [a[0] + radius * angle.cos(), a[1] + radius * angle.sin()];
                        if let Err(e) = crate::sketch::edit::circle(&mut next, a, start, Some(end))
                        {
                            self.error = Some(e);
                            return;
                        }
                        if self.commit_sketch(next, cx) {
                            self.anchor = None;
                            self.arc_start = None
                        }
                    } else {
                        self.arc_start = Some(at)
                    }
                }
                Tool::Rectangle => {
                    if (at[0] - a[0]).abs() < 1e-6 || (at[1] - a[1]).abs() < 1e-6 {
                        return;
                    }
                    // Fusion rectangles infer relationships; dimensions are added explicitly.
                    let corners = [a, [at[0], a[1]], at, [a[0], at[1]]];
                    for i in 0..4 {
                        let id = next.line(corners[i], corners[(i + 1) % 4]);
                        next.constrain(if i % 2 == 0 {
                            ConstraintKind::Horizontal { line: id }
                        } else {
                            ConstraintKind::Vertical { line: id }
                        });
                    }
                    if !suppress && a == [0., 0.] {
                        let point = next.point(a);
                        next.constrain(ConstraintKind::Fixed { point, xy: a })
                    }
                    if self.commit_sketch(next, cx) {
                        self.anchor = None
                    }
                }
                _ => {
                    let id = next.line(a, at);
                    if !suppress && (at[1] - a[1]).abs() < 1e-6 {
                        next.constrain(ConstraintKind::Horizontal { line: id })
                    }
                    if !suppress && (at[0] - a[0]).abs() < 1e-6 {
                        next.constrain(ConstraintKind::Vertical { line: id })
                    }
                    if !suppress && a == [0., 0.] {
                        let point = next.point(a);
                        if !next
                            .constraints
                            .iter()
                            .any(|c| matches!(c.kind,ConstraintKind::Fixed{point:p,..} if p==point))
                        {
                            next.constrain(ConstraintKind::Fixed { point, xy: a })
                        }
                    }
                    if self.commit_sketch(next, cx) {
                        self.anchor = Some(at)
                    }
                }
            }
        } else {
            self.anchor = Some(at)
        }
        cx.notify();
    }
    fn apply_constraint(&mut self, kind: u8, cx: &mut Context<Self>) {
        let action = match kind {
            0 => Action::Horizontal,
            1 => Action::Vertical,
            2 => Action::Fixed,
            _ => Action::Dimension,
        };
        if action == Action::Dimension {
            self.apply_dimension(cx)
        } else {
            self.constrain_selection(action, cx)
        }
    }
    fn constrain_selection(&mut self, action: Action, cx: &mut Context<Self>) {
        let mut d = self.display_design();
        let selected = &self.selection;
        let lines: Vec<_> = selected
            .iter()
            .copied()
            .filter(|id| d.lines.iter().any(|l| l.id == *id))
            .collect();
        let points: Vec<_> = selected
            .iter()
            .copied()
            .filter(|id| d.points.iter().any(|p| p.id == *id))
            .collect();
        let curves: Vec<_> = selected
            .iter()
            .copied()
            .filter(|id| !crate::sketch::entities::curve_points(&d, *id).is_empty())
            .collect();
        let circles: Vec<_> = selected
            .iter()
            .copied()
            .filter(|id| d.circles.iter().any(|c| c.id == *id))
            .collect();
        if action == Action::Fixed && !selected.is_empty() {
            self.pending_constraint = None;
            crate::sketch::edit::fixed(&mut d, selected);
            self.commit_sketch(d, cx);
            return;
        }
        if (action == Action::Horizontal || action == Action::Vertical) && !lines.is_empty() {
            for line in lines {
                d.constrain(if action == Action::Horizontal {
                    ConstraintKind::Horizontal { line }
                } else {
                    ConstraintKind::Vertical { line }
                });
            }
            self.pending_constraint = None;
            self.commit_sketch(d, cx);
            return;
        }
        let kind = match action {
            Action::Horizontal if points.len() == 2 => Some(ConstraintKind::HorizontalPoints {
                points: [points[0], points[1]],
            }),
            Action::Vertical if points.len() == 2 => Some(ConstraintKind::VerticalPoints {
                points: [points[0], points[1]],
            }),
            Action::Horizontal if !lines.is_empty() => {
                Some(ConstraintKind::Horizontal { line: lines[0] })
            }
            Action::Vertical if !lines.is_empty() => {
                Some(ConstraintKind::Vertical { line: lines[0] })
            }
            Action::Parallel if lines.len() == 2 => Some(ConstraintKind::Parallel {
                lines: [lines[0], lines[1]],
            }),
            Action::Perpendicular if lines.len() == 2 => Some(ConstraintKind::Perpendicular {
                lines: [lines[0], lines[1]],
            }),
            Action::Collinear if lines.len() == 2 => Some(ConstraintKind::Collinear {
                lines: [lines[0], lines[1]],
            }),
            Action::Equal if curves.len() == 2 => Some(ConstraintKind::Equal {
                curves: [curves[0], curves[1]],
            }),
            Action::Tangent if curves.len() == 2 => Some(ConstraintKind::Tangent {
                curves: [curves[0], curves[1]],
            }),
            Action::Concentric if circles.len() == 2 => Some(ConstraintKind::Concentric {
                circles: [circles[0], circles[1]],
            }),
            Action::Midpoint if points.len() == 1 && lines.len() == 1 => {
                Some(ConstraintKind::Midpoint {
                    point: points[0],
                    line: lines[0],
                })
            }
            Action::Symmetry if points.len() == 2 && lines.len() == 1 => {
                Some(ConstraintKind::Symmetry {
                    points: [points[0], points[1]],
                    axis: lines[0],
                })
            }
            Action::Coincident if points.len() == 2 => Some(ConstraintKind::Coincident {
                points: [points[0], points[1]],
            }),
            Action::Coincident if points.len() == 1 && lines.len() == 1 => {
                Some(ConstraintKind::PointOnLine {
                    point: points[0],
                    line: lines[0],
                })
            }
            Action::Coincident if points.len() == 1 && circles.len() == 1 => {
                Some(ConstraintKind::PointOnCircle {
                    point: points[0],
                    circle: circles[0],
                })
            }
            _ => None,
        };
        if let Some(kind) = kind {
            self.pending_constraint = None;
            self.error = None;
            d.constrain(kind);
            self.commit_sketch(d, cx);
        } else {
            self.pending_constraint = Some(action);
            self.tool = Tool::Select;
            self.status = "Select the required points or curves in order".into();
            self.error = None;
            cx.notify()
        }
    }
    fn apply_dimension(&mut self, cx: &mut Context<Self>) {
        let mut d = self.display_design();
        let raw = self.dimension.read(cx).content.to_string();
        let angular = self
            .dimension_edit
            .and_then(|id| {
                d.constraints
                    .iter()
                    .chain(&d.driven_dimensions)
                    .find(|c| c.id == id)
            })
            .is_some_and(|c| matches!(c.kind, ConstraintKind::Angle { .. }))
            || crate::sketch::dimensions::kind(&d, &self.selection, Uuid::nil())
                .is_ok_and(|c| matches!(c, ConstraintKind::Angle { .. }));
        let expression = crate::parameters::expression::dimension_input(&raw, angular);
        if let Some(id) = self.dimension_edit {
            let existing = d
                .constraints
                .iter()
                .chain(&d.driven_dimensions)
                .find(|c| c.id == id)
                .cloned();
            if let Some(c) = existing {
                let was_driven = d.driven_dimensions.iter().any(|c| c.id == id);
                if self.reference_dimension && !was_driven {
                    d.constraints.retain(|c| c.id != id);
                    d.driven_dimensions.push(c);
                } else if !self.reference_dimension {
                    if let Some(parameter) = c.kind.parameter() {
                        if let Some(p) = d.parameters.iter_mut().find(|p| p.id == parameter) {
                            p.expression = expression;
                        } else {
                            d.parameters.push(crate::document::schema::Parameter {
                                scalar: false,
                                id: parameter,
                                name: format!("d{}", d.parameters.len() + 1),
                                expression,
                                angular: matches!(c.kind, ConstraintKind::Angle { .. }),
                            });
                        }
                    }
                    if was_driven {
                        d.driven_dimensions.retain(|c| c.id != id);
                        d.constraints.push(c);
                    }
                }
                self.commit_sketch(d, cx);
            }
            return;
        }
        let parameter = Uuid::new_v4();
        match crate::sketch::dimensions::kind_at(
            &d,
            &self.selection,
            parameter,
            self.dimension_position,
        ) {
            Ok(kind) => {
                let id = Uuid::new_v4();
                if self.reference_dimension {
                    d.driven_dimensions
                        .push(crate::document::schema::Constraint { id, kind });
                } else {
                    let name = format!("d{}", d.parameters.len() + 1);
                    d.parameters.push(crate::document::schema::Parameter {
                        scalar: false,
                        id: parameter,
                        name,
                        expression,
                        angular: matches!(kind, ConstraintKind::Angle { .. }),
                    });
                    d.constraints
                        .push(crate::document::schema::Constraint { id, kind });
                }
                let at = self
                    .dimension_position
                    .unwrap_or_else(|| crate::sketch::entities::position(&d, self.selection[0]));
                d.dimension_positions.insert(id, at);
                if self.commit_sketch(d, cx) {
                    self.dimension_edit = None;
                    self.dimension_position = None;
                    self.selection.clear();
                    self.tool = Tool::Dimension;
                }
            }
            Err(e) => {
                self.error = Some(e);
                cx.notify();
            }
        }
    }
    fn apply_transform(&mut self, cx: &mut Context<Self>) {
        let mut parameters = Design::default();
        let x = parameters.parameter(
            "x",
            if matches!(
                self.transform_action,
                Action::Move | Action::RectangularPattern
            ) {
                self.transform_x.read(cx).content.to_string()
            } else {
                "0 mm".into()
            },
        );
        let y = parameters.parameter(
            "y",
            if matches!(
                self.transform_action,
                Action::Move | Action::RectangularPattern
            ) {
                self.transform_y.read(cx).content.to_string()
            } else {
                "0 mm".into()
            },
        );
        let angle =
            parameters.parameter("angle", self.transform_angle.read(cx).content.to_string());
        parameters.parameters.last_mut().unwrap().angular = true;
        let values = match crate::parameters::expression::evaluate(&parameters) {
            Ok(p) => p,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };
        let x = values[&x];
        let y = values[&y];
        let angle = values[&angle];
        let count = self
            .transform_count
            .read(cx)
            .content
            .parse::<f64>()
            .unwrap_or(f64::NAN);
        let mut d = self.display_design();
        let mut selected = self.selection.clone();
        let mut axis = None;
        if self.transform_action == Action::Mirror {
            if let Some(id) = selected.last().copied()
                && let Some(line) = d.lines.iter().find(|l| l.id == id)
            {
                axis = Some(line.ends.map(|id| crate::sketch::entities::point(&d, id)));
                selected.pop();
            } else {
                self.error = Some("Select geometry, then Ctrl-click the mirror line".into());
                return;
            }
        }
        let origin = if matches!(
            self.transform_action,
            Action::Scale | Action::CircularPattern
        ) {
            selected
                .last()
                .and_then(|id| d.points.iter().find(|p| p.id == *id))
                .map(|p| p.xy)
                .unwrap_or([0., 0.])
        } else {
            [0., 0.]
        };
        if matches!(
            self.transform_action,
            Action::Scale | Action::CircularPattern
        ) && selected
            .last()
            .is_some_and(|id| d.points.iter().any(|p| p.id == *id))
        {
            selected.pop();
        }
        let repetitions = if matches!(
            self.transform_action,
            Action::RectangularPattern | Action::CircularPattern
        ) {
            if count.fract() != 0. || !(2. ..=32.).contains(&count) {
                self.error = Some("Pattern count must be between 2 and 32".into());
                return;
            }
            count as usize
        } else {
            1
        };
        if self.transform_action == Action::Scale && (!count.is_finite() || count <= 0.) {
            self.error = Some("Scale factor must be positive".into());
            return;
        }
        let rows = if self.transform_action == Action::RectangularPattern {
            match self.transform_rows.read(cx).content.parse::<usize>() {
                Ok(n) if (1..=32).contains(&n) => n,
                _ => {
                    self.error = Some("Y quantity must be between 1 and 32".into());
                    return;
                }
            }
        } else {
            1
        };
        let mut result = Ok(vec![]);
        for index in 0..repetitions * rows {
            if repetitions > 1 && index == 0 {
                continue;
            }
            let factor = if repetitions > 1 { index as f64 } else { 1. };
            let rotation = if self.transform_action == Action::CircularPattern {
                let sweep = if angle.abs() < 1e-10 {
                    std::f64::consts::TAU
                } else {
                    angle
                };
                sweep * factor
                    / if (sweep.abs() - std::f64::consts::TAU).abs() < 1e-8 {
                        repetitions as f64
                    } else {
                        (repetitions - 1) as f64
                    }
            } else if self.transform_action == Action::RectangularPattern {
                0.
            } else {
                angle
            };
            let action = self.transform_action;
            let column = if action == Action::RectangularPattern {
                (index % repetitions) as f64
            } else {
                factor
            };
            let row = if action == Action::RectangularPattern {
                (index / repetitions) as f64
            } else {
                factor
            };
            let map = |p: [f64; 2]| {
                if let Some([a, b]) = axis {
                    let v = [b[0] - a[0], b[1] - a[1]];
                    let t = ((p[0] - a[0]) * v[0] + (p[1] - a[1]) * v[1])
                        / (v[0] * v[0] + v[1] * v[1]).max(1e-24);
                    return [2. * (a[0] + t * v[0]) - p[0], 2. * (a[1] + t * v[1]) - p[1]];
                }
                if action == Action::Scale {
                    return [
                        origin[0] + (p[0] - origin[0]) * count,
                        origin[1] + (p[1] - origin[1]) * count,
                    ];
                }
                let a = [p[0] - origin[0], p[1] - origin[1]];
                [
                    origin[0] + a[0] * rotation.cos() - a[1] * rotation.sin()
                        + if action == Action::CircularPattern {
                            0.
                        } else {
                            x * column
                        },
                    origin[1]
                        + a[0] * rotation.sin()
                        + a[1] * rotation.cos()
                        + if action == Action::CircularPattern {
                            0.
                        } else {
                            y * row
                        },
                ]
            };
            result = crate::sketch::edit::transform(
                &mut d,
                &selected,
                map,
                self.transform_copy || repetitions > 1 || action == Action::Mirror,
            );
            if result.is_err() {
                break;
            }
        }
        match result {
            Ok(_) => {
                self.commit_sketch(d, cx);
            }
            Err(e) => {
                self.error = Some(e);
                cx.notify();
            }
        }
    }
    fn dimension_hit(
        &self,
        d: &Design,
        c: &crate::document::schema::Constraint,
        at: [f64; 2],
    ) -> bool {
        let ends = match c.kind {
            ConstraintKind::Length { line, .. } => {
                d.lines.iter().find(|l| l.id == line).map(|l| l.ends)
            }
            ConstraintKind::Distance { points, .. }
            | ConstraintKind::DistanceX { points, .. }
            | ConstraintKind::DistanceY { points, .. }
            | ConstraintKind::ProjectedDistance { points, .. } => Some(points),
            _ => None,
        };
        let Some(ends) = ends else {
            return false;
        };
        let a = crate::sketch::entities::position(d, ends[0]);
        let b = crate::sketch::entities::position(d, ends[1]);
        let anchor = self.dimension_anchor(d, c);
        let direction = match c.kind {
            ConstraintKind::DistanceX { .. } => [1., 0.],
            ConstraintKind::DistanceY { .. } => [0., 1.],
            ConstraintKind::ProjectedDistance { direction, .. } => direction,
            _ => [b[0] - a[0], b[1] - a[1]],
        };
        let length = direction[0].hypot(direction[1]).max(1e-12);
        let normal = [-direction[1] / length, direction[0] / length];
        let project = |p: [f64; 2]| {
            let offset = (anchor[0] - p[0]) * normal[0] + (anchor[1] - p[1]) * normal[1];
            [p[0] + offset * normal[0], p[1] + offset * normal[1]]
        };
        let Some(a) = self.sketch_screen(project(a)) else {
            return false;
        };
        let Some(b) = self.sketch_screen(project(b)) else {
            return false;
        };
        let Some(p) = self.sketch_screen(at) else {
            return false;
        };
        let delta = [b[0] - a[0], b[1] - a[1]];
        let fraction = (((p[0] - a[0]) * delta[0] + (p[1] - a[1]) * delta[1])
            / (delta[0] * delta[0] + delta[1] * delta[1]).max(1e-12))
        .clamp(0., 1.);
        (p[0] - a[0] - fraction * delta[0]).hypot(p[1] - a[1] - fraction * delta[1]) < 7.
    }
    fn dimension_anchor(&self, d: &Design, c: &crate::document::schema::Constraint) -> [f64; 2] {
        d.dimension_positions
            .get(&c.id)
            .copied()
            .unwrap_or_else(|| {
                let first = c.kind.references().first().copied().unwrap_or(Uuid::nil());
                let p = crate::sketch::entities::position(d, first);
                if let ConstraintKind::Diameter { circle, .. }
                | ConstraintKind::Radius { circle, .. } = c.kind
                    && let Some(circle) = d.circles.iter().find(|v| v.id == circle)
                {
                    let center = crate::sketch::entities::position(d, circle.center);
                    let rim = crate::sketch::entities::position(d, circle.rim);
                    let radius = (rim[0] - center[0]).hypot(rim[1] - center[1]);
                    return [center[0] + radius * 1.25, center[1] + radius * 0.55];
                }
                [p[0] + 0.004, p[1] + 0.004]
            })
    }
    fn poll_drag_solve(&mut self, cx: &mut Context<Self>) {
        if let Some((mut candidate, result)) =
            self.drag_solve.as_ref().and_then(|r| r.try_recv().ok())
        {
            self.drag_solve = None;
            if let Some(drag) = &mut self.sketch_drag {
                if let Ok(solution) = result
                    && solution.conflicts.is_empty()
                {
                    let changed = drag
                        .before
                        .points
                        .iter()
                        .zip(&solution.points)
                        .any(|(p, xy)| (p.xy[0] - xy[0]).hypot(p.xy[1] - xy[1]) > 1e-10);
                    for (p, xy) in candidate.points.iter_mut().zip(&solution.points) {
                        p.xy = *xy;
                    }
                    self.design = candidate;
                    self.solved = solution.points;
                    if changed {
                        if !drag.moved {
                            self.revision = self.revision.wrapping_add(1);
                        }
                        drag.moved = true;
                        self.dirty_cache.set((u64::MAX, false));
                    }
                }
            }
        }
        if self.drag_solve.is_none()
            && let Some(at) = self.drag_target.take()
            && let Some(drag) = &self.sketch_drag
        {
            let mut candidate = drag.before.clone();
            let delta = [at[0] - drag.start[0], at[1] - drag.start[1]];
            let targets: Vec<_> = candidate
                .points
                .iter_mut()
                .filter(|p| drag.points.contains(&p.id))
                .map(|p| {
                    p.xy[0] += delta[0];
                    p.xy[1] += delta[1];
                    (p.id, p.xy)
                })
                .collect();
            let (send, receive) = std::sync::mpsc::channel();
            self.drag_solve = Some(receive);
            std::thread::spawn(move || {
                let result =
                    crate::parameters::expression::evaluate(&candidate).and_then(|parameters| {
                        crate::solver::nonlinear::solve_drag_positions(
                            &candidate,
                            &parameters,
                            &targets,
                        )
                    });
                let _ = send.send((candidate, result));
            });
        }
        if self.drag_released && self.drag_solve.is_none() && self.drag_target.is_none() {
            self.drag_released = false;
            self.finish_sketch_drag(false, cx);
        }
    }
    fn finish_sketch_drag(&mut self, cancel: bool, cx: &mut Context<Self>) {
        if cancel {
            self.drag_target = None;
            self.drag_solve = None;
            self.drag_released = false;
            self.pending_constraint = None;
        } else if self.drag_solve.is_some() || self.drag_target.is_some() {
            self.drag_released = true;
            return;
        }
        if let Some((id, before, moved)) = self.dimension_drag.take() {
            self.dirty_cache.set((u64::MAX, false));
            if cancel {
                self.design = before;
            } else if moved {
                self.undo.push(before);
                if self.undo.len() > 100 {
                    self.undo.remove(0);
                }
                self.redo.clear();
            } else {
                self.dimension_edit = Some(id);
                self.dimension_position = self.design.dimension_positions.get(&id).copied();
                let d = self.display_design();
                if let Some(c) = d
                    .constraints
                    .iter()
                    .chain(&d.driven_dimensions)
                    .find(|c| c.id == id)
                {
                    self.reference_dimension = d.driven_dimensions.iter().any(|c| c.id == id);
                    let text = c
                        .kind
                        .parameter()
                        .and_then(|id| d.parameters.iter().find(|p| p.id == id))
                        .map(|p| p.expression.clone())
                        .unwrap_or_else(|| {
                            let value = crate::sketch::dimensions::value(&d, &c.kind).unwrap_or(0.);
                            if matches!(c.kind, ConstraintKind::Angle { .. }) {
                                format!("{} deg", value.to_degrees())
                            } else {
                                format!("{} mm", value * 1000.)
                            }
                        });
                    self.dimension = cx.new(|cx| TextInput::new(&text, cx));
                    self.panel = Some(Panel::Dimension);
                }
            }
            cx.notify();
        }
        if let Some((a, b, _)) = self.marquee.take()
            && !cancel
        {
            let d = self.display_design();
            let min = [a[0].min(b[0]), a[1].min(b[1])];
            let max = [a[0].max(b[0]), a[1].max(b[1])];
            let inside =
                |p: [f64; 2]| p[0] >= min[0] && p[0] <= max[0] && p[1] >= min[1] && p[1] <= max[1];
            if (a[0] - b[0]).hypot(a[1] - b[1]) * self.scale > 3. {
                for id in crate::sketch::entities::curve_ids(&d) {
                    let ps = crate::sketch::entities::samples(&d, id);
                    let selected = if b[0] >= a[0] {
                        ps.iter().all(|p| inside(*p))
                    } else {
                        ps.windows(2).any(|p| {
                            crate::sketch::entities::segment_intersects_box(p[0], p[1], min, max)
                        })
                    };
                    if selected && !self.selection.contains(&id) {
                        self.selection.push(id)
                    }
                }
                for p in &d.points {
                    if inside(p.xy)
                        && crate::sketch::entities::curve_ids(&d).iter().all(|id| {
                            !crate::sketch::entities::curve_points(&d, *id).contains(&p.id)
                        })
                        && !self.selection.contains(&p.id)
                    {
                        self.selection.push(p.id)
                    }
                }
            }
            cx.notify();
        }
        if let Some(drag) = self.sketch_drag.take() {
            if cancel {
                self.design = drag.before;
                self.rebuild(cx)
            } else if drag.moved {
                self.undo.push(drag.before);
                self.redo.clear();
                self.rebuild(cx)
            }
        }
    }
}
