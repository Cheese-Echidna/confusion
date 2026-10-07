//! Interactive planar sketch and parametric solid workspace with shared-device rendering.
//!
//! Exports WorkspaceView behind desktop. Connections: application/bootstrap creates
//! the view, render/gpui_bridge owns compositor integration, camera handles navigation.
//! UI coordinates are normalized against actual layout bounds before GPU picking.

#[cfg(feature = "desktop")]
mod implementation {
    use crate::render::{
        camera::{Camera, ProjectionMode},
        gpui_bridge::GpuViewport,
        scene::DemoVertex,
    };
    use crate::ui::{
        components::{button, icon, separator},
        sketch_canvas::SketchCanvas,
        theme as t,
        toolbar::{self, Action, Feature, Mode},
        view_cube,
    };
    use crate::{
        document::schema::{ConstraintKind, ConstructionKind, Design, Extrusion},
        runtime::worker::Worker,
        ui::text_input::TextInput,
    };
    use gpui::{prelude::*, *};
    use std::collections::HashSet;
    use std::{cell::Cell, path::PathBuf, rc::Rc};
    use uuid::Uuid;
    #[derive(Clone, Copy, PartialEq)]
    enum Panel {
        Document,
        Parameters,
        Extrude,
        Dimension,
        View,
    }

    #[derive(Clone, Copy, PartialEq)]
    enum Tool {
        Select,
        Line,
        Rectangle,
    }

    struct Drag {
        button: MouseButton,
        previous: Point<Pixels>,
    }

    pub struct WorkspaceView {
        gpu: GpuViewport,
        camera: Camera,
        view_revision: u64,
        selected: u32,
        pending_pick: Option<([f64; 2], u64)>,
        bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
        drag: Option<Drag>,
        focus: FocusHandle,
        error: Option<String>,
        design: Design,
        worker: Worker,
        revision: u64,
        solved: Vec<[f64; 2]>,
        sketch: bool,
        tool: Tool,
        anchor: Option<[f64; 2]>,
        hover: Option<[f64; 2]>,
        line: Option<Uuid>,
        status: String,
        inputs: Vec<(Uuid, Entity<TextInput>)>,
        depth: Entity<TextInput>,
        dimension: Entity<TextInput>,
        path: Entity<TextInput>,
        saved_path: Option<PathBuf>,
        undo: Vec<Design>,
        redo: Vec<Design>,
        scale: f64,
        center: [f64; 2],
        volume: Option<f64>,
        conflicts: Vec<Uuid>,
        mode: Mode,
        panel: Option<Panel>,
        menu: Option<&'static str>,
        file_open: bool,
        expanded: HashSet<&'static str>,
        hidden: HashSet<&'static str>,
        grid: bool,
        snap: bool,
        mesh: Option<(Vec<DemoVertex>, Vec<u32>)>,
        construction_cursor: Option<Uuid>,
        cube_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
        menu_x: f32,
    }

    impl WorkspaceView {
        pub fn new(surface: WgpuSurfaceHandle, cx: &mut Context<Self>) -> Self {
            let mut gpu = GpuViewport::new(surface);
            gpu.set_mesh(&[], &[]);
            Self {
                gpu,
                camera: Camera::default(),
                view_revision: 0,
                selected: 0,
                pending_pick: None,
                bounds: Rc::new(Cell::new(None)),
                drag: None,
                focus: cx.focus_handle(),
                error: None,
                design: Design::default(),
                worker: Worker::new(),
                revision: 0,
                solved: vec![],
                sketch: false,
                tool: Tool::Select,
                anchor: None,
                hover: None,
                line: None,
                status: "Create a sketch on the XY plane".into(),
                inputs: vec![],
                depth: cx.new(|cx| TextInput::new("10 mm", cx)),
                dimension: cx.new(|cx| TextInput::new("40 mm", cx)),
                path: cx.new(|cx| TextInput::new("Untitled.con", cx)),
                saved_path: None,
                undo: vec![],
                redo: vec![],
                scale: 6000.,
                center: [0.04, 0.025],
                volume: None,
                conflicts: vec![],
                mode: Mode::Solid,
                panel: None,
                menu: None,
                file_open: false,
                expanded: HashSet::from(["origin", "sketches", "bodies"]),
                hidden: HashSet::new(),
                grid: true,
                snap: true,
                mesh: None,
                construction_cursor: None,
                cube_bounds: Rc::new(Cell::new(None)),
                menu_x: 12.,
            }
        }

        pub fn focus(&self, window: &mut Window) {
            window.focus(&self.focus);
        }

        fn changed_camera(&mut self) {
            self.view_revision = self.view_revision.wrapping_add(1);
            self.pending_pick = None;
        }

        fn fit(&mut self) {
            if self.sketch {
                if !self.solved.is_empty() {
                    let min = [
                        self.solved
                            .iter()
                            .map(|p| p[0])
                            .fold(f64::INFINITY, f64::min),
                        self.solved
                            .iter()
                            .map(|p| p[1])
                            .fold(f64::INFINITY, f64::min),
                    ];
                    let max = [
                        self.solved
                            .iter()
                            .map(|p| p[0])
                            .fold(f64::NEG_INFINITY, f64::max),
                        self.solved
                            .iter()
                            .map(|p| p[1])
                            .fold(f64::NEG_INFINITY, f64::max),
                    ];
                    self.center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
                    if let Some(bounds) = self.bounds.get() {
                        self.scale = (f64::from(bounds.size.width) / (max[0] - min[0] + 0.02))
                            .min(f64::from(bounds.size.height) / (max[1] - min[1] + 0.02))
                            * 0.8;
                    }
                }
            } else {
                let coords = if self.solved.is_empty() {
                    self.design.points.iter().map(|p| p.xy).collect()
                } else {
                    self.solved.clone()
                };
                if !coords.is_empty() {
                    let min = [
                        coords.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min),
                        coords.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min),
                    ];
                    let max = [
                        coords
                            .iter()
                            .map(|p| p[0])
                            .fold(f64::NEG_INFINITY, f64::max),
                        coords
                            .iter()
                            .map(|p| p[1])
                            .fold(f64::NEG_INFINITY, f64::max),
                    ];
                    self.camera.target = nalgebra::Point3::new(
                        (min[0] + max[0]) * 12.5,
                        (min[1] + max[1]) * 12.5,
                        0.,
                    );
                    self.camera.distance = ((max[0] - min[0]).hypot(max[1] - min[1]) * 70.).max(3.);
                }
            }
            self.changed_camera();
        }
        fn checkpoint(&mut self) {
            self.undo.push(self.design.clone());
            if self.undo.len() > 100 {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
        fn rebuild(&mut self, cx: &mut Context<Self>) {
            self.revision = self.revision.wrapping_add(1);
            self.changed_camera();
            self.selected = 0;
            self.volume = None;
            self.conflicts.clear();
            self.gpu.set_mesh(&[], &[]);
            self.status = "Solving…".into();
            self.error = None;
            self.design.sync_construction();
            self.mesh = None;
            let snapshot = self
                .construction_cursor
                .and_then(|id| self.design.through_feature(id).ok())
                .unwrap_or_else(|| self.design.clone());
            self.worker.submit(self.revision, snapshot);
            for p in &self.design.parameters {
                if !self.inputs.iter().any(|(id, _)| *id == p.id) {
                    self.inputs
                        .push((p.id, cx.new(|cx| TextInput::new(&p.expression, cx))));
                }
            }
            self.inputs
                .retain(|(id, _)| self.design.parameters.iter().any(|p| p.id == *id));
            cx.notify();
        }
        fn screen_to_sketch(&self, position: Point<Pixels>) -> Option<[f64; 2]> {
            let b = self.bounds.get()?;
            let x = f64::from(position.x - b.origin.x - b.size.width * 0.5) / self.scale
                + self.center[0];
            let y = -f64::from(position.y - b.origin.y - b.size.height * 0.5) / self.scale
                + self.center[1];
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
        fn sketch_click(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
            let Some(point) = self.screen_to_sketch(position) else {
                return;
            };
            if self.tool == Tool::Select {
                let coords = if self.solved.len() == self.design.points.len() {
                    self.solved.clone()
                } else {
                    self.design.points.iter().map(|p| p.xy).collect()
                };
                self.line = self
                    .design
                    .lines
                    .iter()
                    .find(|l| {
                        let [a, b] = l.ends.map(|id| {
                            coords[self.design.points.iter().position(|p| p.id == id).unwrap()]
                        });
                        let dx = b[0] - a[0];
                        let dy = b[1] - a[1];
                        let t = (((point[0] - a[0]) * dx + (point[1] - a[1]) * dy)
                            / (dx * dx + dy * dy))
                            .clamp(0., 1.);
                        (point[0] - a[0] - t * dx).hypot(point[1] - a[1] - t * dy) * self.scale < 8.
                    })
                    .map(|l| l.id);
                cx.notify();
                return;
            }
            if let Some(a) = self.anchor {
                if (point[0] - a[0]).hypot(point[1] - a[1]) < 1e-6 {
                    return;
                }
                self.checkpoint();
                if self.solved.len() == self.design.points.len() {
                    for (p, xy) in self.design.points.iter_mut().zip(&self.solved) {
                        p.xy = *xy;
                    }
                }
                if self.tool == Tool::Rectangle {
                    if (point[0] - a[0]).abs() < 1e-6 || (point[1] - a[1]).abs() < 1e-6 {
                        return;
                    }
                    self.design.rectangle(a, point);
                    self.anchor = None;
                    self.tool = Tool::Select;
                } else {
                    let id = self.design.line(a, point);
                    if (point[1] - a[1]).abs() < 1e-6 {
                        self.design
                            .constrain(ConstraintKind::Horizontal { line: id });
                    }
                    if (point[0] - a[0]).abs() < 1e-6 {
                        self.design.constrain(ConstraintKind::Vertical { line: id });
                    }
                    self.anchor = Some(point);
                }
                self.rebuild(cx);
            } else {
                self.anchor = Some(point);
                cx.notify();
            }
        }
        fn apply_constraint(&mut self, kind: u8, cx: &mut Context<Self>) {
            let Some(id) = self.line else {
                self.error = Some("Select a sketch line first".into());
                return;
            };
            self.checkpoint();
            let constraint = match kind {
                0 => ConstraintKind::Horizontal { line: id },
                1 => ConstraintKind::Vertical { line: id },
                2 => {
                    let point = self.design.lines.iter().find(|l| l.id == id).unwrap().ends[0];
                    let i = self
                        .design
                        .points
                        .iter()
                        .position(|p| p.id == point)
                        .unwrap();
                    let xy = self
                        .solved
                        .get(i)
                        .copied()
                        .unwrap_or(self.design.points[i].xy);
                    ConstraintKind::Fixed { point, xy }
                }
                _ => {
                    let expression = self.dimension.read(cx).content.to_string();
                    let n = self.design.parameters.len();
                    let parameter = self.design.parameter(&format!("length{n}"), expression);
                    ConstraintKind::Length {
                        line: id,
                        parameter,
                    }
                }
            };
            self.design.constrain(constraint);
            self.rebuild(cx);
        }
        fn extrude(&mut self, cx: &mut Context<Self>) {
            self.checkpoint();
            let expression = self.depth.read(cx).content.to_string();
            let depth = self.design.parameter("depth", expression);
            self.inputs.retain(|(id, _)| *id != depth);
            if let Some(e) = &mut self.design.extrusion {
                e.depth = depth;
            } else {
                self.design.extrusion = Some(Extrusion {
                    id: Uuid::new_v4(),
                    depth,
                });
            }
            self.sketch = false;
            self.mode = Mode::Solid;
            self.construction_cursor = None;
            self.anchor = None;
            self.tool = Tool::Select;
            self.rebuild(cx);
            self.fit();
        }
        fn new_document(&mut self, cx: &mut Context<Self>) {
            self.checkpoint();
            self.design = Design::default();
            self.inputs.clear();
            self.solved.clear();
            self.line = None;
            self.anchor = None;
            self.sketch = false;
            self.mode = Mode::Solid;
            self.construction_cursor = None;
            self.panel = None;
            self.menu = None;
            self.tool = Tool::Select;
            self.saved_path = None;
            self.path = cx.new(|cx| TextInput::new("Untitled.con", cx));
            self.depth = cx.new(|cx| TextInput::new("10 mm", cx));
            self.rebuild(cx);
        }
        fn save(&mut self, cx: &mut Context<Self>) {
            let mut path = PathBuf::from(self.path.read(cx).content.to_string());
            if path.extension().and_then(|x| x.to_str()) != Some("con") {
                path.set_extension("con");
            }
            if path.exists() && self.saved_path.as_ref() != Some(&path) {
                self.error =
                    Some("That file exists. Open it first or choose a new filename.".into());
                return;
            }
            match crate::persistence::container::save(&path, &self.design) {
                Ok(()) => {
                    self.saved_path = Some(path.clone());
                    self.status = format!("Saved {}", path.display());
                    self.error = None;
                }
                Err(e) => self.error = Some(e),
            }
            cx.notify();
        }
        fn open(&mut self, cx: &mut Context<Self>) {
            let mut path = PathBuf::from(self.path.read(cx).content.to_string());
            if path.extension().and_then(|x| x.to_str()) != Some("con") {
                path.set_extension("con");
            }
            match crate::persistence::container::load(&path) {
                Ok(d) => {
                    self.checkpoint();
                    self.design = d;
                    self.saved_path = Some(path);
                    self.inputs.clear();
                    self.solved.clear();
                    self.line = None;
                    self.anchor = None;
                    self.sketch = false;
                    self.mode = Mode::Solid;
                    self.construction_cursor = None;
                    self.panel = None;
                    if let Some(e) = &self.design.extrusion
                        && let Some(p) = self.design.parameters.iter().find(|p| p.id == e.depth)
                    {
                        self.depth = cx.new(|cx| TextInput::new(&p.expression, cx));
                    }
                    self.rebuild(cx);
                }
                Err(e) => self.error = Some(e),
            }
            cx.notify();
        }
        fn pick(&mut self, position: Point<Pixels>) {
            if let Some(bounds) = self.bounds.get()
                && bounds.size.width > px(0.0)
                && bounds.size.height > px(0.0)
            {
                let normalized = [
                    ((position.x - bounds.origin.x) / bounds.size.width) as f64,
                    ((position.y - bounds.origin.y) / bounds.size.height) as f64,
                ];
                self.view_revision = self.view_revision.wrapping_add(1);
                self.pending_pick = Some((normalized, self.view_revision));
            }
        }

        fn refresh_mesh(&mut self) {
            if self.mode == Mode::Solid && !self.hidden.contains("bodies") {
                if let Some((vertices, indices)) = &self.mesh {
                    self.gpu.set_mesh(vertices, indices);
                    return;
                }
            }
            self.gpu.set_mesh(&[], &[]);
        }
        fn set_mode(&mut self, mode: Mode, cx: &mut Context<Self>) {
            self.mode = mode;
            self.sketch = mode == Mode::Sketch;
            self.tool = Tool::Select;
            self.anchor = None;
            self.menu = None;
            self.panel = None;
            self.construction_cursor = None;
            self.changed_camera();
            self.refresh_mesh();
            self.rebuild(cx);
            if mode == Mode::Drawing {
                self.status = "Drawing tools are not implemented".into();
            }
        }
        fn feature(&mut self, feature: Feature, window: &mut Window, cx: &mut Context<Self>) {
            self.menu = None;
            window.focus(&self.focus);
            let Some(action) = feature.action else {
                self.error = Some(format!("{} is not implemented", feature.name));
                cx.notify();
                return;
            };
            match action {
                Action::Sketch => {
                    self.set_mode(Mode::Sketch, cx);
                    if self.design.construction.is_empty() {
                        self.checkpoint();
                        self.design.ensure_sketch();
                        self.rebuild(cx);
                    }
                }
                Action::Line | Action::Rectangle => {
                    if self.mode != Mode::Sketch {
                        self.set_mode(Mode::Sketch, cx);
                    }
                    self.tool = if action == Action::Line {
                        Tool::Line
                    } else {
                        Tool::Rectangle
                    };
                    self.anchor = None;
                }
                Action::Select => {
                    self.tool = Tool::Select;
                    self.anchor = None;
                }
                Action::Parameters => self.panel = Some(Panel::Parameters),
                Action::Extrude => self.panel = Some(Panel::Extrude),
                Action::Dimension => {
                    self.panel = Some(Panel::Dimension);
                    self.tool = Tool::Select;
                    self.anchor = None;
                }
                Action::Horizontal => self.apply_constraint(0, cx),
                Action::Vertical => self.apply_constraint(1, cx),
                Action::Fixed => self.apply_constraint(2, cx),
                Action::Finish => {
                    self.set_mode(Mode::Solid, cx);
                    self.fit();
                }
            }
            cx.notify();
        }
        fn restore_edit(&mut self, cx: &mut Context<Self>) {
            self.inputs.clear();
            self.solved.clear();
            self.line = None;
            self.anchor = None;
            self.construction_cursor = None;
            self.depth = cx.new(|cx| {
                TextInput::new(
                    self.design
                        .extrusion
                        .as_ref()
                        .and_then(|e| self.design.parameters.iter().find(|p| p.id == e.depth))
                        .map_or("10 mm", |p| p.expression.as_str()),
                    cx,
                )
            });
            self.rebuild(cx);
        }
        fn undo_edit(&mut self, cx: &mut Context<Self>) {
            if let Some(d) = self.undo.pop() {
                self.redo.push(self.design.clone());
                self.design = d;
                self.restore_edit(cx);
            }
        }
        fn redo_edit(&mut self, cx: &mut Context<Self>) {
            if let Some(d) = self.redo.pop() {
                self.undo.push(self.design.clone());
                self.design = d;
                self.restore_edit(cx);
            }
        }
        fn construction_step(&mut self, id: Option<Uuid>, edit: bool, cx: &mut Context<Self>) {
            self.construction_cursor = id;
            self.menu = None;
            self.tool = Tool::Select;
            self.anchor = None;
            self.line = None;
            let kind = id
                .and_then(|id| self.design.construction.iter().find(|f| f.id == id))
                .map(|f| f.kind.clone());
            self.mode = if matches!(kind, Some(ConstructionKind::Sketch)) {
                Mode::Sketch
            } else {
                Mode::Solid
            };
            self.sketch = self.mode == Mode::Sketch;
            self.panel = if edit {
                Some(if self.sketch {
                    Panel::Parameters
                } else {
                    Panel::Extrude
                })
            } else {
                None
            };
            self.rebuild(cx);
        }
        fn keyboard(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
            if event.keystroke.modifiers.control || event.keystroke.modifiers.platform {
                match event.keystroke.key.as_str() {
                    "s" => {
                        if self.saved_path.is_some() {
                            self.save(cx);
                        } else {
                            self.file_open = false;
                            self.panel = Some(Panel::Document);
                        }
                    }
                    "o" => {
                        self.file_open = true;
                        self.panel = Some(Panel::Document);
                    }
                    "n" => self.new_document(cx),
                    "z" if self.focus.is_focused(window) => {
                        if event.keystroke.modifiers.shift {
                            self.redo_edit(cx);
                        } else {
                            self.undo_edit(cx);
                        }
                    }
                    "y" if self.focus.is_focused(window) => self.redo_edit(cx),
                    _ => return,
                }
                cx.stop_propagation();
                cx.notify();
                return;
            }
            if !self.focus.is_focused(window) {
                if event.keystroke.key == "escape" {
                    window.focus(&self.focus);
                } else {
                    return;
                }
            }
            match event.keystroke.key.as_str() {
                "escape" => {
                    self.anchor = None;
                    self.tool = Tool::Select;
                    self.line = None;
                    self.selected = 0;
                    self.menu = None;
                    self.panel = None;
                    self.changed_camera();
                }
                "f" => self.fit(),
                "l" | "r" => {
                    self.set_mode(Mode::Sketch, cx);
                    self.tool = if event.keystroke.key == "l" {
                        Tool::Line
                    } else {
                        Tool::Rectangle
                    };
                }
                "d" if self.sketch => self.panel = Some(Panel::Dimension),
                "e" => self.panel = Some(Panel::Extrude),
                _ => return,
            }
            cx.notify();
        }
    }

    impl WorkspaceView {
        fn top_bar(&self, cx: &mut Context<Self>) -> Div {
            let mut row = div()
                .h(px(42.))
                .flex_none()
                .flex()
                .items_center()
                .px_2()
                .gap_1()
                .bg(rgb(t::PANEL))
                .border_b_1()
                .border_color(rgb(t::BORDER));
            row = row
                .child(
                    button("file-open", "open", "Open design · Ctrl+O", false, true).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.file_open = true;
                            this.panel = Some(Panel::Document);
                            this.menu = None;
                            cx.notify();
                        }),
                    ),
                )
                .child(
                    button("file-new", "new", "New design · Ctrl+N", false, true).on_click(
                        cx.listener(|this, _, window, cx| {
                            this.new_document(cx);
                            window.focus(&this.focus);
                        }),
                    ),
                )
                .child(
                    button(
                        "file-export",
                        "export",
                        "Export",
                        self.menu == Some("Export"),
                        false,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.menu_x = 8.;
                        this.menu = Some("Export");
                        cx.notify();
                    })),
                )
                .child(
                    button("file-save", "save", "Save · Ctrl+S", false, true).on_click(
                        cx.listener(|this, _, _, cx| {
                            if this.saved_path.is_some() {
                                this.save(cx);
                            } else {
                                this.file_open = false;
                                this.panel = Some(Panel::Document);
                            }
                            this.menu = None;
                            cx.notify();
                        }),
                    ),
                )
                .child(separator())
                .child(
                    button("undo", "undo", "Undo edit · Ctrl+Z", false, true)
                        .opacity(if self.undo.is_empty() { 0.4 } else { 1. })
                        .on_click(cx.listener(|this, _, _, cx| this.undo_edit(cx))),
                )
                .child(
                    button("redo", "redo", "Redo edit · Ctrl+Y", false, true)
                        .opacity(if self.redo.is_empty() { 0.4 } else { 1. })
                        .on_click(cx.listener(|this, _, _, cx| this.redo_edit(cx))),
                )
                .child(separator());
            for mode in [Mode::Solid, Mode::Sketch, Mode::Drawing] {
                row = row.child(
                    div()
                        .id(SharedString::from(format!("mode-{}", mode.label())))
                        .h_full()
                        .px_4()
                        .flex()
                        .items_center()
                        .gap_2()
                        .cursor_pointer()
                        .bg(rgb(if self.mode == mode {
                            t::VIEWPORT
                        } else {
                            t::PANEL
                        }))
                        .text_color(rgb(if self.mode == mode { t::TEXT } else { t::MUTED }))
                        .hover(|s| s.bg(rgb(t::HOVER)))
                        .child(icon(
                            match mode {
                                Mode::Solid => "create_block",
                                Mode::Sketch => "line_rectangle",
                                Mode::Drawing => "drawing_settings",
                            },
                            16.,
                            if self.mode == mode {
                                t::ACCENT
                            } else {
                                t::MUTED
                            },
                        ))
                        .child(mode.label())
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.set_mode(mode, cx);
                            window.focus(&this.focus);
                        })),
                )
            }
            row.child(div().flex_1()).child(
                div()
                    .text_size(px(11.))
                    .text_color(rgb(t::MUTED))
                    .px_3()
                    .child(self.document_name(cx)),
            )
        }
        fn document_name(&self, cx: &App) -> String {
            let path = self.path.read(cx).content.to_string();
            PathBuf::from(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_owned()
        }
        fn ribbon(&self, cx: &mut Context<Self>) -> Stateful<Div> {
            let mut row = div()
                .id("ribbon")
                .h(px(66.))
                .flex_none()
                .flex()
                .items_center()
                .px_3()
                .gap_3()
                .bg(rgb(t::VIEWPORT))
                .border_b_1()
                .border_color(rgb(t::BORDER))
                .overflow_x_scroll();
            for group in toolbar::groups(self.mode) {
                let name = group.name;
                let mut column = div().flex_none().flex().flex_col().gap_1();
                let mut tools = div().flex().gap_1();
                // Three discoverable primary icons per group; every remaining tool is in its menu.
                for &feature in group.features.iter().take(3) {
                    let active = feature.action.is_some_and(|a| {
                        matches!(
                            (a, self.tool),
                            (Action::Line, Tool::Line) | (Action::Rectangle, Tool::Rectangle)
                        )
                    });
                    tools =
                        tools.child(
                            button(
                                feature.id,
                                feature.icon,
                                feature.name,
                                active,
                                feature.available(),
                            )
                            .on_click(cx.listener(
                                move |this, _, window, cx| this.feature(feature, window, cx),
                            )),
                        );
                }
                column = column.child(tools).child(
                    div()
                        .id(SharedString::from(format!("group-{name}")))
                        .h(px(15.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_1()
                        .cursor_pointer()
                        .rounded_sm()
                        .hover(|s| s.bg(rgb(t::HOVER)))
                        .child(icon("down", 11., t::MUTED))
                        .tooltip(move |_, cx| cx.new(|_| GroupTip(name)).into())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.menu = if this.menu == Some(name) {
                                None
                            } else {
                                Some(name)
                            };
                            cx.notify();
                        })),
                );
                row = row
                    .child(column)
                    .child(div().h(px(36.)).w(px(1.)).bg(rgb(t::BORDER)).flex_none());
            }
            row
        }
        fn browser(&self, cx: &mut Context<Self>) -> Div {
            let mut tree = div()
                .w(px(230.))
                .flex_none()
                .flex()
                .flex_col()
                .bg(rgb(t::PANEL))
                .border_r_1()
                .border_color(rgb(t::BORDER));
            tree = tree.child(
                div()
                    .h(px(37.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .px_3()
                    .gap_2()
                    .border_b_1()
                    .border_color(rgb(t::BORDER))
                    .child(icon("create_block", 16., t::MUTED))
                    .child(self.document_name(cx)),
            );
            let mut rows = div()
                .id("document-tree")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .py_2();
            for (key, label, image) in [
                ("construction", "Construction", "construction_layer"),
                ("sketches", "Sketches", "line_rectangle"),
                ("bodies", "Bodies", "create_block"),
                ("components", "Components", "insert_active_block"),
                ("origin", "Origin", "set_rel_zero"),
                ("analysis", "Analysis", "measure"),
            ] {
                let expanded = self.expanded.contains(key);
                let hidden = self.hidden.contains(key);
                let mut category = div()
                    .h(px(30.))
                    .flex()
                    .items_center()
                    .px_2()
                    .gap_2()
                    .child(
                        button(
                            SharedString::from(format!("expand-{label}")),
                            if expanded {
                                "down"
                            } else {
                                "dockwidgets_right"
                            },
                            &format!("{} {label}", if expanded { "Collapse" } else { "Expand" }),
                            false,
                            true,
                        )
                        .size(px(20.))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if !this.expanded.remove(key) {
                                this.expanded.insert(key);
                            }
                            cx.notify();
                        })),
                    )
                    .child(icon(image, 16., t::MUTED))
                    .child(div().flex_1().child(label))
                    .child(
                        button(
                            SharedString::from(format!("visibility-{label}")),
                            if hidden { "invisible" } else { "visible" },
                            &format!("{} {label}", if hidden { "Show" } else { "Hide" }),
                            false,
                            true,
                        )
                        .size(px(22.))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if !this.hidden.remove(key) {
                                this.hidden.insert(key);
                            }
                            this.changed_camera();
                            this.refresh_mesh();
                            cx.notify();
                        })),
                    );
                if hidden {
                    category = category.text_color(rgb(t::DISABLED));
                }
                rows = rows.child(category);
                if expanded {
                    match key {
                        "sketches" => {
                            for f in self
                                .design
                                .construction
                                .iter()
                                .filter(|f| matches!(f.kind, ConstructionKind::Sketch))
                            {
                                let id = f.id;
                                rows = rows.child(
                                    tree_item(
                                        SharedString::from(format!("sketch-{id}")),
                                        "line_rectangle",
                                        &f.name,
                                        self.mode == Mode::Sketch,
                                    )
                                    .on_click(cx.listener(
                                        move |this, _, window, cx| {
                                            this.construction_step(Some(id), true, cx);
                                            window.focus(&this.focus);
                                        },
                                    )),
                                );
                            }
                        }
                        "bodies" => {
                            if let Some(e) = &self.design.extrusion {
                                let id = e.id;
                                rows = rows.child(
                                    tree_item(
                                        "body-1",
                                        "create_block",
                                        "Body 1",
                                        self.panel == Some(Panel::Extrude),
                                    )
                                    .on_click(cx.listener(
                                        move |this, _, _, cx| {
                                            this.construction_step(Some(id), true, cx)
                                        },
                                    )),
                                );
                            }
                        }
                        "origin" => {
                            for (label, image, direction, up) in [
                                (
                                    "X",
                                    "line_horizontal",
                                    nalgebra::Vector3::x(),
                                    nalgebra::Vector3::z(),
                                ),
                                (
                                    "Y",
                                    "line_vertical",
                                    nalgebra::Vector3::y(),
                                    nalgebra::Vector3::z(),
                                ),
                                (
                                    "Z",
                                    "line_perpendicular",
                                    nalgebra::Vector3::z(),
                                    nalgebra::Vector3::y(),
                                ),
                            ] {
                                rows = rows.child(
                                    tree_item(
                                        SharedString::from(format!("axis-{label}")),
                                        image,
                                        label,
                                        false,
                                    )
                                    .on_click(cx.listener(
                                        move |this, _, _, cx| {
                                            this.camera.set_direction(direction, up);
                                            this.changed_camera();
                                            cx.notify();
                                        },
                                    )),
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            tree.child(rows)
        }
        fn inspector(&self, cx: &mut Context<Self>) -> Option<Div> {
            let panel = self.panel?;
            let mut body = div()
                .w(px(270.))
                .flex_none()
                .flex()
                .flex_col()
                .bg(rgb(t::PANEL))
                .border_l_1()
                .border_color(rgb(t::BORDER));
            let title = match panel {
                Panel::Document => {
                    if self.file_open {
                        "Open design"
                    } else {
                        "Save design"
                    }
                }
                Panel::Parameters => "Parameters",
                Panel::Extrude => "Extrude",
                Panel::Dimension => "Dimension",
                Panel::View => "View",
            };
            body = body.child(
                div()
                    .h(px(37.))
                    .flex_none()
                    .px_3()
                    .flex()
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(rgb(t::BORDER))
                    .child(title)
                    .child(
                        button("close-panel", "close", "Close panel · Escape", false, true)
                            .size(px(24.))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.panel = None;
                                cx.notify();
                            })),
                    ),
            );
            let mut content = div()
                .id("inspector-content")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .p_3()
                .flex()
                .flex_col()
                .gap_3();
            match panel {
                Panel::Document => {
                    content = content.child(self.path.clone()).child(
                        div().flex().justify_end().child(
                            button(
                                "confirm-file",
                                if self.file_open { "open" } else { "save" },
                                title,
                                false,
                                true,
                            )
                            .on_click(cx.listener(
                                |this, _, window, cx| {
                                    if this.file_open {
                                        this.open(cx);
                                    } else {
                                        this.save(cx);
                                        if this.error.is_none() {
                                            this.panel = None;
                                        }
                                    }
                                    window.focus(&this.focus);
                                    cx.notify();
                                },
                            )),
                        ),
                    );
                }
                Panel::Parameters => {
                    for (id, input) in &self.inputs {
                        let id = *id;
                        let input = input.clone();
                        let p = self.design.parameters.iter().find(|p| p.id == id).unwrap();
                        content = content
                            .child(div().text_color(rgb(t::MUTED)).child(p.name.clone()))
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .items_center()
                                    .child(div().flex_1().min_w_0().child(input.clone()))
                                    .child(
                                        button(
                                            SharedString::from(format!("apply-{id}")),
                                            "exclusive",
                                            "Apply parameter",
                                            false,
                                            true,
                                        )
                                        .on_click(
                                            cx.listener(move |this, _, window, cx| {
                                                this.checkpoint();
                                                let expression = input.read(cx).content.to_string();
                                                if let Some(p) = this
                                                    .design
                                                    .parameters
                                                    .iter_mut()
                                                    .find(|p| p.id == id)
                                                {
                                                    p.expression = expression.clone();
                                                }
                                                if this
                                                    .design
                                                    .extrusion
                                                    .as_ref()
                                                    .is_some_and(|e| e.depth == id)
                                                {
                                                    this.depth = cx
                                                        .new(|cx| TextInput::new(&expression, cx));
                                                }
                                                this.rebuild(cx);
                                                window.focus(&this.focus);
                                            }),
                                        ),
                                    ),
                            );
                    }
                    if self.inputs.is_empty() {
                        content =
                            content.child(div().text_color(rgb(t::MUTED)).child("No parameters"));
                    }
                }
                Panel::Extrude => {
                    content = content
                        .child(div().text_color(rgb(t::MUTED)).child("Depth"))
                        .child(self.depth.clone())
                        .child(div().flex().justify_end().child(
                            button("apply-extrude", "up", "Apply extrusion", false, true).on_click(
                                cx.listener(|this, _, window, cx| {
                                    this.extrude(cx);
                                    window.focus(&this.focus);
                                }),
                            ),
                        ))
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(rgb(t::MUTED))
                                .child("New body · XY profile"),
                        );
                }
                Panel::Dimension => {
                    content = content
                        .child(
                            div()
                                .text_color(rgb(t::MUTED))
                                .child(if self.line.is_some() {
                                    "Length"
                                } else {
                                    "Select a sketch line"
                                }),
                        )
                        .child(self.dimension.clone())
                        .child(
                            div().flex().justify_end().child(
                                button(
                                    "apply-dimension",
                                    "dim_linear",
                                    "Apply dimension",
                                    false,
                                    true,
                                )
                                .on_click(cx.listener(
                                    |this, _, window, cx| {
                                        this.apply_constraint(3, cx);
                                        window.focus(&this.focus);
                                    },
                                )),
                            ),
                        );
                }
                Panel::View => {
                    for (id, image, label, enabled) in [
                        ("view-grid", "grid", "Grid", self.grid),
                        ("view-snap", "snap_grid", "Snap to grid", self.snap),
                    ] {
                        content = content.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(button(id, image, label, enabled, true).on_click(
                                    cx.listener(move |this, _, _, cx| {
                                        if id == "view-grid" {
                                            this.grid = !this.grid;
                                        } else {
                                            this.snap = !this.snap;
                                        }
                                        cx.notify();
                                    }),
                                ))
                                .child(label),
                        );
                    }
                    content = content.child(div().text_color(rgb(t::MUTED)).child("Visual style"));
                    for (id, image, label) in [
                        ("style-shaded", "attributes", "Shaded"),
                        ("style-edges", "line", "Shaded with edges"),
                        ("style-wire", "line_rectangle", "Wireframe"),
                        ("style-hidden", "invisible", "Hidden edges"),
                    ] {
                        let available = id == "style-shaded";
                        content = content.child(
                            div()
                                .flex()
                                .gap_2()
                                .items_center()
                                .child(button(id, image, label, available, available).on_click(
                                    cx.listener(move |this, _, _, cx| {
                                        if !available {
                                            this.error =
                                                Some(format!("{label} is not implemented"));
                                        }
                                        cx.notify();
                                    }),
                                ))
                                .child(label),
                        );
                    }
                }
            }
            if panel == Panel::Dimension || (panel == Panel::Parameters && self.line.is_some()) {
                if let Some(line) = self.line {
                    let ends = self
                        .design
                        .lines
                        .iter()
                        .find(|l| l.id == line)
                        .map(|l| l.ends)
                        .unwrap_or_default();
                    for c in &self.design.constraints {
                        let label = match &c.kind {
                            ConstraintKind::Horizontal { line: id } if *id == line => {
                                Some("Horizontal")
                            }
                            ConstraintKind::Vertical { line: id } if *id == line => {
                                Some("Vertical")
                            }
                            ConstraintKind::Length { line: id, .. } if *id == line => {
                                Some("Length")
                            }
                            ConstraintKind::Fixed { point, .. } if ends.contains(point) => {
                                Some("Fixed point")
                            }
                            ConstraintKind::DistanceX { points, .. }
                                if points.iter().any(|p| ends.contains(p)) =>
                            {
                                Some("Width")
                            }
                            ConstraintKind::DistanceY { points, .. }
                                if points.iter().any(|p| ends.contains(p)) =>
                            {
                                Some("Height")
                            }
                            _ => None,
                        };
                        if let Some(label) = label {
                            let id = c.id;
                            content = content.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .text_color(rgb(if self.conflicts.contains(&id) {
                                        t::ERROR
                                    } else {
                                        t::MUTED
                                    }))
                                    .child(label)
                                    .child(
                                        button(
                                            SharedString::from(format!("remove-{id}")),
                                            "remove",
                                            "Remove constraint",
                                            false,
                                            true,
                                        )
                                        .on_click(
                                            cx.listener(move |this, _, window, cx| {
                                                this.checkpoint();
                                                this.design.constraints.retain(|c| c.id != id);
                                                this.rebuild(cx);
                                                window.focus(&this.focus);
                                            }),
                                        ),
                                    ),
                            )
                        }
                    }
                }
            }
            Some(body.child(content))
        }
        fn cube_camera(&self) -> Camera {
            let mut camera = self.camera.clone();
            if self.sketch {
                camera.set_direction(nalgebra::Vector3::z(), nalgebra::Vector3::y());
            }
            camera
        }
        fn navigation(&self, cx: &mut Context<Self>) -> Div {
            let camera = self.cube_camera();
            let capture = self.cube_bounds.clone();
            let mut overlay = div()
                .occlude()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|_, _, _, cx| cx.stop_propagation()),
                )
                .on_mouse_down(
                    MouseButton::Middle,
                    cx.listener(|_, _, _, cx| cx.stop_propagation()),
                )
                .absolute()
                .top(px(12.))
                .right(px(12.))
                .w(px(124.))
                .flex()
                .flex_col()
                .gap_1();
            overlay = overlay.child(
                div()
                    .id("view-cube")
                    .h(px(112.))
                    .w(px(124.))
                    .occlude()
                    .cursor_pointer()
                    .child(
                        canvas(
                            move |bounds, _, _| capture.set(Some(bounds)),
                            move |bounds, _, window, cx| paint_cube(bounds, &camera, window, cx),
                        )
                        .size_full(),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseDownEvent, _, cx| {
                            if let Some(bounds) = this.cube_bounds.get() {
                                let p = [
                                    f64::from(event.position.x - bounds.origin.x),
                                    f64::from(event.position.y - bounds.origin.y),
                                ];
                                if let Some(face) = view_cube::pick(&this.cube_camera(), p) {
                                    if this.sketch {
                                        if face.label == "Top" {
                                            this.fit();
                                        } else {
                                            this.status = "Sketch view is locked to XY".into();
                                        }
                                    } else {
                                        this.camera.set_direction(face.direction, face.up);
                                        this.changed_camera();
                                    }
                                    cx.stop_propagation();
                                    cx.notify();
                                }
                            }
                        }),
                    ),
            );
            overlay = overlay.child(
                div()
                    .flex()
                    .justify_center()
                    .gap_1()
                    .child(
                        button("view-home", "center_to_page", "Home view", false, true).on_click(
                            cx.listener(|this, _, _, cx| {
                                let projection = this.camera.projection;
                                this.camera = Camera::default();
                                this.camera.projection = projection;
                                this.fit();
                                cx.notify();
                            }),
                        ),
                    )
                    .child(
                        button(
                            "view-projection",
                            "camera",
                            if self.camera.projection == ProjectionMode::Perspective {
                                "Switch to orthographic"
                            } else {
                                "Switch to perspective"
                            },
                            self.camera.projection == ProjectionMode::Orthographic,
                            true,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.camera.projection =
                                if this.camera.projection == ProjectionMode::Perspective {
                                    ProjectionMode::Orthographic
                                } else {
                                    ProjectionMode::Perspective
                                };
                            this.changed_camera();
                            cx.notify();
                        })),
                    )
                    .child(
                        button("view-fit", "zoom_auto", "Fit view · F", false, true).on_click(
                            cx.listener(|this, _, _, cx| {
                                this.fit();
                                cx.notify();
                            }),
                        ),
                    ),
            );
            overlay.child(
                div()
                    .flex()
                    .justify_center()
                    .gap_1()
                    .child(
                        button("grid-toggle", "grid", "Grid", self.grid, true).on_click(
                            cx.listener(|this, _, _, cx| {
                                this.grid = !this.grid;
                                cx.notify();
                            }),
                        ),
                    )
                    .child(
                        button("snap-toggle", "snap_grid", "Snap to grid", self.snap, true)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.snap = !this.snap;
                                cx.notify();
                            })),
                    )
                    .child(
                        button(
                            "view-settings",
                            "options",
                            "View settings",
                            self.panel == Some(Panel::View),
                            true,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.panel = if this.panel == Some(Panel::View) {
                                None
                            } else {
                                Some(Panel::View)
                            };
                            cx.notify();
                        })),
                    ),
            )
        }
        fn timeline(&self, cx: &mut Context<Self>) -> Div {
            let mut row = div()
                .h(px(48.))
                .flex_none()
                .flex()
                .items_center()
                .gap_1()
                .px_2()
                .bg(rgb(t::PANEL))
                .border_t_1()
                .border_color(rgb(t::BORDER));
            row = row
                .child(
                    button("timeline-start", "upmost", "Before extrusion", false, true).on_click(
                        cx.listener(|this, _, _, cx| {
                            let id = this.design.construction.first().map(|f| f.id);
                            this.construction_step(id, false, cx);
                        }),
                    ),
                )
                .child(
                    button(
                        "timeline-previous",
                        "up",
                        "Previous construction feature",
                        false,
                        true,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        let index = this
                            .construction_cursor
                            .and_then(|id| this.design.construction.iter().position(|f| f.id == id))
                            .unwrap_or(this.design.construction.len());
                        let id = this
                            .design
                            .construction
                            .get(index.saturating_sub(1))
                            .map(|f| f.id);
                        this.construction_step(id, false, cx);
                    })),
                )
                .child(separator());
            let mut strip = div()
                .id("construction-timeline")
                .flex_1()
                .min_w_0()
                .h_full()
                .flex()
                .items_center()
                .gap_2()
                .overflow_x_scroll();
            let cursor = self
                .construction_cursor
                .and_then(|id| self.design.construction.iter().position(|f| f.id == id));
            for (index, f) in self.design.construction.iter().enumerate() {
                let id = f.id;
                let active = self.construction_cursor == Some(id);
                let icon_name = match f.kind {
                    ConstructionKind::Sketch => "line_rectangle",
                    ConstructionKind::Extrude { .. } => "up",
                };
                strip = strip
                    .child(
                        button(
                            SharedString::from(format!("feature-{id}")),
                            icon_name,
                            &format!("{} · Edit feature", f.name),
                            active,
                            true,
                        )
                        .opacity(if cursor.is_some_and(|i| index > i) {
                            0.35
                        } else {
                            1.
                        })
                        .on_click(cx.listener(
                            move |this, _, window, cx| {
                                this.construction_step(Some(id), true, cx);
                                window.focus(&this.focus);
                            },
                        )),
                    )
                    .when(active, |s| {
                        s.child(div().w(px(2.)).h(px(28.)).bg(rgb(t::ACCENT)).flex_none())
                    });
            }
            row.child(strip).child(separator()).child(
                button(
                    "timeline-end",
                    "downmost",
                    "Restore all construction features",
                    false,
                    true,
                )
                .on_click(cx.listener(|this, _, _, cx| this.construction_step(None, false, cx))),
            )
        }
        fn menu_overlay(&self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
            let name = self.menu?;
            let features: Vec<Feature> = if name == "Export" {
                vec![
                    Feature {
                        id: "export-step",
                        name: "STEP",
                        icon: "export",
                        action: None,
                    },
                    Feature {
                        id: "export-stl",
                        name: "STL",
                        icon: "export",
                        action: None,
                    },
                ]
            } else {
                toolbar::groups(self.mode)
                    .into_iter()
                    .find(|g| g.name == name)?
                    .features
                    .to_vec()
            };
            let mut menu = div()
                .id("tool-menu")
                .absolute()
                .top(px(104.))
                .left(px(self.menu_x))
                .w(px(328.))
                .max_h(px(440.))
                .flex()
                .flex_col()
                .bg(rgb(t::PANEL))
                .border_1()
                .border_color(rgb(t::BORDER))
                .rounded_sm()
                .occlude()
                .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                    this.menu = None;
                    cx.notify();
                }));
            menu = menu.child(div().px_3().py_2().text_color(rgb(t::MUTED)).child(name));
            let mut list = div()
                .id("tool-menu-list")
                .overflow_y_scroll()
                .min_h_0()
                .flex_1()
                .py_1();
            for feature in features {
                list = list.child(
                    div()
                        .id(feature.id)
                        .h(px(31.))
                        .px_3()
                        .flex()
                        .items_center()
                        .gap_3()
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(t::HOVER)))
                        .child(icon(
                            feature.icon,
                            18.,
                            if feature.available() {
                                t::TEXT
                            } else {
                                t::MUTED
                            },
                        ))
                        .child(div().flex_1().child(feature.name))
                        .when(!feature.available(), |el| {
                            el.child(
                                div()
                                    .text_size(px(10.))
                                    .text_color(rgb(t::DISABLED))
                                    .child("Not implemented"),
                            )
                        })
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.feature(feature, window, cx)
                        })),
                );
            }
            Some(menu.child(list))
        }
        fn scene_element(&self, cx: &mut Context<Self>) -> Stateful<Div> {
            let coords: Vec<_> = if self.solved.len() == self.design.points.len() {
                self.solved.clone()
            } else {
                self.design.points.iter().map(|p| p.xy).collect()
            };
            let lines = self
                .design
                .lines
                .iter()
                .map(|l| {
                    (
                        l.id,
                        l.ends.map(|id| {
                            coords[self.design.points.iter().position(|p| p.id == id).unwrap()]
                        }),
                    )
                })
                .collect();
            let mut annotations = Vec::new();
            for c in &self.design.constraints {
                let pair = match &c.kind {
                    ConstraintKind::DistanceX { points, parameter }
                    | ConstraintKind::DistanceY { points, parameter } => {
                        Some((*points, *parameter))
                    }
                    ConstraintKind::Length { line, parameter } => self
                        .design
                        .lines
                        .iter()
                        .find(|l| l.id == *line)
                        .map(|l| (l.ends, *parameter)),
                    _ => None,
                };
                if let Some((points, id)) = pair {
                    let [a, b] = points.map(|id| {
                        coords[self.design.points.iter().position(|p| p.id == id).unwrap()]
                    });
                    if let Some(p) = self.design.parameters.iter().find(|p| p.id == id) {
                        annotations.push((
                            [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5],
                            format!("{} = {}", p.name, p.expression),
                        ));
                    }
                }
            }
            let overlay = SketchCanvas {
                bounds: self.bounds.clone(),
                active: self.sketch,
                visible: !self.hidden.contains("sketches"),
                grid: self.grid,
                axes: !self.hidden.contains("origin"),
                coords,
                lines,
                annotations,
                selected: self.line,
                anchor: self.anchor,
                hover: self.hover,
                rectangle: self.tool == Tool::Rectangle,
                scale: self.scale,
                center: self.center,
            };
            let capture = self.bounds.clone();
            div()
                .id("model-viewport")
                .relative()
                .flex_1()
                .min_w_0()
                .min_h_0()
                .overflow_hidden()
                .bg(rgb(t::VIEWPORT))
                .child(wgpu_surface(self.gpu.surface.clone()).absolute().inset_0())
                .child(overlay.element())
                .when(self.mode == Mode::Drawing, |el| {
                    el.child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .w(px(420.))
                                    .h(px(297.))
                                    .bg(rgb(t::TEXT))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(icon("drawing_settings", 32., t::DISABLED)),
                            ),
                    )
                })
                .child(
                    canvas(
                        move |bounds, _, _| capture.set(Some(bounds)),
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .inset_0(),
                )
                .child(self.navigation(cx))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, event: &MouseDownEvent, window, cx| {
                        if this.mode == Mode::Drawing {
                            return;
                        }
                        window.focus(&this.focus);
                        this.menu = None;
                        if this.sketch {
                            this.sketch_click(event.position, cx);
                        } else {
                            this.pick(event.position);
                        }
                        cx.notify();
                    }),
                )
                .on_mouse_down(
                    MouseButton::Middle,
                    cx.listener(|this, event: &MouseDownEvent, window, _| {
                        if this.mode == Mode::Drawing {
                            return;
                        }
                        window.focus(&this.focus);
                        this.drag = Some(Drag {
                            button: event.button,
                            previous: event.position,
                        });
                    }),
                )
                .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                    if this.sketch {
                        this.hover = this.screen_to_sketch(event.position);
                    }
                    let Some(drag) = &mut this.drag else {
                        return;
                    };
                    if event.pressed_button != Some(drag.button) {
                        this.drag = None;
                        return;
                    }
                    let delta = [
                        f64::from(event.position.x - drag.previous.x),
                        f64::from(event.position.y - drag.previous.y),
                    ];
                    drag.previous = event.position;
                    if this.sketch {
                        this.center[0] -= delta[0] / this.scale;
                        this.center[1] += delta[1] / this.scale;
                    } else if event.modifiers.control {
                        this.camera.free_orbit(delta);
                    } else if event.modifiers.shift {
                        this.camera.orbit(delta);
                    } else {
                        let height = this.bounds.get().map_or(1., |b| f64::from(b.size.height));
                        this.camera.pan(delta, height);
                    }
                    this.changed_camera();
                    cx.notify();
                }))
                .on_mouse_up(
                    MouseButton::Middle,
                    cx.listener(|this, _, _, _| this.drag = None),
                )
                .on_mouse_up_out(
                    MouseButton::Middle,
                    cx.listener(|this, _, _, _| this.drag = None),
                )
                .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _, cx| {
                    if this.mode == Mode::Drawing {
                        return;
                    }
                    let delta = f64::from(event.delta.pixel_delta(px(40.)).y);
                    if this.sketch {
                        this.scale = (this.scale * (delta * 0.0025).exp()).clamp(500., 100000.);
                    } else {
                        this.camera.zoom(delta);
                    }
                    this.changed_camera();
                    cx.notify();
                }))
        }
    }
    struct GroupTip(&'static str);
    impl Render for GroupTip {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .px_3()
                .py_2()
                .bg(rgb(t::PANEL))
                .border_1()
                .border_color(rgb(t::BORDER))
                .text_color(rgb(t::TEXT))
                .text_size(px(12.))
                .child(self.0)
        }
    }
    fn tree_item(
        id: impl Into<ElementId>,
        image: &str,
        label: &str,
        selected: bool,
    ) -> Stateful<Div> {
        div()
            .id(id)
            .h(px(28.))
            .pl(px(38.))
            .pr_2()
            .flex()
            .items_center()
            .gap_2()
            .cursor_pointer()
            .bg(rgb(if selected { t::SELECTED } else { t::PANEL }))
            .hover(|s| s.bg(rgb(t::HOVER)))
            .child(icon(image, 15., t::MUTED))
            .child(label.to_owned())
    }
    fn paint_cube(bounds: Bounds<Pixels>, camera: &Camera, window: &mut Window, cx: &mut App) {
        let screen = |p: [f64; 2]| bounds.origin + point(px(p[0] as f32), px(p[1] as f32));
        for face in view_cube::faces(camera) {
            let mut path = PathBuilder::fill();
            path.move_to(screen(face.polygon[0]));
            for p in &face.polygon[1..] {
                path.line_to(screen(*p));
            }
            path.close();
            if let Ok(path) = path.build() {
                window.paint_path(
                    path,
                    rgb(if face.label == "Top" {
                        t::HOVER
                    } else {
                        t::PANEL
                    }),
                );
            }
            let mut path = PathBuilder::stroke(px(1.));
            path.move_to(screen(face.polygon[0]));
            for p in &face.polygon[1..] {
                path.line_to(screen(*p));
            }
            path.close();
            if let Ok(path) = path.build() {
                window.paint_path(path, rgb(t::BORDER));
            }
            let center = face
                .polygon
                .iter()
                .fold([0., 0.], |a, b| [a[0] + b[0] * 0.25, a[1] + b[1] * 0.25]);
            paint_label(
                face.label,
                screen(center) - point(px(12.), px(6.)),
                t::MUTED,
                10.,
                window,
                cx,
            );
        }
        let origin = [25., 94.];
        for (name, axis, color) in [
            ("X", nalgebra::Vector3::x(), t::ERROR),
            ("Y", nalgebra::Vector3::y(), t::SUCCESS),
            ("Z", nalgebra::Vector3::z(), t::ACCENT),
        ] {
            let end = [
                origin[0] + axis.dot(&camera.right()) * 18.,
                origin[1] - axis.dot(&camera.up()) * 18.,
            ];
            let mut path = PathBuilder::stroke(px(1.));
            path.move_to(screen(origin));
            path.line_to(screen(end));
            if let Ok(path) = path.build() {
                window.paint_path(path, rgb(color));
            }
            paint_label(
                name,
                screen(end) + point(px(2.), px(-5.)),
                color,
                9.,
                window,
                cx,
            );
        }
    }
    fn paint_label(
        label: &str,
        position: Point<Pixels>,
        color: u32,
        font_size: f32,
        window: &mut Window,
        cx: &mut App,
    ) {
        let style = window.text_style();
        let text: SharedString = label.to_owned().into();
        let run = TextRun {
            len: text.len(),
            font: style.font(),
            color: rgb(color).into(),
            background_color: None,
            underline: None,
            strikethrough: None,
            letter_spacing: None,
        };
        let line = window
            .text_system()
            .shape_line(text, px(font_size), &[run], None);
        let _ = line.paint(position, px(font_size + 4.), window, cx);
    }
    impl Render for WorkspaceView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            if let Some(result) = self.worker.poll()
                && result.revision == self.revision
            {
                match result.solution {
                    Ok(solution) => {
                        self.solved = solution.points;
                        self.status = if self.design.points.is_empty() {
                            "Ready".into()
                        } else if solution.conflicts.is_empty() {
                            if solution.dof == 0 {
                                "Fully constrained".into()
                            } else {
                                format!("{} DOF", solution.dof)
                            }
                        } else {
                            format!("{} conflicting constraints", solution.conflicts.len())
                        };
                        self.conflicts = solution.conflicts;
                    }
                    Err(e) => self.error = Some(e),
                }
                match result.mesh {
                    Ok(Some(mesh)) => {
                        self.volume = Some(mesh.volume);
                        self.mesh = Some((
                            mesh.vertices
                                .iter()
                                .map(|v| DemoVertex {
                                    position: [
                                        (v.x * 25.) as f32,
                                        (v.y * 25.) as f32,
                                        (v.z * 25.) as f32,
                                    ],
                                    normal: [v.nx as f32, v.ny as f32, v.nz as f32],
                                    face: v.face,
                                })
                                .collect(),
                            mesh.indices,
                        ));
                        self.refresh_mesh();
                    }
                    Ok(None) => {}
                    Err(e) => self.error = Some(e),
                }
            }
            self.gpu.set_grid(self.mode == Mode::Solid && self.grid);
            match self
                .gpu
                .draw(&self.camera, self.selected, self.pending_pick.take())
            {
                Ok(Some(pick)) if pick.revision == self.view_revision => self.selected = pick.face,
                Ok(_) => {}
                Err(e) => self.error = Some(e),
            }
            let entity = cx.entity().downgrade();
            window.on_next_frame(move |_, cx| {
                let _ = entity.update(cx, |_, cx| cx.notify());
            });
            let status = self.error.clone().unwrap_or_else(|| {
                if self.mode == Mode::Drawing {
                    "Drawing · Not implemented".into()
                } else {
                    self.status.clone()
                }
            });
            let mut main = div()
                .flex()
                .flex_1()
                .min_h_0()
                .child(self.browser(cx))
                .child(self.scene_element(cx));
            if let Some(inspector) = self.inspector(cx) {
                main = main.child(inspector);
            }
            let menu = self.menu_overlay(cx);
            div()
                .size_full()
                .relative()
                .flex()
                .flex_col()
                .bg(rgb(t::VIEWPORT))
                .text_color(rgb(t::TEXT))
                .font_family("sans-serif")
                .text_size(px(12.))
                .track_focus(&self.focus)
                .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                    this.keyboard(event, window, cx)
                }))
                .child(self.top_bar(cx))
                .child(self.ribbon(cx))
                .child(main)
                .child(self.timeline(cx))
                .child(
                    div()
                        .h(px(24.))
                        .flex_none()
                        .px_3()
                        .flex()
                        .items_center()
                        .justify_between()
                        .bg(rgb(t::PANEL))
                        .border_t_1()
                        .border_color(rgb(t::BORDER))
                        .text_size(px(11.))
                        .child(
                            div()
                                .text_color(rgb(if self.error.is_some() {
                                    t::ERROR
                                } else {
                                    t::MUTED
                                }))
                                .child(status),
                        )
                        .child(
                            div().text_color(rgb(t::MUTED)).child(
                                self.volume
                                    .map_or("mm".into(), |v| format!("{:.0} mm³", v * 1e9)),
                            ),
                        ),
                )
                .children(menu)
        }
    }
}
#[cfg(feature = "desktop")]
pub use implementation::WorkspaceView;
