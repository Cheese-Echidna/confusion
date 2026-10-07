//! Multi-document CAD shell with a floating browser, pinned tools and construction marker.
//!
//! Exports WorkspaceView behind desktop. Connections: application/bootstrap creates
//! the view, render/gpui_bridge owns compositor integration, camera handles navigation.
//! Tab snapshots retain independent design/undo/view state; settings/store persists toolbar pins.
//! UI coordinates are normalized against actual layout bounds before GPU picking.

#[cfg(feature = "desktop")]
mod implementation {
    use crate::render::{
        camera::{Camera, ProjectionMode},
        gpui_bridge::GpuViewport,
        scene::DemoVertex,
    };
    use crate::ui::{
        components::{button, icon, separator, text_button},
        sketch_canvas::SketchCanvas,
        theme as t,
        toolbar::{self, Action, Feature, Mode},
        view_cube,
    };
    use crate::{
        document::model::{CapRole, ExtrudeFeature, ExtrudeOperation, SketchPlane},
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
        Parameters,
        Extrude,
        Create,
        Dimension,
        View,
        Measure,
        Inspect,
        Offset,
        Search,
        Transform,
        Fillet,
        SolidModify,
        Materials,
    }

    #[derive(Clone, Copy, PartialEq)]
    enum Tool {
        Select,
        Line,
        Rectangle,
        Circle,
        Circle2,
        Circle3,
        Arc3,
        Ellipse,
        Polygon,
        Slot,
        Spline,
        CenterRectangle,
        Arc,
        Point,
        Dimension,
        Measure,
        Break,
        Trim,
        Extend,
    }

    struct SketchDrag {
        before: Design,
        points: Vec<Uuid>,
        start: [f64; 2],
        moved: bool,
    }
    struct Drag {
        button: MouseButton,
        previous: Point<Pixels>,
    }

    #[derive(Clone, Copy)]
    enum CloseTarget {
        Tab(usize),
        Window,
    }
    #[derive(Clone)]
    struct DocumentTab {
        recovery_id: Uuid,
        design: Design,
        dirty: bool,
        undo: Vec<Design>,
        redo: Vec<Design>,
        path: String,
        saved_path: Option<PathBuf>,
        saved_fingerprint: blake3::Hash,
        mode: Mode,
        camera: Camera,
        scale: f64,
        center: [f64; 2],
    }

    pub struct WorkspaceView {
        recovery: Option<crate::persistence::recovery::RecoveryManager>,
        recovery_schedule: crate::application::session::RecoverySchedule,
        recovery_check: std::time::Instant,
        recovery_tracked: HashSet<Uuid>,
        dirty_cache: Cell<(u64, bool)>,
        extrude_drag: Option<(Point<Pixels>, f64, [f64; 2])>,
        create_editor: Option<CreateEditor>,
        gpu: GpuViewport,
        ui_font_family: SharedString,
        world_sketches: Vec<(Uuid, Design, crate::sketch::workplane::Workplane)>,
        body_faces: std::collections::HashMap<u32, (Uuid, u32)>,
        hidden_sketches: HashSet<Uuid>,
        hidden_bodies: HashSet<(Uuid, u32)>,
        align_sketch_pending: bool,
        frame_rate: crate::ui::frame_rate::FrameRate,
        native_dialog: Option<(bool, std::sync::mpsc::Receiver<Option<PathBuf>>)>,
        export_dialog: Option<(
            crate::exchange::export::ExportFormat,
            std::sync::mpsc::Receiver<Option<PathBuf>>,
        )>,
        export_result: Option<std::sync::mpsc::Receiver<Result<PathBuf, String>>>,
        step_export_dialog: Option<std::sync::mpsc::Receiver<Option<PathBuf>>>,
        close_choice: usize,
        close_target: Option<CloseTarget>,
        close_approved: HashSet<usize>,
        close_hook_installed: bool,
        camera: Camera,
        view_revision: u64,
        selected: u32,
        face_anchors: Vec<crate::kernel::bridge::ffi::FaceAnchor>,
        solid_editor: SolidEditor,
        evaluated_features: Vec<Uuid>,
        extrude_operation: ExtrudeOperation,
        extrude_target: Option<Uuid>,
        editing_feature: Option<Uuid>,
        fit_pending: bool,
        pending_pick: Option<([f64; 2], u64)>,
        bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
        drag: Option<Drag>,
        focus: FocusHandle,
        error: Option<String>,
        design: Design,
        worker: Worker,
        pending_candidate: Option<crate::runtime::candidate::CandidateEdit>,
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
        saved_fingerprint: blake3::Hash,
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
        menu_leave_deadline: Option<std::time::Instant>,
        choosing_sketch_face: bool,
        sketch_region: Option<crate::sketch::regions::Region>,
        before_construction: bool,
        documents: Vec<DocumentTab>,
        active_document: usize,
        pinned: HashSet<&'static str>,
        timeline_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
        timeline_drag: bool,
        selection: Vec<Uuid>,
        point_dof: Vec<usize>,
        sketch_drag: Option<SketchDrag>,
        arc_start: Option<[f64; 2]>,
        keymap: crate::settings::keymap::Keymap,
        reference_dimension: bool,
        dimension_edit: Option<Uuid>,
        dimension_position: Option<[f64; 2]>,
        search: Entity<TextInput>,
        tool_points: Vec<[f64; 2]>,
        transform_action: Action,
        transform_x: Entity<TextInput>,
        transform_y: Entity<TextInput>,
        transform_angle: Entity<TextInput>,
        transform_count: Entity<TextInput>,
        transform_copy: bool,
        marquee: Option<([f64; 2], [f64; 2], bool)>,
        transform_rows: Entity<TextInput>,
        show_constraints: bool,
        show_dimensions: bool,
        dimension_drag: Option<(Uuid, Design, bool)>,
        editor_input: Option<EntityId>,
        search_index: usize,
        search_query: String,
        inspection: Option<crate::kernel::bridge::ffi::Inspection>,
        inspection_action: Action,
        inspection_preview: bool,
        section_axis: usize,
        section_position: f32,
        section_offset: Entity<TextInput>,
    }

    include!("sketch_workflow.rs");
    include!("solid_create.rs");
    include!("solid_modify.rs");
    include!("solid_inspect.rs");
    include!("step_exchange.rs");

    impl WorkspaceView {
        pub fn new(surface: WgpuSurfaceHandle, cx: &mut Context<Self>) -> Self {
            // Resolve a real installed sans family once. Generic CSS names are not
            // mapped by the native text backend and repeat costly error creation.
            let fonts = cx.text_system().all_font_names();
            let family = [
                "Segoe UI",
                "Helvetica Neue",
                "Helvetica",
                "Noto Sans",
                "DejaVu Sans",
                "Liberation Sans",
                "Arial",
            ]
            .into_iter()
            .find(|candidate| fonts.iter().any(|name| name == candidate));
            let ui_font_family = family.map(SharedString::from).unwrap_or_else(|| {
                let id = cx.text_system().resolve_font(&gpui::font(".SystemUIFont"));
                cx.text_system()
                    .get_font_for_id(id)
                    .expect("resolved system font")
                    .family
            });
            let saved_pins = crate::settings::store::toolbar_pins();
            let mut gpu = GpuViewport::new(surface);
            gpu.set_mesh(&[], &[]);
            let keymap = crate::settings::keymap::load();
            let keymap_error = keymap.as_ref().err().cloned();
            let recovery = crate::platform::paths::recovery_directory()
                .and_then(|path| crate::persistence::recovery::RecoveryManager::new(&path));
            let recovery_error = recovery.as_ref().err().cloned();
            Self {
                recovery: recovery.ok(),
                recovery_schedule: Default::default(),
                recovery_check: std::time::Instant::now(),
                recovery_tracked: HashSet::new(),
                dirty_cache: Cell::new((u64::MAX, false)),
                extrude_drag: None,
                create_editor: None,
                gpu,
                ui_font_family,
                world_sketches: vec![],
                body_faces: Default::default(),
                hidden_sketches: Default::default(),
                hidden_bodies: Default::default(),
                align_sketch_pending: false,
                frame_rate: Default::default(),
                native_dialog: None,
                export_dialog: None,
                export_result: None,
                step_export_dialog: None,
                close_choice: 2,
                close_target: None,
                close_approved: Default::default(),
                close_hook_installed: false,
                camera: Camera::default(),
                view_revision: 0,
                selected: 0,
                face_anchors: vec![],
                solid_editor: SolidEditor::new(cx),
                evaluated_features: vec![],
                extrude_operation: ExtrudeOperation::NewBody,
                extrude_target: None,
                editing_feature: None,
                fit_pending: false,
                pending_pick: None,
                bounds: Rc::new(Cell::new(None)),
                drag: None,
                focus: cx.focus_handle(),
                error: keymap_error.or(recovery_error),
                design: Design::default(),
                worker: Worker::new(),
                revision: 0,
                pending_candidate: None,
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
                saved_fingerprint: crate::document::dirty::fingerprint(&Design::default()),
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
                menu_leave_deadline: None,
                choosing_sketch_face: false,
                sketch_region: None,
                before_construction: false,
                documents: vec![DocumentTab {
                    recovery_id: Uuid::new_v4(),
                    design: Design::default(),
                    dirty: false,
                    undo: vec![],
                    redo: vec![],
                    path: "Untitled.con".into(),
                    saved_path: None,
                    saved_fingerprint: crate::document::dirty::fingerprint(&Design::default()),
                    mode: Mode::Solid,
                    camera: Camera::default(),
                    scale: 6000.,
                    center: [0.04, 0.025],
                }],
                active_document: 0,
                pinned: [Mode::Solid, Mode::Sketch, Mode::Drawing]
                    .into_iter()
                    .flat_map(|m| {
                        toolbar::groups(m).into_iter().flat_map(|g| {
                            g.features
                                .iter()
                                .enumerate()
                                .filter(|(i, f)| {
                                    saved_pins
                                        .as_ref()
                                        .map_or(*i < 3, |pins| pins.contains(f.id))
                                })
                                .map(|(_, f)| f.id)
                        })
                    })
                    .collect(),
                timeline_bounds: Rc::new(Cell::new(None)),
                timeline_drag: false,
                selection: vec![],
                point_dof: vec![],
                sketch_drag: None,
                arc_start: None,
                keymap: keymap.unwrap_or_default(),
                reference_dimension: false,
                dimension_edit: None,
                dimension_position: None,
                search: cx.new(|cx| TextInput::new("", cx).with_placeholder("Search tools")),
                tool_points: vec![],
                transform_action: Action::Move,
                transform_x: cx.new(|cx| TextInput::new("10 mm", cx)),
                transform_y: cx.new(|cx| TextInput::new("0 mm", cx)),
                transform_angle: cx.new(|cx| TextInput::new("0 deg", cx)),
                transform_count: cx.new(|cx| TextInput::new("3", cx)),
                transform_copy: false,
                marquee: None,
                transform_rows: cx.new(|cx| TextInput::new("1", cx)),
                show_constraints: true,
                show_dimensions: true,
                dimension_drag: None,
                editor_input: None,
                search_index: 0,
                search_query: String::new(),
                inspection: None,
                inspection_action: Action::Measure,
                inspection_preview: false,
                section_axis: 2,
                section_position: 0.,
                section_offset: cx.new(|cx| TextInput::new("0 mm", cx)),
            }
        }

        pub fn focus(&self, window: &mut Window) {
            window.focus(&self.focus);
        }

        fn active_frame(&self) -> crate::sketch::workplane::Workplane {
            self.world_sketches
                .iter()
                .find(|(id, _, _)| Some(*id) == self.design.current_sketch_id())
                .map_or_else(Default::default, |(_, _, frame)| *frame)
        }
        fn sketch_screen(&self, at: [f64; 2]) -> Option<[f64; 2]> {
            let bounds = self.bounds.get()?;
            let world = self.active_frame().world(at);
            self.camera.project(
                nalgebra::Point3::from(world.coords * 25.),
                [
                    f64::from(bounds.size.width) as u32,
                    f64::from(bounds.size.height) as u32,
                ],
            )
        }
        fn align_to_sketch(&mut self) {
            let frame = self.active_frame();
            self.camera.set_direction(frame.normal, frame.y);
            self.camera.projection = ProjectionMode::Orthographic;
            self.camera.target = nalgebra::Point3::from(frame.world(self.center).coords * 25.);
            let height = self.bounds.get().map_or(575., |b| f64::from(b.size.height));
            self.camera.distance =
                height * 25. / (2. * self.scale * (Camera::FIELD_OF_VIEW / 2.).tan());
            self.align_sketch_pending = false;
            self.changed_camera();
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
            } else if let Some((vertices, _)) = &self.mesh {
                if !vertices.is_empty() {
                    let mut min = [f64::INFINITY; 3];
                    let mut max = [f64::NEG_INFINITY; 3];
                    for v in vertices {
                        for k in 0..3 {
                            min[k] = min[k].min(v.position[k] as f64);
                            max[k] = max[k].max(v.position[k] as f64);
                        }
                    }
                    self.camera.target = nalgebra::Point3::new(
                        (min[0] + max[0]) * 0.5,
                        (min[1] + max[1]) * 0.5,
                        (min[2] + max[2]) * 0.5,
                    );
                    let diagonal = ((max[0] - min[0]).powi(2)
                        + (max[1] - min[1]).powi(2)
                        + (max[2] - min[2]).powi(2))
                    .sqrt();
                    self.camera.distance = (diagonal * 2.).max(0.1);
                    self.fit_pending = false;
                }
            } else {
                self.fit_pending = true;
            }
            if self.sketch {
                self.align_to_sketch();
            }
            self.changed_camera();
        }
        fn cancel_candidate(&mut self) {
            if self.pending_candidate.take().is_some() {
                self.worker.cancel();
            }
        }
        fn candidate_context(&self, cx: &App) -> String {
            match self.panel {
                Some(Panel::Create) => self.create_editor.as_ref().map_or_else(String::new, |e| {
                    format!(
                        "create:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}",
                        e.kind,
                        e.editing,
                        e.sketch,
                        e.second_sketch,
                        e.target,
                        e.second_target,
                        self.selection,
                        e.fields
                            .iter()
                            .map(|f| f.read(cx).content.to_string())
                            .collect::<Vec<_>>()
                    )
                }),
                Some(Panel::SolidModify) => {
                    let e = &self.solid_editor;
                    format!(
                        "modify:{:?}:{:?}:{:?}:{:?}:{}:{}:{}:{:?}",
                        e.kind,
                        e.editing,
                        e.target,
                        e.tool,
                        e.copy,
                        e.mode,
                        self.selected,
                        e.fields
                            .iter()
                            .map(|f| f.read(cx).content.to_string())
                            .collect::<Vec<_>>()
                    )
                }
                _ => String::new(),
            }
        }
        fn validate_candidate(&mut self, candidate: Design, cx: &mut Context<Self>) {
            if self.pending_candidate.is_some() {
                return;
            }
            self.revision = self.revision.wrapping_add(1);
            self.pending_candidate = Some(crate::runtime::candidate::CandidateEdit::new(
                self.revision,
                &self.design,
                self.candidate_context(cx),
                candidate.clone(),
            ));
            self.error = None;
            self.status = "Validating…".into();
            self.worker.submit(self.revision, candidate);
            cx.notify();
        }
        fn checkpoint(&mut self) {
            self.cancel_candidate();
            self.undo.push(self.design.clone());
            if self.undo.len() > 100 {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
        fn rebuild(&mut self, cx: &mut Context<Self>) {
            self.cancel_candidate();
            self.sketch_region = None;
            self.revision = self.revision.wrapping_add(1);
            self.changed_camera();
            self.selected = 0;
            self.face_anchors.clear();
            self.body_faces.clear();
            self.evaluated_features.clear();
            self.volume = None;
            self.inspection = None;
            self.conflicts.clear();
            self.status = "Solving…".into();
            self.error = None;
            self.design.sync_construction();
            if self.design.solid_features().is_empty() && self.design.create_features.is_empty() {
                self.mesh = None;
                self.gpu.set_mesh(&[], &[]);
            }
            let snapshot = self
                .construction_cursor
                .and_then(|id| self.design.through_feature(id).ok())
                .unwrap_or_else(|| self.design.clone());
            self.worker.submit(
                self.revision,
                if self.before_construction {
                    Design::default()
                } else {
                    snapshot
                },
            );
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
        fn extrusion_preview(&self, cx: &App) -> Option<crate::ui::extrude_gizmo::ExtrudeGizmo> {
            if self.panel != Some(Panel::Extrude) {
                return None;
            }
            let d = self.display_design();
            let regions = crate::sketch::regions::regions(
                &d,
                &d.points.iter().map(|p| p.xy).collect::<Vec<_>>(),
            )
            .ok()?;
            let region = regions
                .iter()
                .find(|r| r.boundary.iter().any(|id| self.selection.contains(id)))
                .or_else(|| {
                    self.design
                        .solid_features()
                        .iter()
                        .find(|f| Some(f.id) == self.editing_feature)
                        .and_then(|f| regions.iter().find(|r| r.boundary == f.boundary))
                })
                .or_else(|| (regions.len() == 1).then(|| &regions[0]))?;
            let curves: Vec<_> = region
                .boundary
                .iter()
                .map(|id| crate::sketch::entities::samples(&d, *id))
                .collect();
            let points: Vec<_> = curves.iter().flatten().collect();
            if points.is_empty() {
                return None;
            }
            let center = [
                points.iter().map(|p| p[0]).sum::<f64>() / points.len() as f64,
                points.iter().map(|p| p[1]).sum::<f64>() / points.len() as f64,
            ];
            let mut values = self.design.clone();
            let id = values.parameter(
                "preview_depth",
                crate::parameters::expression::dimension_input(&self.depth.read(cx).content, false),
            );
            let depth = *crate::parameters::expression::evaluate(&values)
                .ok()?
                .get(&id)?;
            if depth <= 0. {
                return None;
            }
            let sign = if matches!(
                self.extrude_operation,
                ExtrudeOperation::Cut | ExtrudeOperation::CutNewBody
            ) {
                -1.
            } else {
                1.
            };
            Some(crate::ui::extrude_gizmo::ExtrudeGizmo {
                camera: self.camera.clone(),
                frame: self.active_frame(),
                curves,
                center,
                depth: depth * sign,
            })
        }
        fn open_extrude(&mut self, cx: &mut Context<Self>) {
            self.editing_feature = self
                .design
                .solid_features()
                .iter()
                .find(|f| Some(f.sketch) == self.design.current_sketch_id())
                .map(|f| f.id);
            if let Some(feature) = self
                .design
                .solid_features()
                .iter()
                .find(|f| Some(f.id) == self.editing_feature)
            {
                self.extrude_operation = feature.operation;
                self.extrude_target = feature.target;
                if let Some(p) = self
                    .design
                    .parameters
                    .iter()
                    .find(|p| p.id == feature.depth)
                {
                    let text = p.expression.clone();
                    self.depth = cx.new(|cx| TextInput::new(&text, cx));
                }
            } else {
                self.extrude_operation =
                    if matches!(self.design.active_plane, SketchPlane::Face { .. }) {
                        ExtrudeOperation::Cut
                    } else {
                        ExtrudeOperation::NewBody
                    };
                self.extrude_target = match self.design.active_plane {
                    SketchPlane::Face { support, .. } => Some(support),
                    _ => self.design.latest_body_feature(),
                };
                self.depth = cx.new(|cx| TextInput::new("5 mm", cx));
            }
            self.panel = Some(Panel::Extrude);
            let frame = self.active_frame();
            if self.camera.outward().dot(&frame.normal).abs() > 0.95 {
                self.camera
                    .set_direction(frame.normal + frame.x * 0.65 - frame.y * 0.65, frame.y);
                self.changed_camera();
            }
        }
        fn create_sketch(&mut self, cx: &mut Context<Self>) {
            let plane = if self.selected != 0 {
                let anchors: Vec<_> = self
                    .face_anchors
                    .iter()
                    .filter(|a| a.face == self.selected)
                    .collect();
                if anchors.len() != 1 || anchors[0].ambiguous {
                    self.error = Some(
                        "Select an unambiguous planar extrusion cap to create a sketch".into(),
                    );
                    cx.notify();
                    return;
                }
                let a = anchors[0];
                SketchPlane::Face {
                    support: self.evaluated_features[a.support as usize],
                    producer: self.evaluated_features[a.producer as usize],
                    role: if a.role == 1 {
                        CapRole::Start
                    } else {
                        CapRole::End
                    },
                }
            } else {
                SketchPlane::Xy
            };
            let mut candidate = self.design.clone();
            if candidate.current_sketch_id().is_none()
                || (candidate.points.is_empty()
                    && candidate.extrusion.is_none()
                    && candidate.features.is_empty())
            {
                candidate.ensure_sketch();
                candidate.active_plane = plane;
            } else if let Err(e) = candidate.create_sketch(plane) {
                self.error = Some(e);
                cx.notify();
                return;
            }
            if let Err(e) = candidate.validate() {
                self.error = Some(e);
                cx.notify();
                return;
            }
            self.checkpoint();
            self.design = candidate;
            self.solved.clear();
            self.selection.clear();
            self.editing_feature = None;
            self.set_mode(Mode::Sketch, cx);
            self.panel = None;
            self.status = match self.design.active_plane {
                SketchPlane::Xy => "Sketch on XY plane".into(),
                SketchPlane::Face { .. } => "Sketch on selected face".into(),
            };
        }
        fn extrude(&mut self, cx: &mut Context<Self>) {
            let mut candidate = self.design.clone();
            let sketch = candidate.ensure_sketch();
            let input = match candidate.sketch_input(sketch) {
                Ok(d) => d,
                Err(e) => {
                    self.error = Some(e);
                    cx.notify();
                    return;
                }
            };
            let parameters = match crate::parameters::expression::evaluate(&candidate) {
                Ok(p) => p,
                Err(e) => {
                    self.error = Some(e);
                    cx.notify();
                    return;
                }
            };
            let solution = match crate::solver::nonlinear::solve(&input, &parameters) {
                Ok(s) if s.conflicts.is_empty() => s,
                Ok(_) => {
                    self.error = Some("Resolve sketch conflicts before extruding".into());
                    cx.notify();
                    return;
                }
                Err(e) => {
                    self.error = Some(e);
                    cx.notify();
                    return;
                }
            };
            let existing = candidate
                .solid_features()
                .into_iter()
                .find(|f| Some(f.id) == self.editing_feature);
            let region =
                crate::sketch::regions::regions(&input, &solution.points).and_then(|regions| {
                    let selected: Vec<_> = regions
                        .iter()
                        .filter(|r| r.boundary.iter().any(|id| self.selection.contains(id)))
                        .collect();
                    if selected.len() == 1 {
                        Ok(selected[0].clone())
                    } else if selected.is_empty() {
                        crate::sketch::regions::select(
                            &input,
                            &solution.points,
                            existing.as_ref().map_or(&[][..], |e| e.boundary.as_slice()),
                        )
                    } else {
                        Err("Select a boundary curve from one region before extruding".into())
                    }
                });
            let region = match region {
                Ok(r) => r,
                Err(e) => {
                    self.error = Some(e);
                    cx.notify();
                    return;
                }
            };
            let expression =
                crate::parameters::expression::dimension_input(&self.depth.read(cx).content, false);
            let depth = if let Some(e) = &existing {
                candidate
                    .parameters
                    .iter_mut()
                    .find(|p| p.id == e.depth)
                    .unwrap()
                    .expression = expression;
                e.depth
            } else {
                let mut n = candidate.solid_features().len() + 1;
                while candidate
                    .parameters
                    .iter()
                    .any(|p| p.name == format!("depth{n}"))
                {
                    n += 1;
                }
                candidate.parameter(&format!("depth{n}"), expression)
            };
            let target = if self.extrude_operation == ExtrudeOperation::NewBody {
                None
            } else {
                self.extrude_target
            };
            if existing
                .as_ref()
                .is_some_and(|e| candidate.extrusion.as_ref().is_some_and(|b| b.id == e.id))
            {
                if self.extrude_operation != ExtrudeOperation::NewBody {
                    self.error = Some("The first extrusion creates a new body".into());
                    cx.notify();
                    return;
                }
                let e = candidate.extrusion.as_mut().unwrap();
                e.depth = depth;
                e.boundary = region.boundary;
            } else if candidate.extrusion.is_none()
                && candidate.features.is_empty()
                && candidate.primary_sketch_id() == Some(sketch)
                && self.extrude_operation == ExtrudeOperation::NewBody
            {
                candidate.extrusion = Some(Extrusion {
                    id: Uuid::new_v4(),
                    depth,
                    boundary: region.boundary,
                });
            } else {
                let feature = ExtrudeFeature {
                    id: existing.as_ref().map_or_else(Uuid::new_v4, |e| e.id),
                    name: existing.as_ref().map_or_else(
                        || format!("Extrude {}", candidate.solid_features().len() + 1),
                        |e| e.name.clone(),
                    ),
                    sketch,
                    boundary: region.boundary,
                    depth,
                    operation: self.extrude_operation,
                    target,
                };
                if let Some(index) = candidate.features.iter().position(|f| f.id == feature.id) {
                    candidate.features[index] = feature;
                } else {
                    candidate.features.push(feature);
                }
            }
            candidate.sync_construction();
            if let Err(e) = candidate
                .validate()
                .and_then(|_| crate::parameters::expression::evaluate(&candidate).map(|_| ()))
            {
                self.error = Some(e);
                cx.notify();
                return;
            }
            self.checkpoint();
            self.design = candidate;
            self.inputs.retain(|(id, _)| *id != depth);
            self.sketch = false;
            self.mode = Mode::Solid;
            self.panel = None;
            self.construction_cursor = None;
            self.before_construction = false;
            self.anchor = None;
            self.tool = Tool::Select;
            self.editing_feature = None;
            self.rebuild(cx);
            self.fit();
        }
        fn forget_recovery(&mut self, index: usize) {
            if let Some(manager) = &mut self.recovery {
                if let Err(error) = manager.forget(self.documents[index].recovery_id) {
                    self.error = Some(error);
                }
            }
        }
        fn poll_recovery(&mut self, _cx: &mut Context<Self>) {
            if let Some(result) = self.recovery.as_mut().and_then(|m| m.poll()) {
                if let Err(error) = result {
                    self.error = Some(error);
                    self.recovery_schedule.failed();
                }
            }
            let now = std::time::Instant::now();
            if now.duration_since(self.recovery_check).as_secs() < 1 {
                return;
            }
            self.recovery_check = now;
            if self.recovery.is_none() {
                return;
            }
            let mut hash = blake3::Hasher::new();
            let mut dirty = vec![];
            for (index, tab) in self.documents.iter().enumerate() {
                let (design, saved) = if index == self.active_document {
                    (&self.design, self.saved_fingerprint)
                } else {
                    (&tab.design, tab.saved_fingerprint)
                };
                let fingerprint = crate::document::dirty::fingerprint(design);
                if fingerprint != saved && !self.close_approved.contains(&index) {
                    hash.update(tab.recovery_id.as_bytes());
                    hash.update(fingerprint.as_bytes());
                    dirty.push((tab.recovery_id, design));
                }
            }
            let current: HashSet<_> = dirty.iter().map(|(id, _)| *id).collect();
            let removed: Vec<_> = self
                .recovery_tracked
                .difference(&current)
                .copied()
                .collect();
            for id in removed {
                match self.recovery.as_mut().unwrap().forget(id) {
                    Ok(()) => {
                        self.recovery_tracked.remove(&id);
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            if self.recovery_schedule.observe(hash.finalize(), now) {
                let snapshots = dirty
                    .into_iter()
                    .map(|(id, design)| (id, design.clone()))
                    .collect();
                if self.recovery.as_mut().unwrap().snapshot(snapshots) {
                    self.recovery_tracked = current;
                    self.recovery_schedule.submitted(now);
                }
            }
        }
        fn recovery_prompt(&self, cx: &mut Context<Self>) -> Option<Div> {
            let candidate = self.recovery.as_ref()?.candidates.first()?;
            let damaged = candidate.error.is_some();
            Some(
                div()
                    .absolute()
                    .top(px(70.))
                    .left(px(20.))
                    .p_4()
                    .bg(rgb(t::PANEL))
                    .child("Unsaved work from a previous session")
                    .child(candidate.label.clone())
                    .child(candidate.error.clone().unwrap_or_else(|| {
                        "Recover into an unsaved tab, or discard this snapshot.".into()
                    }))
                    .child(
                        text_button("recover-snapshot", "Recover", "Restore document").when(
                            !damaged,
                            |el| {
                                el.on_click(cx.listener(|this, _, _, cx| {
                                    let result = this.recovery.as_mut().unwrap().recover(0);
                                    match result {
                                        Ok((id, design)) => {
                                            this.new_document(cx);
                                            this.documents[this.active_document].recovery_id = id;
                                            this.recovery_tracked.insert(id);
                                            this.design = design;
                                            this.saved_path = None;
                                            this.saved_fingerprint =
                                                blake3::hash(b"recovered unsaved document");
                                            this.dirty_cache.set((u64::MAX, false));
                                            this.path =
                                                cx.new(|cx| TextInput::new("Recovered.con", cx));
                                            this.restore_edit(cx);
                                            this.status =
                                                "Recovered document; save it to keep your work"
                                                    .into();
                                        }
                                        Err(error) => {
                                            this.error = Some(format!("Recovery failed: {error}"))
                                        }
                                    }
                                    cx.notify();
                                }))
                            },
                        ),
                    )
                    .child(
                        text_button(
                            "discard-snapshot",
                            "Discard",
                            "Permanently remove this snapshot",
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Err(error) = this.recovery.as_mut().unwrap().discard(0) {
                                this.error = Some(error);
                            }
                            cx.notify();
                        })),
                    ),
            )
        }
        fn stash_document(&mut self, cx: &App) {
            self.documents[self.active_document] = DocumentTab {
                recovery_id: self.documents[self.active_document].recovery_id,
                design: self.design.clone(),
                dirty: crate::document::dirty::fingerprint(&self.design) != self.saved_fingerprint,
                undo: self.undo.clone(),
                redo: self.redo.clone(),
                path: self.path.read(cx).content.to_string(),
                saved_path: self.saved_path.clone(),
                saved_fingerprint: self.saved_fingerprint,
                mode: self.mode,
                camera: self.camera.clone(),
                scale: self.scale,
                center: self.center,
            };
        }
        fn switch_document(&mut self, index: usize, cx: &mut Context<Self>) {
            self.mesh = None;
            self.world_sketches.clear();
            self.body_faces.clear();
            self.gpu.set_mesh(&[], &[]);
            self.stash_document(cx);
            self.active_document = index;
            let tab = self.documents[index].clone();
            self.design = tab.design;
            self.undo = tab.undo;
            self.redo = tab.redo;
            self.saved_path = tab.saved_path;
            self.saved_fingerprint = tab.saved_fingerprint;
            self.path = cx.new(|cx| TextInput::new(&tab.path, cx));
            self.mode = tab.mode;
            self.sketch = self.mode == Mode::Sketch;
            self.camera = tab.camera;
            self.scale = tab.scale;
            self.center = tab.center;
            self.panel = None;
            self.menu = None;
            self.restore_edit(cx);
        }
        fn close_document_now(&mut self, index: usize, cx: &mut Context<Self>) {
            self.mesh = None;
            self.world_sketches.clear();
            self.body_faces.clear();
            self.gpu.set_mesh(&[], &[]);
            self.stash_document(cx);
            self.forget_recovery(index);
            self.documents.remove(index);
            if self.documents.is_empty() {
                self.documents.push(DocumentTab {
                    recovery_id: Uuid::new_v4(),
                    design: Design::default(),
                    dirty: false,
                    undo: vec![],
                    redo: vec![],
                    path: "Untitled.con".into(),
                    saved_path: None,
                    saved_fingerprint: crate::document::dirty::fingerprint(&Design::default()),
                    mode: Mode::Solid,
                    camera: Camera::default(),
                    scale: 6000.,
                    center: [0.04, 0.025],
                });
            }
            self.active_document = if index < self.active_document {
                self.active_document - 1
            } else {
                self.active_document.min(self.documents.len() - 1)
            };
            let tab = self.documents[self.active_document].clone();
            // Restore through the normal switching path without overwriting a remaining tab.
            self.design = tab.design.clone();
            self.undo = tab.undo.clone();
            self.redo = tab.redo.clone();
            self.path = cx.new(|cx| TextInput::new(&tab.path, cx));
            self.saved_path = tab.saved_path.clone();
            self.saved_fingerprint = tab.saved_fingerprint;
            self.mode = tab.mode;
            self.camera = tab.camera;
            self.scale = tab.scale;
            self.center = tab.center;
            self.sketch = self.mode == Mode::Sketch;
            self.panel = None;
            self.menu = None;
            self.restore_edit(cx);
        }
        fn new_document(&mut self, cx: &mut Context<Self>) {
            self.mesh = None;
            self.world_sketches.clear();
            self.body_faces.clear();
            self.gpu.set_mesh(&[], &[]);
            self.stash_document(cx);
            self.documents
                .push(self.documents[self.active_document].clone());
            self.active_document = self.documents.len() - 1;
            self.documents[self.active_document].recovery_id = Uuid::new_v4();
            self.undo.clear();
            self.redo.clear();
            self.design = Design::default();
            self.inputs.clear();
            self.solved.clear();
            self.line = None;
            self.anchor = None;
            self.sketch = false;
            self.mode = Mode::Solid;
            self.construction_cursor = None;
            self.before_construction = false;
            self.panel = None;
            self.menu = None;
            self.tool = Tool::Select;
            self.saved_path = None;
            self.saved_fingerprint = crate::document::dirty::fingerprint(&self.design);
            self.dirty_cache.set((u64::MAX, false));
            self.path = cx.new(|cx| TextInput::new("Untitled.con", cx));
            self.depth = cx.new(|cx| TextInput::new("10 mm", cx));
            self.rebuild(cx);
        }
        fn dirty(&self) -> bool {
            let (revision, dirty) = self.dirty_cache.get();
            if revision == self.revision {
                return dirty;
            }
            let dirty = crate::document::dirty::fingerprint(&self.design) != self.saved_fingerprint;
            self.dirty_cache.set((self.revision, dirty));
            dirty
        }
        fn start_file_dialog(&mut self, open: bool, cx: &mut Context<Self>) {
            if self.native_dialog.is_some() {
                return;
            }
            self.menu = None;
            self.panel = None;
            self.error = None;
            self.native_dialog = Some((
                open,
                crate::platform::dialogs::select_file(
                    open,
                    PathBuf::from(self.path.read(cx).content.to_string()),
                ),
            ));
            cx.notify();
        }
        fn request_save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
            if self.saved_path.is_none() {
                self.start_file_dialog(false, cx);
            } else {
                self.save(cx);
                if self.error.is_none() {
                    self.finish_close(window, cx);
                }
            }
        }
        fn request_close_document(&mut self, index: usize, cx: &mut Context<Self>) {
            self.stash_document(cx);
            if crate::document::dirty::fingerprint(&self.documents[index].design)
                != self.documents[index].saved_fingerprint
            {
                if index != self.active_document {
                    self.switch_document(index, cx);
                }
                self.close_choice = 2;
                self.close_target = Some(CloseTarget::Tab(index));
                cx.notify();
            } else {
                self.close_document_now(index, cx);
            }
        }
        pub fn request_window_close(
            &mut self,
            window: &mut Window,
            cx: &mut Context<Self>,
        ) -> bool {
            if self.native_dialog.is_some() {
                return false;
            }
            self.stash_document(cx);
            if let Some(index) = self
                .documents
                .iter()
                .enumerate()
                .find(|(index, tab)| {
                    !self.close_approved.contains(index)
                        && crate::document::dirty::fingerprint(&tab.design) != tab.saved_fingerprint
                })
                .map(|(i, _)| i)
            {
                if index != self.active_document {
                    self.switch_document(index, cx);
                }
                self.close_choice = 2;
                window.focus(&self.focus);
                self.close_target = Some(CloseTarget::Window);
                cx.notify();
                false
            } else {
                for index in 0..self.documents.len() {
                    self.forget_recovery(index);
                }
                // Final intentional shutdown drains ordered cleanup before process exit.
                if let Some(manager) = &self.recovery {
                    if let Err(error) = manager.flush() {
                        self.error = Some(error);
                        return false;
                    }
                }
                true
            }
        }
        fn finish_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
            match self.close_target.take() {
                Some(CloseTarget::Tab(index)) => self.close_document_now(index, cx),
                Some(CloseTarget::Window) if self.request_window_close(window, cx) => {
                    window.remove_window()
                }
                _ => {}
            }
        }
        fn close_confirmation(&self, cx: &mut Context<Self>) -> Option<Div> {
            if self.close_target.is_none() && self.native_dialog.is_none() {
                return None;
            }
            let mut content = div()
                .w(px(380.))
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .bg(rgb(t::PANEL))
                .border_1()
                .border_color(rgb(t::BORDER))
                .rounded_md();
            if self.native_dialog.is_some() {
                content = content.child("Choose a file in the system dialog");
            } else {
                content = content
                    .child(format!("Save changes to {}?", self.document_name(cx)))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(t::MUTED))
                            .child("Your changes will be lost if you discard them."),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(
                                text_button("cancel-close", "Cancel", "Keep editing · Escape")
                                    .when(self.close_choice == 0, |el| el.bg(rgb(t::SELECTED)))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.close_target = None;
                                        this.close_approved.clear();
                                        window.focus(&this.focus);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                text_button("discard-close", "Discard", "Close without saving")
                                    .when(self.close_choice == 1, |el| el.bg(rgb(t::SELECTED)))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        if matches!(this.close_target, Some(CloseTarget::Window)) {
                                            this.close_approved.insert(this.active_document);
                                        }
                                        this.finish_close(window, cx);
                                    })),
                            )
                            .child(
                                text_button("save-close", "Save", "Save and close · Enter")
                                    .when(self.close_choice == 2, |el| el.bg(rgb(t::SELECTED)))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.request_save(window, cx)
                                    })),
                            ),
                    );
                if let Some(error) = &self.error {
                    content = content.child(div().text_color(rgb(t::ERROR)).child(error.clone()));
                }
            }
            Some(
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .bg(rgba(0x282828b0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_, _, _, cx| cx.stop_propagation()),
                    )
                    .on_mouse_down(
                        MouseButton::Middle,
                        cx.listener(|_, _, _, cx| cx.stop_propagation()),
                    )
                    .child(content),
            )
        }
        fn save(&mut self, cx: &mut Context<Self>) {
            let mut path = PathBuf::from(self.path.read(cx).content.to_string());
            if path.extension().and_then(|x| x.to_str()) != Some("con") {
                path.set_extension("con");
            }
            match crate::persistence::container::save(&path, &self.design) {
                Ok(()) => {
                    self.forget_recovery(self.active_document);
                    self.saved_path = Some(path.clone());
                    self.saved_fingerprint = crate::document::dirty::fingerprint(&self.design);
                    self.dirty_cache.set((u64::MAX, false));
                    self.status = format!("Saved {}", path.display());
                    self.error = None;
                }
                Err(e) => self.error = Some(e),
            }
            cx.notify();
        }
        fn open(&mut self, cx: &mut Context<Self>) {
            let path = PathBuf::from(self.path.read(cx).content.to_string());
            let imported = !path
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x.eq_ignore_ascii_case("con"));
            match crate::exchange::import::load_design(&path) {
                Ok(d) => {
                    let opened_path = self.path.read(cx).content.to_string();
                    let previous_path = self
                        .saved_path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| self.documents[self.active_document].path.clone());
                    self.path = cx.new(|cx| TextInput::new(&previous_path, cx));
                    self.new_document(cx);
                    self.path = cx.new(|cx| TextInput::new(&opened_path, cx));
                    self.undo.clear();
                    self.redo.clear();
                    self.design = d;
                    self.saved_path = if imported { None } else { Some(path) };
                    self.saved_fingerprint = if imported {
                        crate::document::dirty::fingerprint(&Design::default())
                    } else {
                        crate::document::dirty::fingerprint(&self.design)
                    };
                    self.dirty_cache.set((u64::MAX, false));
                    self.inputs.clear();
                    self.solved.clear();
                    self.line = None;
                    self.anchor = None;
                    self.sketch = false;
                    self.mode = Mode::Solid;
                    self.construction_cursor = None;
                    self.before_construction = false;
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
            self.inspection_preview = false;
            if self.mode != Mode::Drawing
                && !self.hidden.contains("bodies")
                && let Some((vertices, indices)) = &self.mesh
            {
                let visible: Vec<u32> = indices
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .filter(|triangle| {
                        let face = vertices[triangle[0] as usize].face;
                        self.body_faces
                            .get(&face)
                            .is_none_or(|body| !self.hidden_bodies.contains(body))
                    })
                    .flatten()
                    .copied()
                    .collect();
                if self.panel == Some(Panel::Inspect)
                    && self.inspection_action == Action::SectionAnalysis
                {
                    let (clipped, triangles) = crate::ui::analysis::clipped_mesh(
                        vertices,
                        &visible,
                        self.section_axis,
                        self.section_position,
                    );
                    self.gpu.set_mesh(&clipped, &triangles);
                    self.inspection_preview = true;
                } else if let Some(tinted) = self.inspection_colors(vertices) {
                    self.gpu.set_mesh(&tinted, &visible);
                    self.inspection_preview = true;
                } else {
                    self.gpu.set_mesh(vertices, &visible);
                }
                return;
            }
            self.gpu.set_mesh(&[], &[]);
        }
        fn set_mode(&mut self, mode: Mode, cx: &mut Context<Self>) {
            self.mode = mode;
            self.sketch = mode == Mode::Sketch;
            if self.sketch {
                self.align_sketch_pending = true;
                self.align_to_sketch();
                self.align_sketch_pending = true;
            }
            self.tool = Tool::Select;
            self.anchor = None;
            self.menu = None;
            self.panel = None;
            self.construction_cursor = None;
            self.before_construction = false;
            self.changed_camera();
            self.refresh_mesh();
            self.rebuild(cx);
            if mode == Mode::Drawing {
                self.status = "Drawing tools are not implemented".into();
            }
        }
        fn feature(&mut self, feature: Feature, window: &mut Window, cx: &mut Context<Self>) {
            self.error = None;
            self.menu = None;
            window.focus(&self.focus);
            let Some(action) = feature.action else {
                self.error = Some(format!("{} is not implemented", feature.name));
                cx.notify();
                return;
            };
            match action {
                Action::ImportFusion => self.start_file_dialog(true, cx),
                Action::ExportStep => self.request_step_export(cx),
                Action::Export(format) => {
                    if self.export_dialog.is_none()
                        && self.step_export_dialog.is_none()
                        && self.export_result.is_none()
                        && self.native_dialog.is_none()
                    {
                        self.menu = None;
                        self.error = None;
                        self.export_dialog = Some((
                            format,
                            crate::platform::dialogs::select_export(
                                format,
                                self.saved_path
                                    .clone()
                                    .unwrap_or_else(|| PathBuf::from("Untitled.con")),
                            ),
                        ));
                    }
                }
                Action::SolidModify(kind) => self.open_solid_modify(kind, cx),
                Action::PhysicalMaterial | Action::Appearance | Action::ManageMaterials => {
                    self.open_materials(action, cx)
                }
                Action::Sketch => {
                    if self.selected == 0 && self.mesh.is_some() {
                        self.choosing_sketch_face = true;
                        self.set_mode(Mode::Solid, cx);
                        self.status =
                            "Select a planar face for the sketch, or choose XY plane".into();
                    } else {
                        self.create_sketch(cx);
                    }
                }
                Action::Line
                | Action::Rectangle
                | Action::Circle
                | Action::Circle2
                | Action::Circle3
                | Action::Arc3
                | Action::Ellipse
                | Action::Polygon
                | Action::Slot
                | Action::Spline
                | Action::CenterRectangle
                | Action::CenterArc
                | Action::Point
                | Action::Trim
                | Action::Extend
                | Action::Break => {
                    if self.mode != Mode::Sketch {
                        self.set_mode(Mode::Sketch, cx);
                    }
                    self.tool = match action {
                        Action::Line => Tool::Line,
                        Action::Rectangle => Tool::Rectangle,
                        Action::Circle => Tool::Circle,
                        Action::Circle2 => Tool::Circle2,
                        Action::Circle3 => Tool::Circle3,
                        Action::Arc3 => Tool::Arc3,
                        Action::Ellipse => Tool::Ellipse,
                        Action::Polygon => Tool::Polygon,
                        Action::Slot => Tool::Slot,
                        Action::Spline => Tool::Spline,
                        Action::CenterRectangle => Tool::CenterRectangle,
                        Action::CenterArc => Tool::Arc,
                        Action::Point => Tool::Point,
                        Action::Trim => Tool::Trim,
                        Action::Extend => Tool::Extend,
                        _ => Tool::Break,
                    };
                    self.arc_start = None;
                    self.tool_points.clear();
                    self.anchor = None;
                }
                Action::Select => {
                    self.tool = Tool::Select;
                    self.anchor = None;
                }
                Action::Parameters => self.panel = Some(Panel::Parameters),
                Action::Extrude => self.open_extrude(cx),
                Action::SolidCreate(kind) => self.open_create(kind, None, cx),
                Action::Dimension => {
                    self.panel = Some(Panel::Dimension);
                    self.tool = Tool::Dimension;
                    self.dimension_edit = None;
                    self.dimension_position = None;
                    self.anchor = None;
                }
                Action::Measure if self.mode == Mode::Solid => {
                    self.inspection_action = action;
                    self.tool = Tool::Select;
                    self.panel = Some(Panel::Inspect);
                }
                Action::SectionAnalysis
                | Action::Interference
                | Action::CenterOfMass
                | Action::CurvatureAnalysis
                | Action::DraftAnalysis
                | Action::ValidateSolid => {
                    self.inspection_action = action;
                    self.tool = Tool::Select;
                    if action == Action::SectionAnalysis
                        && let Some((vertices, _)) = &self.mesh
                    {
                        let min = vertices
                            .iter()
                            .map(|v| v.position[self.section_axis])
                            .fold(f32::INFINITY, f32::min);
                        let max = vertices
                            .iter()
                            .map(|v| v.position[self.section_axis])
                            .fold(f32::NEG_INFINITY, f32::max);
                        if min.is_finite() && max.is_finite() {
                            self.section_position = (min + max) * 0.5;
                            self.section_offset = cx.new(|cx| {
                                TextInput::new(&format!("{} mm", self.section_position * 40.), cx)
                            });
                        }
                    }

                    self.panel = Some(Panel::Inspect);
                    self.refresh_mesh();
                }
                Action::Measure => {
                    self.tool = Tool::Measure;
                    self.selection.clear();
                    self.panel = Some(Panel::Measure)
                }
                Action::Offset => self.panel = Some(Panel::Offset),
                Action::Move
                | Action::Mirror
                | Action::Scale
                | Action::RectangularPattern
                | Action::CircularPattern => {
                    self.transform_action = action;
                    self.panel = Some(Panel::Transform);
                    self.tool = Tool::Select;
                }
                Action::Fillet => self.panel = Some(Panel::Fillet),
                Action::Construction => {
                    let mut d = self.display_design();
                    crate::sketch::edit::toggle_construction(&mut d, &self.selection);
                    self.commit_sketch(d, cx);
                }
                Action::Delete => {
                    let mut d = self.display_design();
                    d.constraints.retain(|c| !self.selection.contains(&c.id));
                    d.driven_dimensions
                        .retain(|c| !self.selection.contains(&c.id));
                    crate::sketch::edit::delete(&mut d, &self.selection);
                    if self.commit_sketch(d, cx) {
                        self.selection.clear();
                        self.line = None;
                    }
                }
                Action::Coincident
                | Action::Parallel
                | Action::Perpendicular
                | Action::Equal
                | Action::Collinear
                | Action::Concentric
                | Action::Tangent
                | Action::Midpoint
                | Action::Symmetry => self.constrain_selection(action, cx),
                Action::Horizontal => self.apply_constraint(0, cx),
                Action::Vertical => self.apply_constraint(1, cx),
                Action::Fixed => self.apply_constraint(2, cx),
                Action::ViewFront
                | Action::ViewBack
                | Action::ViewLeft
                | Action::ViewRight
                | Action::ViewTop
                | Action::ViewBottom => {
                    use nalgebra::Vector3;
                    let (direction, up) = match action {
                        Action::ViewFront => (-Vector3::y(), Vector3::z()),
                        Action::ViewBack => (Vector3::y(), Vector3::z()),
                        Action::ViewLeft => (-Vector3::x(), Vector3::z()),
                        Action::ViewRight => (Vector3::x(), Vector3::z()),
                        Action::ViewTop => (Vector3::z(), Vector3::y()),
                        _ => (-Vector3::z(), -Vector3::y()),
                    };
                    self.camera.set_direction(direction, up);
                    self.changed_camera();
                }
                Action::Finish => {
                    self.set_mode(Mode::Solid, cx);
                    self.fit();
                }
            }
            if self.panel != Some(Panel::Dimension)
                && let Some(input) = self.panel_fields().first()
            {
                input.update(cx, |input, cx| input.focus_and_select(window, cx));
            }
            self.refresh_mesh();
            cx.notify();
        }
        fn restore_edit(&mut self, cx: &mut Context<Self>) {
            self.inputs.clear();
            self.solved.clear();
            self.line = None;
            self.anchor = None;
            self.construction_cursor = if self.sketch {
                self.design.current_sketch_id()
            } else {
                None
            };
            self.before_construction = false;
            self.editing_feature = None;
            self.selection.clear();
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
            if self.pending_candidate.is_some() {
                self.rebuild(cx);
                return;
            }
            if let Some(d) = self.undo.pop() {
                self.redo.push(self.design.clone());
                self.design = d;
                self.restore_edit(cx);
            }
        }
        fn redo_edit(&mut self, cx: &mut Context<Self>) {
            if self.pending_candidate.is_some() {
                self.rebuild(cx);
                return;
            }
            if let Some(d) = self.redo.pop() {
                self.undo.push(self.design.clone());
                self.design = d;
                self.restore_edit(cx);
            }
        }
        fn construction_step(&mut self, id: Option<Uuid>, edit: bool, cx: &mut Context<Self>) {
            if edit
                && let Some(modify) =
                    id.filter(|id| self.design.solid_edits.iter().any(|e| e.id == *id))
            {
                self.open_modify_edit(modify, cx);
                return;
            }
            if edit
                && let Some(f) = id
                    .and_then(|id| self.design.create_features.iter().find(|f| f.id == id))
                    .cloned()
            {
                self.open_create(f.kind, Some(f.id), cx);
                return;
            }
            self.construction_cursor = id;
            self.before_construction = false;
            self.menu = None;
            self.tool = Tool::Select;
            self.anchor = None;
            self.line = None;
            let kind = id
                .and_then(|id| self.design.construction.iter().find(|f| f.id == id))
                .map(|f| f.kind.clone());
            if edit || matches!(kind, Some(ConstructionKind::Sketch)) {
                let sketch = match kind {
                    Some(ConstructionKind::Sketch) => id,
                    Some(ConstructionKind::Extrude { sketch }) => Some(sketch),
                    _ => None,
                };
                if let Some(sketch) = sketch {
                    if let Err(e) = self.design.activate_sketch(sketch) {
                        self.error = Some(e);
                        cx.notify();
                        return;
                    }
                    self.solved.clear();
                    self.selection.clear();
                }
            }
            self.mode = if matches!(kind, Some(ConstructionKind::Sketch)) {
                Mode::Sketch
            } else {
                Mode::Solid
            };
            self.sketch = self.mode == Mode::Sketch;
            if self.sketch {
                self.align_to_sketch();
                self.align_sketch_pending = true;
            }
            self.panel = if edit && !self.sketch {
                Some(Panel::Extrude)
            } else {
                None
            };
            if edit && !self.sketch {
                self.open_extrude(cx);
                self.editing_feature = id;
                if let Some(feature) = self
                    .design
                    .solid_features()
                    .iter()
                    .find(|f| Some(f.id) == id)
                {
                    self.extrude_operation = feature.operation;
                    self.extrude_target = feature.target;
                    if let Some(p) = self
                        .design
                        .parameters
                        .iter()
                        .find(|p| p.id == feature.depth)
                    {
                        let text = p.expression.clone();
                        self.depth = cx.new(|cx| TextInput::new(&text, cx));
                    }
                }
            }
            self.rebuild(cx);
        }
        fn apply_parameters(&mut self, cx: &mut Context<Self>) {
            let mut candidate = self.display_design();
            for (id, input) in &self.inputs {
                if let Some(parameter) = candidate.parameters.iter_mut().find(|p| p.id == *id) {
                    parameter.expression = input.read(cx).content.to_string();
                }
            }
            self.commit_sketch(candidate, cx);
        }
        fn apply_offset_or_fillet(&mut self, fillet: bool, cx: &mut Context<Self>) {
            let mut candidate = self.display_design();
            let mut parameters = Design::default();
            let id = parameters.parameter("distance", self.dimension.read(cx).content.to_string());
            let result = crate::parameters::expression::evaluate(&parameters).and_then(|values| {
                if fillet {
                    crate::sketch::edit::fillet(&mut candidate, &self.selection, values[&id])
                } else {
                    crate::sketch::edit::offset(&mut candidate, &self.selection, values[&id])
                }
            });
            match result {
                Ok(()) => {
                    self.commit_sketch(candidate, cx);
                }
                Err(error) => self.error = Some(error),
            }
        }
        fn panel_fields(&self) -> Vec<Entity<TextInput>> {
            match self.panel {
                Some(Panel::SolidModify | Panel::Materials) => self.solid_editor.fields.clone(),
                Some(Panel::Inspect) if self.inspection_action == Action::SectionAnalysis => {
                    vec![self.section_offset.clone()]
                }
                Some(Panel::Extrude) => vec![self.depth.clone()],
                Some(Panel::Create) => self
                    .create_editor
                    .as_ref()
                    .map_or_else(Vec::new, |e| e.fields.clone()),
                Some(Panel::Dimension | Panel::Offset | Panel::Fillet) => {
                    vec![self.dimension.clone()]
                }
                Some(Panel::Search) => vec![self.search.clone()],
                Some(Panel::Parameters) => {
                    self.inputs.iter().map(|(_, input)| input.clone()).collect()
                }
                Some(Panel::Transform) => {
                    let mut fields = Vec::new();
                    if matches!(
                        self.transform_action,
                        Action::Move | Action::RectangularPattern
                    ) {
                        fields.extend([self.transform_x.clone(), self.transform_y.clone()]);
                    }
                    if matches!(
                        self.transform_action,
                        Action::Move | Action::CircularPattern
                    ) {
                        fields.push(self.transform_angle.clone());
                    }
                    if matches!(
                        self.transform_action,
                        Action::Scale | Action::RectangularPattern | Action::CircularPattern
                    ) {
                        fields.push(self.transform_count.clone());
                    }
                    if self.transform_action == Action::RectangularPattern {
                        fields.push(self.transform_rows.clone());
                    }
                    fields
                }
                _ => vec![],
            }
        }
        fn confirm_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
            self.error = None;
            match self.panel {
                Some(Panel::SolidModify) => self.apply_solid_modify(cx),
                Some(Panel::Materials) => self.apply_material(cx),
                Some(Panel::Inspect) if self.inspection_action == Action::SectionAnalysis => {
                    let mut input = Design::default();
                    let id = input.parameter(
                        "section_offset",
                        crate::parameters::expression::dimension_input(
                            &self.section_offset.read(cx).content,
                            false,
                        ),
                    );
                    match crate::parameters::expression::evaluate(&input) {
                        Ok(values)
                            if values[&id].is_finite()
                                && (values[&id] * 25.).abs() <= f32::MAX as f64 =>
                        {
                            self.section_position = (values[&id] * 25.) as f32;
                            self.changed_camera();
                            self.refresh_mesh();
                        }
                        Ok(_) => {
                            self.error = Some(
                                "Enter a finite section offset within the coordinate range".into(),
                            )
                        }
                        Err(error) => self.error = Some(error),
                    }
                    cx.notify();
                    return;
                }
                Some(Panel::Dimension) => self.apply_dimension(cx),
                Some(Panel::Create) => self.apply_create(cx),
                Some(Panel::Extrude) => {
                    let mut preview = Design {
                        parameters: self.design.parameters.clone(),
                        ..Design::default()
                    };
                    let expression = crate::parameters::expression::dimension_input(
                        &self.depth.read(cx).content,
                        false,
                    );
                    let id = preview.parameter("depth_preview", expression);
                    let result =
                        crate::parameters::expression::evaluate(&preview).and_then(|values| {
                            let depth = values[&id];
                            if depth.is_finite() && depth > 1e-7 && depth <= 1000. {
                                Ok(())
                            } else {
                                Err("Enter a positive depth within the modeling limits".into())
                            }
                        });
                    if let Err(error) = result {
                        self.error = Some(error);
                        cx.notify();
                        return;
                    }
                    self.extrude(cx);
                }
                Some(Panel::Transform) => self.apply_transform(cx),
                Some(Panel::Parameters) => self.apply_parameters(cx),
                Some(Panel::Offset) => self.apply_offset_or_fillet(false, cx),
                Some(Panel::Fillet) => self.apply_offset_or_fillet(true, cx),
                _ => return,
            }
            if self.error.is_none() && self.pending_candidate.is_none() {
                self.panel = None;
                window.focus(&self.focus);
            }
            cx.notify();
        }
        fn keyboard(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
            if self.native_dialog.is_some() {
                cx.stop_propagation();
                return;
            }
            if self.close_target.is_some() {
                if event.keystroke.key == "escape" {
                    self.close_target = None;
                    self.close_approved.clear();
                    cx.notify();
                }
                if event.keystroke.key == "tab" {
                    self.close_choice = (self.close_choice
                        + if event.keystroke.modifiers.shift {
                            2
                        } else {
                            1
                        })
                        % 3;
                    cx.notify();
                }
                if event.keystroke.key == "enter" {
                    match self.close_choice {
                        0 => {
                            self.close_target = None;
                            self.close_approved.clear();
                            window.focus(&self.focus);
                            cx.notify();
                        }
                        1 => {
                            if matches!(self.close_target, Some(CloseTarget::Window)) {
                                self.close_approved.insert(self.active_document);
                            }
                            self.finish_close(window, cx);
                        }
                        _ => self.request_save(window, cx),
                    }
                }
                cx.stop_propagation();
                return;
            }
            if self.panel == Some(Panel::Search) {
                let query = self.search.read(cx).content.to_lowercase();
                let matches: Vec<_> = toolbar::groups(self.mode)
                    .into_iter()
                    .flat_map(|group| group.features)
                    .filter(|feature| {
                        feature.available() && feature.name.to_lowercase().contains(&query)
                    })
                    .copied()
                    .collect();
                if query != self.search_query {
                    self.search_query = query;
                    self.search_index = 0;
                }
                match event.keystroke.key.as_str() {
                    "up" | "down" => {
                        if !matches.is_empty() {
                            self.search_index = if event.keystroke.key == "up" {
                                (self.search_index + matches.len() - 1) % matches.len()
                            } else {
                                (self.search_index + 1) % matches.len()
                            };
                        }
                        cx.stop_propagation();
                        cx.notify();
                        return;
                    }
                    "enter" => {
                        if let Some(feature) = matches.get(self.search_index).copied() {
                            self.panel = None;
                            self.feature(feature, window, cx);
                        }
                        cx.stop_propagation();
                        cx.notify();
                        return;
                    }
                    _ => {}
                }
            }
            if self.panel.is_some() {
                if event.keystroke.key == "escape" {
                    self.error = None;
                    self.panel = None;
                    self.dimension_edit = None;
                    window.focus(&self.focus);
                    cx.stop_propagation();
                    cx.notify();
                    return;
                }
                if event.keystroke.key == "tab" {
                    let fields = self.panel_fields();
                    if !fields.is_empty() {
                        let current = fields
                            .iter()
                            .position(|field| field.read(cx).focus_handle(cx).is_focused(window));
                        let next = match current {
                            Some(i) if event.keystroke.modifiers.shift => {
                                (i + fields.len() - 1) % fields.len()
                            }
                            Some(i) => (i + 1) % fields.len(),
                            None if event.keystroke.modifiers.shift => fields.len() - 1,
                            None => 0,
                        };
                        fields[next].update(cx, |input, cx| input.focus_and_select(window, cx));
                    }
                    cx.stop_propagation();
                    return;
                }
                if event.keystroke.key == "enter" {
                    self.confirm_editor(window, cx);
                    cx.stop_propagation();
                    cx.notify();
                    return;
                }
            }
            if event.keystroke.key == "enter"
                && self.tool == Tool::Spline
                && self.focus.is_focused(window)
            {
                let mut d = self.display_design();
                match crate::sketch::edit::spline(&mut d, &self.tool_points, true) {
                    Ok(_) => {
                        if self.commit_sketch(d, cx) {
                            self.tool_points.clear();
                        }
                    }
                    Err(e) => self.error = Some(e),
                }
                cx.notify();
                return;
            }
            let focused = self.focus.is_focused(window);
            if event.keystroke.key == "escape" && !focused {
                window.focus(&self.focus);
                return;
            }
            let modifiers = event.keystroke.modifiers;
            let mut chord = String::new();
            if modifiers.control || modifiers.platform {
                chord.push_str("ctrl-")
            }
            if modifiers.alt {
                chord.push_str("alt-")
            }
            if modifiers.shift {
                chord.push_str("shift-")
            }
            chord.push_str(&event.keystroke.key.to_lowercase());
            let command = self
                .keymap
                .resolve(&chord, self.sketch, !focused)
                .map(str::to_owned);
            let Some(command) = command else { return };
            match command.as_str() {
                "undo" => self.undo_edit(cx),
                "redo" => self.redo_edit(cx),
                "new" => self.new_document(cx),
                "save" => self.request_save(window, cx),
                "save-as" => self.start_file_dialog(false, cx),
                "open" => self.start_file_dialog(true, cx),
                "cancel" => {
                    self.finish_sketch_drag(true, cx);
                    self.anchor = None;
                    self.arc_start = None;
                    self.tool_points.clear();
                    self.choosing_sketch_face = false;
                    self.tool = Tool::Select;
                    self.selection.clear();
                    self.line = None;
                    self.menu = None;
                    self.panel = None;
                    self.dimension_edit = None;
                }
                "select-all" => {
                    let d = self.display_design();
                    self.selection = crate::sketch::entities::curve_ids(&d);
                    self.line = d.lines.first().map(|l| l.id);
                }
                "command-search" => self.panel = Some(Panel::Search),
                id => {
                    let feature = [Mode::Sketch, Mode::Solid]
                        .into_iter()
                        .flat_map(toolbar::groups)
                        .flat_map(|g| g.features)
                        .find(|f| f.id == id)
                        .copied();
                    if let Some(feature) = feature {
                        self.feature(feature, window, cx)
                    }
                }
            }
            cx.stop_propagation();
            cx.notify();
        }
    }

    impl WorkspaceView {
        fn top_bar(&self, cx: &mut Context<Self>) -> Div {
            let mut row = div()
                .h(px(50.))
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
                            this.start_file_dialog(this.file_open, cx);
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
                        cx.listener(|this, _, window, cx| {
                            if this.saved_path.is_some() {
                                this.request_save(window, cx);
                            } else {
                                this.file_open = false;
                                this.start_file_dialog(this.file_open, cx);
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
            let mut tabs = div()
                .id("document-tabs")
                .flex_1()
                .min_w_0()
                .h_full()
                .flex()
                .overflow_x_scroll();
            for (index, tab) in self.documents.iter().enumerate() {
                let name = if index == self.active_document {
                    self.document_name(cx)
                } else {
                    PathBuf::from(&tab.path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Untitled")
                        .to_owned()
                };
                tabs = tabs.child(
                    div()
                        .id(("document-tab", index))
                        .flex_none()
                        .h_full()
                        .px_3()
                        .flex()
                        .items_center()
                        .gap_2()
                        .bg(rgb(if index == self.active_document {
                            t::VIEWPORT
                        } else {
                            t::PANEL
                        }))
                        .cursor_pointer()
                        .child(name)
                        .when(
                            if index == self.active_document {
                                self.dirty()
                            } else {
                                tab.dirty
                            },
                            |el| el.child(div().text_color(rgb(t::WARNING)).child("•")),
                        )
                        .child(
                            button(
                                ("close-document", index),
                                "close",
                                "Close design",
                                false,
                                true,
                            )
                            .size(px(26.))
                            .on_click(cx.listener(
                                move |this, _, window, cx| {
                                    cx.stop_propagation();
                                    this.request_close_document(index, cx);
                                    window.focus(&this.focus);
                                },
                            )),
                        )
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.switch_document(index, cx);
                            window.focus(&this.focus);
                        })),
                );
            }
            row.child(tabs).child(
                button("add-document", "add", "New design", false, true)
                    .on_click(cx.listener(|this, _, _, cx| this.new_document(cx))),
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
                .h(px(86.))
                .flex_none()
                .flex()
                .items_center()
                .px_3()
                .gap_3()
                .bg(rgb(t::VIEWPORT))
                .border_b_1()
                .border_color(rgb(t::BORDER))
                .overflow_x_scroll();
            row = row.child(
                div()
                    .id("workspace-selector")
                    .h(px(64.))
                    .w(px(110.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .bg(rgb(t::PANEL))
                    .rounded_sm()
                    .cursor_pointer()
                    .child(self.mode.label())
                    .child(div().text_color(rgb(t::MUTED)).child("▾"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.menu_x = 12.;
                        this.menu = Some("Workspace");
                        cx.notify();
                    })),
            );
            for group in toolbar::groups(self.mode) {
                let name = group.name;
                let mut column = div().flex_none().flex().flex_col().gap_1();
                let mut tools = div().flex().gap_1();
                // Three discoverable primary icons per group; every remaining tool is in its menu.
                for &feature in group.features.iter().filter(|f| self.pinned.contains(f.id)) {
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
                        .h(px(23.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_1()
                        .cursor_pointer()
                        .rounded_sm()
                        .hover(|s| s.bg(rgb(t::HOVER)))
                        .child(name)
                        .child(div().text_color(rgb(t::MUTED)).child("▾"))
                        .on_hover(cx.listener(move |this, hovered: &bool, window, cx| {
                            if *hovered {
                                this.menu_leave_deadline = None;
                            } else if this.menu == Some(name) {
                                this.menu_leave_deadline = Some(
                                    std::time::Instant::now()
                                        + std::time::Duration::from_millis(150),
                                );
                            }
                            if *hovered && this.menu != Some(name) {
                                this.menu_x = (f64::from(window.mouse_position().x) as f32 - 48.)
                                    .clamp(
                                        8.,
                                        f64::from(window.bounds().size.width).max(352.) as f32
                                            - 336.,
                                    );
                                this.menu = Some(name);
                                cx.notify();
                            }
                        }))
                        .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                            this.menu_x = (f64::from(event.position().x) as f32 - 48.).clamp(
                                8.,
                                f64::from(window.bounds().size.width).max(352.) as f32 - 336.,
                            );
                            this.menu = Some(name);
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
                .w(px(260.))
                .flex_none()
                .flex()
                .flex_col()
                .absolute()
                .top(px(16.))
                .left(px(16.))
                .max_h(px(self.bounds.get().map_or(500., |b| {
                    (f64::from(b.size.height) as f32 - 32.).max(120.)
                })))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|_, _, _, cx| cx.stop_propagation()),
                )
                .on_mouse_down(
                    MouseButton::Middle,
                    cx.listener(|_, _, _, cx| cx.stop_propagation()),
                )
                .border_color(rgb(t::BORDER));
            tree = tree.child(
                div()
                    .h(px(37.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .px_3()
                    .gap_2()
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
                    .id(SharedString::from(format!("branch-{key}")))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(t::HOVER)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if !this.expanded.remove(key) {
                            this.expanded.insert(key);
                        }
                        cx.notify();
                    }))
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .px_2()
                    .gap_2()
                    .child(
                        div()
                            .w(px(14.))
                            .text_color(rgb(t::MUTED))
                            .child(if expanded { "▾" } else { "▸" }),
                    )
                    .child(icon(image, 16., t::MUTED))
                    .child(div().flex_1().child(label))
                    .child(
                        text_button(
                            SharedString::from(format!("visibility-{label}")),
                            if hidden { "○" } else { "●" },
                            &format!("{} {label}", if hidden { "Show" } else { "Hide" }),
                        )
                        .size(px(24.))
                        .bg(rgba(0))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.stop_propagation();
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
                                        self.mode == Mode::Sketch
                                            && self.design.current_sketch_id() == Some(id),
                                    )
                                    .child(div().flex_1())
                                    .child(
                                        text_button(
                                            SharedString::from(format!("sketch-visible-{id}")),
                                            if self.hidden_sketches.contains(&id) {
                                                "○"
                                            } else {
                                                "●"
                                            },
                                            "Toggle sketch visibility",
                                        )
                                        .size(px(24.))
                                        .bg(rgba(0))
                                        .on_click(
                                            cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                if !this.hidden_sketches.remove(&id) {
                                                    this.hidden_sketches.insert(id);
                                                }
                                                cx.notify();
                                            }),
                                        ),
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
                            for (id, name) in self.design.current_create_bodies() {
                                rows = rows.child(
                                    tree_item(
                                        SharedString::from(format!("body-{id}")),
                                        "create_block",
                                        &name,
                                        self.construction_cursor == Some(id),
                                    )
                                    .child(div().flex_1())
                                    .child(
                                        text_button(
                                            SharedString::from(format!("body-visible-{id}")),
                                            if self.hidden_bodies.contains(&(id, 0)) {
                                                "○"
                                            } else {
                                                "●"
                                            },
                                            "Toggle body visibility",
                                        )
                                        .size(px(24.))
                                        .bg(rgba(0))
                                        .on_click(
                                            cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                if !this.hidden_bodies.remove(&(id, 0)) {
                                                    this.hidden_bodies.insert((id, 0));
                                                }
                                                this.refresh_mesh();
                                                this.changed_camera();
                                                cx.notify();
                                            }),
                                        ),
                                    )
                                    .on_click(cx.listener(
                                        move |this, _, _, cx| {
                                            this.construction_step(Some(id), true, cx)
                                        },
                                    )),
                                );
                            }
                            for e in self
                                .design
                                .solid_features()
                                .iter()
                                .filter(|e| e.operation == ExtrudeOperation::CutNewBody)
                            {
                                let id = e.id;
                                rows = rows.child(
                                    tree_item(
                                        SharedString::from(format!("cut-body-{id}")),
                                        "create_block",
                                        &format!("{} · cut body", e.name),
                                        false,
                                    )
                                    .child(div().flex_1())
                                    .child(
                                        text_button(
                                            SharedString::from(format!("cut-visible-{id}")),
                                            if self.hidden_bodies.contains(&(id, 1)) {
                                                "○"
                                            } else {
                                                "●"
                                            },
                                            "Toggle cut body visibility",
                                        )
                                        .size(px(24.))
                                        .bg(rgba(0))
                                        .on_click(
                                            cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                if !this.hidden_bodies.remove(&(id, 1)) {
                                                    this.hidden_bodies.insert((id, 1));
                                                }
                                                this.refresh_mesh();
                                                this.changed_camera();
                                                cx.notify();
                                            }),
                                        ),
                                    ),
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
            if panel == Panel::Dimension
                && self.dimension_position.is_none()
                && self.dimension_edit.is_none()
            {
                return None;
            }
            let mut body = div()
                .w(px(270.))
                .flex_none()
                .flex()
                .flex_col()
                .bg(rgb(t::PANEL))
                .border_1()
                .rounded_md()
                .border_color(rgb(t::BORDER))
                .absolute()
                .top(px(72.))
                .right(px(100.))
                .max_h(px(self.bounds.get().map_or(500., |b| {
                    (f64::from(b.size.height) as f32 - 100.).max(160.)
                })))
                .occlude()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|_, _, _, cx| cx.stop_propagation()),
                )
                .on_mouse_down(
                    MouseButton::Middle,
                    cx.listener(|_, _, _, cx| cx.stop_propagation()),
                );
            if panel == Panel::Dimension {
                let at = self
                    .dimension_position
                    .or_else(|| {
                        self.dimension_edit.and_then(|id| {
                            let d = self.display_design();
                            d.constraints
                                .iter()
                                .chain(&d.driven_dimensions)
                                .find(|c| c.id == id)
                                .map(|c| self.dimension_anchor(&d, c))
                        })
                    })
                    .or(self.hover);
                if let (Some(at), Some(bounds)) = (at, self.bounds.get()) {
                    let width = f64::from(bounds.size.width) as f32;
                    let height = f64::from(bounds.size.height) as f32;
                    let screen = self
                        .sketch_screen(at)
                        .unwrap_or([width as f64 * 0.5, height as f64 * 0.5]);
                    let x = screen[0] as f32 + 16.;
                    let y = screen[1] as f32 + 16.;
                    body = body
                        .right(Length::Auto)
                        .left(px(x.clamp(8., (width - 198.).max(8.))))
                        .top(px(y.clamp(8., (height - 112.).max(8.))));
                }
                let mut editor = body
                    .w(px(190.))
                    .p_2()
                    .gap_1()
                    .child(self.dimension.clone())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                text_button(
                                    "apply-dimension",
                                    "Apply ↵",
                                    "Confirm dimension · Enter",
                                )
                                .h(px(24.))
                                .px_1()
                                .on_click(cx.listener(
                                    |this, _, window, cx| this.confirm_editor(window, cx),
                                )),
                            )
                            .child(
                                text_button(
                                    "reference-dimension",
                                    if self.reference_dimension {
                                        "Driven"
                                    } else {
                                        "Driving"
                                    },
                                    "Toggle driven reference dimension",
                                )
                                .h(px(24.))
                                .px_1()
                                .text_color(rgb(t::MUTED))
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.reference_dimension = !this.reference_dimension;
                                        cx.notify();
                                    },
                                )),
                            ),
                    );
                if let Some(error) = &self.error {
                    editor = editor.child(
                        div()
                            .text_size(px(11.))
                            .text_color(rgb(t::ERROR))
                            .child(error.clone()),
                    );
                }
                return Some(editor);
            }
            let title = match panel {
                Panel::Parameters => "Parameters",
                Panel::Extrude => "Extrude",
                Panel::Create => self
                    .create_editor
                    .as_ref()
                    .map_or("Create", |e| e.kind.name()),
                Panel::Dimension => "Dimension",
                Panel::View => "View",
                Panel::Measure => "Measure",
                Panel::Inspect => self.inspection_title(),
                Panel::Offset => "Offset",
                Panel::Search => "Sketch toolbox",
                Panel::Transform => match self.transform_action {
                    Action::Scale => "Scale",
                    Action::Mirror => "Mirror",
                    Action::RectangularPattern => "Rectangular pattern",
                    Action::CircularPattern => "Circular pattern",
                    _ => "Move / copy",
                },
                Panel::Fillet => "Sketch fillet",
                Panel::SolidModify => self.solid_editor.kind.label(),
                Panel::Materials => match self.solid_editor.material_action {
                    Action::Appearance => "Appearance",
                    Action::ManageMaterials => "Manage materials",
                    _ => "Physical material",
                },
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
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.error = None;
                                this.panel = None;
                                this.refresh_mesh();
                                window.focus(&this.focus);
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
                Panel::Create => {
                    content = content.child(self.create_controls(cx));
                }
                Panel::SolidModify | Panel::Materials => {
                    content = self.solid_modify_content(content, cx);
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
                                                let mut candidate = this.display_design();
                                                let expression = input.read(cx).content.to_string();
                                                if let Some(p) = candidate
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
                                                this.commit_sketch(candidate, cx);
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
                    let mut operations = div().flex().flex_wrap().gap_1();
                    for (operation, label) in [
                        (ExtrudeOperation::NewBody, "New body"),
                        (ExtrudeOperation::Join, "Join"),
                        (ExtrudeOperation::Cut, "Cut"),
                        (ExtrudeOperation::CutNewBody, "Cut and new body"),
                    ] {
                        operations = operations.child(
                            div()
                                .id(SharedString::from(format!("operation-{label}")))
                                .px_2()
                                .py_1()
                                .rounded_sm()
                                .cursor_pointer()
                                .bg(rgb(if self.extrude_operation == operation {
                                    t::SELECTED
                                } else {
                                    t::PANEL
                                }))
                                .hover(|s| s.bg(rgb(t::HOVER)))
                                .child(label)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.extrude_operation = operation;
                                    cx.notify();
                                })),
                        );
                    }
                    content = content.child(operations);
                    if self.extrude_operation != ExtrudeOperation::NewBody {
                        content =
                            content.child(div().text_color(rgb(t::MUTED)).child("Target body"));
                        let features = self.design.solid_features();
                        for f in &features {
                            if Some(f.id) == self.editing_feature
                                || features.iter().any(|other| {
                                    other.target == Some(f.id)
                                        && Some(other.id) != self.editing_feature
                                })
                            {
                                continue;
                            }
                            let id = f.id;
                            content = content.child(
                                div()
                                    .id(SharedString::from(format!("target-{id}")))
                                    .px_2()
                                    .py_1()
                                    .cursor_pointer()
                                    .bg(rgb(if self.extrude_target == Some(id) {
                                        t::SELECTED
                                    } else {
                                        t::PANEL
                                    }))
                                    .child(f.name.clone())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.extrude_target = Some(id);
                                        cx.notify();
                                    })),
                            );
                        }
                    }
                    content = content
                        .child(div().text_color(rgb(t::MUTED)).child("Depth"))
                        .child(self.depth.clone())
                        .child(
                            div().flex().justify_end().child(
                                text_button(
                                    "apply-extrude",
                                    "Extrude",
                                    "Confirm extrusion · Enter",
                                )
                                .on_click(cx.listener(
                                    |this, _, window, cx| this.confirm_editor(window, cx),
                                )),
                            ),
                        )
                        .child(div().text_size(px(11.)).text_color(rgb(t::MUTED)).child(
                            if matches!(
                                self.extrude_operation,
                                ExtrudeOperation::Cut | ExtrudeOperation::CutNewBody
                            ) {
                                "Cut inward from sketch plane"
                            } else {
                                "Extrude outward from sketch plane"
                            },
                        ));
                }
                Panel::Transform => {
                    let action = self.transform_action;
                    if matches!(action, Action::Move | Action::RectangularPattern) {
                        content = content
                            .child(if action == Action::Move {
                                "X distance"
                            } else {
                                "X spacing"
                            })
                            .child(self.transform_x.clone())
                            .child(if action == Action::Move {
                                "Y distance"
                            } else {
                                "Y spacing"
                            })
                            .child(self.transform_y.clone());
                    }
                    if matches!(action, Action::Move | Action::CircularPattern) {
                        content = content
                            .child(if action == Action::Move {
                                "Rotation about sketch origin"
                            } else {
                                "Sweep angle (0 = full circle)"
                            })
                            .child(self.transform_angle.clone());
                    }
                    if matches!(
                        action,
                        Action::Scale | Action::RectangularPattern | Action::CircularPattern
                    ) {
                        content = content
                            .child(if action == Action::Scale {
                                "Scale factor"
                            } else if action == Action::RectangularPattern {
                                "X quantity"
                            } else {
                                "Quantity"
                            })
                            .child(self.transform_count.clone());
                    }
                    if action == Action::RectangularPattern {
                        content = content
                            .child("Y quantity")
                            .child(self.transform_rows.clone());
                    }
                    if action == Action::Mirror {
                        content = content.child("Select objects, then Shift-click the mirror line");
                    }
                    if matches!(action, Action::Scale | Action::CircularPattern) {
                        content = content.child(
                            "Shift-click a point last to set the center. Default: sketch origin.",
                        );
                    }
                    if matches!(action, Action::Move | Action::Scale) {
                        content = content.child(
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    button(
                                        "copy-sketch",
                                        "copy",
                                        "Create a copy",
                                        self.transform_copy,
                                        true,
                                    )
                                    .on_click(cx.listener(
                                        |this, _, _, cx| {
                                            this.transform_copy = !this.transform_copy;
                                            cx.notify()
                                        },
                                    )),
                                )
                                .child("Create copy"),
                        );
                    }
                    content = content.child(
                        text_button("apply-transform", "Apply", "Confirm transform · Enter")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.confirm_editor(window, cx)),
                            ),
                    );
                }
                Panel::Fillet => {
                    content = content
                        .child("Select two connected straight lines")
                        .child("Radius")
                        .child(self.dimension.clone())
                        .child(
                            text_button("apply-fillet", "Apply fillet", "Confirm fillet · Enter")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.confirm_editor(window, cx)
                                })),
                        );
                }
                Panel::Search => {
                    content = content.child(self.search.clone());
                    let query = self.search.read(cx).content.to_lowercase();
                    let selected = if query == self.search_query {
                        self.search_index
                    } else {
                        0
                    };
                    let matches: Vec<_> = toolbar::groups(self.mode)
                        .into_iter()
                        .flat_map(|group| group.features)
                        .filter(|feature| {
                            feature.available() && feature.name.to_lowercase().contains(&query)
                        })
                        .copied()
                        .collect();
                    for (index, feature) in matches.iter().copied().enumerate() {
                        content = content.child(
                            div()
                                .id(feature.id)
                                .px_2()
                                .py_2()
                                .cursor_pointer()
                                .when(index == selected, |el| el.bg(rgb(t::SELECTED)))
                                .hover(|style| style.bg(rgb(t::HOVER)))
                                .child(feature.name)
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.panel = None;
                                    this.feature(feature, window, cx);
                                })),
                        );
                    }
                    if matches.is_empty() {
                        content = content
                            .child(div().text_color(rgb(t::MUTED)).child("No matching tools"));
                    }
                }
                Panel::Inspect => {
                    for line in self.inspection_lines() {
                        content = content.child(line);
                    }
                    if self.inspection_action == Action::SectionAnalysis {
                        let mut axes = div().flex().gap_2();
                        for (axis, label) in ["X", "Y", "Z"].into_iter().enumerate() {
                            axes = axes.child(
                                text_button(
                                    SharedString::from(format!("section-axis-{axis}")),
                                    label,
                                    "Section plane normal",
                                )
                                .when(self.section_axis == axis, |el| el.bg(rgb(t::SELECTED)))
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.section_axis = axis;
                                        this.changed_camera();
                                        this.refresh_mesh();
                                        cx.notify();
                                    },
                                )),
                            );
                        }
                        content = content
                            .child(axes)
                            .child("Offset from origin")
                            .child(self.section_offset.clone())
                            .child(
                                text_button(
                                    "apply-section",
                                    "Update section",
                                    "Apply plane offset · Enter",
                                )
                                .on_click(cx.listener(
                                    |this, _, window, cx| this.confirm_editor(window, cx),
                                )),
                            );
                    }
                }
                Panel::Measure => {
                    content = content
                        .child(crate::sketch::dimensions::measure(
                            &self.display_design(),
                            &self.selection,
                        ))
                        .child(
                            div()
                                .text_color(rgb(t::MUTED))
                                .child("Shift-click to measure between two selections"),
                        );
                }
                Panel::Offset => {
                    content = content
                        .child("Offset distance")
                        .child(self.dimension.clone())
                        .child(
                            text_button("apply-offset", "Apply offset", "Confirm offset · Enter")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.confirm_editor(window, cx)
                                })),
                        );
                }
                Panel::Dimension => {}
                Panel::View => {
                    for (id, image, label, enabled) in [
                        ("view-grid", "grid", "Grid", self.grid),
                        ("view-snap", "snap_grid", "Snap to grid", self.snap),
                        (
                            "view-constraints",
                            "properties",
                            "Constraints",
                            self.show_constraints,
                        ),
                        (
                            "view-dimensions",
                            "dim_linear",
                            "Dimensions",
                            self.show_dimensions,
                        ),
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
                                        } else if id == "view-snap" {
                                            this.snap = !this.snap;
                                        } else if id == "view-constraints" {
                                            this.show_constraints = !this.show_constraints;
                                        } else {
                                            this.show_dimensions = !this.show_dimensions;
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
            if (panel == Panel::Parameters && self.line.is_some())
                && let Some(line) = self.line
            {
                let ends = self
                    .design
                    .lines
                    .iter()
                    .find(|l| l.id == line)
                    .map(|l| l.ends)
                    .unwrap_or_default();
                for c in &self.design.constraints {
                    let label = c
                        .kind
                        .references()
                        .iter()
                        .any(|id| *id == line || ends.contains(id) || self.selection.contains(id))
                        .then_some(c.kind.label());
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
                                    .on_click(cx.listener(
                                        move |this, _, window, cx| {
                                            this.checkpoint();
                                            this.design.constraints.retain(|c| c.id != id);
                                            this.rebuild(cx);
                                            window.focus(&this.focus);
                                        },
                                    )),
                                ),
                        )
                    }
                }
            }
            if let Some(error) = &self.error {
                content = content.child(
                    div()
                        .text_size(px(12.))
                        .text_color(rgb(t::ERROR))
                        .child(error.clone()),
                );
            }
            Some(
                body.child(content).child(
                    div()
                        .px_3()
                        .pb_2()
                        .text_size(px(11.))
                        .text_color(rgb(t::MUTED))
                        .child(if panel == Panel::Search {
                            "↑ ↓: choose tool · Enter: run · Esc: close"
                        } else {
                            "Tab: next field · Enter: confirm · Esc: cancel"
                        }),
                ),
            )
        }
        fn cube_camera(&self) -> Camera {
            self.camera.clone()
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
                    .text_size(px(11.))
                    .text_color(rgb(t::MUTED))
                    .text_right()
                    .child(format!(
                        "{:.0} FPS · {:.1} ms",
                        self.frame_rate.fps, self.frame_rate.milliseconds
                    )),
            );
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
                        MouseButton::Right,
                        cx.listener(|this, event: &MouseDownEvent, window, cx| {
                            this.menu_x = (f64::from(event.position.x) as f32 - 160.).clamp(
                                8.,
                                f64::from(window.bounds().size.width).max(352.) as f32 - 336.,
                            );
                            this.menu = Some("Orient");
                            cx.stop_propagation();
                            cx.notify();
                        }),
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
                                    this.camera.set_direction(face.direction, face.up);
                                    this.changed_camera();
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
        fn timeline_at(&mut self, x: Pixels, cx: &mut Context<Self>) {
            let Some(bounds) = self.timeline_bounds.get() else {
                return;
            };
            let index = ((f64::from(x - bounds.origin.x) - 16.) / 48.)
                .round()
                .max(0.) as usize;
            let index = index.min(self.design.construction.len());
            let id = if index == 0 {
                None
            } else {
                self.design.construction.get(index - 1).map(|f| f.id)
            };
            self.construction_step(id, false, cx);
            if index == 0 {
                self.before_construction = true;
                self.rebuild(cx);
            }
        }
        fn timeline(&self, cx: &mut Context<Self>) -> Stateful<Div> {
            let capture = self.timeline_bounds.clone();
            let marker = if self.before_construction {
                0
            } else {
                self.construction_cursor
                    .and_then(|id| self.design.construction.iter().position(|f| f.id == id))
                    .map_or(self.design.construction.len(), |i| i + 1)
            };
            let mut row = div()
                .id("construction-timeline")
                .relative()
                .h(px(58.))
                .flex_none()
                .flex()
                .items_center()
                .gap_2()
                .px(px(24.))
                .bg(rgb(t::PANEL))
                .border_t_1()
                .border_color(rgb(t::BORDER))
                .overflow_x_scroll();
            for (index, feature) in self.design.construction.iter().enumerate() {
                let id = feature.id;
                row = row.child(
                    button(
                        SharedString::from(format!("feature-{id}")),
                        match feature.kind {
                            ConstructionKind::Sketch => "line_rectangle",
                            ConstructionKind::Extrude { .. } => "up",
                            ConstructionKind::Create | ConstructionKind::Modify => "up",
                        },
                        &format!("{} · Edit feature", feature.name),
                        self.construction_cursor == Some(id),
                        true,
                    )
                    .opacity(if index >= marker { 0.35 } else { 1. })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.construction_step(Some(id), true, cx);
                        window.focus(&this.focus);
                    })),
                );
            }
            row.child(
                canvas(
                    move |bounds, _, _| capture.set(Some(bounds)),
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            )
            .child(
                div()
                    .id("construction-marker")
                    .absolute()
                    .left(px(16. + marker as f32 * 48.))
                    .top(px(6.))
                    .w(px(8.))
                    .h(px(46.))
                    .bg(rgb(t::ACCENT))
                    .rounded_sm()
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.timeline_drag = true;
                            cx.stop_propagation();
                        }),
                    ),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if this.timeline_drag && event.pressed_button == Some(MouseButton::Left) {
                    this.timeline_at(event.position.x, cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.timeline_drag = false),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.timeline_drag = false),
            )
        }
        fn menu_overlay(&self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
            let name = self.menu?;
            if name == "Workspace" {
                let mut menu = div()
                    .id("workspace-menu")
                    .absolute()
                    .top(px(136.))
                    .left(px(12.))
                    .w(px(190.))
                    .bg(rgb(t::PANEL))
                    .border_1()
                    .border_color(rgb(t::BORDER))
                    .occlude()
                    .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                        this.menu = None;
                        cx.notify();
                    }));
                for mode in [Mode::Solid, Mode::Sketch, Mode::Drawing] {
                    menu = menu.child(
                        div()
                            .id(SharedString::from(format!("workspace-{}", mode.label())))
                            .h(px(42.))
                            .px_3()
                            .flex()
                            .items_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(t::HOVER)))
                            .child(mode.label())
                            .on_click(cx.listener(move |this, _, _, cx| this.set_mode(mode, cx))),
                    );
                }
                return Some(menu);
            }
            let features: Vec<Feature> = if name == "Orient" {
                [
                    ("Front", Action::ViewFront),
                    ("Back", Action::ViewBack),
                    ("Left", Action::ViewLeft),
                    ("Right", Action::ViewRight),
                    ("Top", Action::ViewTop),
                    ("Bottom", Action::ViewBottom),
                ]
                .into_iter()
                .map(|(label, action)| Feature {
                    id: label,
                    name: label,
                    icon: "camera",
                    action: Some(action),
                })
                .collect()
            } else if name == "Export" {
                vec![
                    Feature {
                        id: "export-step",
                        name: "STEP",
                        icon: "export",
                        action: Some(Action::ExportStep),
                    },
                    Feature {
                        id: "export-stl",
                        name: "STL (millimetres)",
                        icon: "export",
                        action: Some(Action::Export(crate::exchange::export::ExportFormat::Stl)),
                    },
                    Feature {
                        id: "export-3mf",
                        name: "3MF",
                        icon: "export",
                        action: Some(Action::Export(
                            crate::exchange::export::ExportFormat::ThreeMf,
                        )),
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
                .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                    this.menu_leave_deadline = if *hovered {
                        None
                    } else {
                        Some(std::time::Instant::now() + std::time::Duration::from_millis(150))
                    };
                    cx.notify();
                }))
                .absolute()
                .top(px(136.))
                .left(px(self.menu_x))
                .w(px(390.))
                .max_h(px(self.bounds.get().map_or(440., |b| {
                    f64::from(b.size.height).clamp(120., 440.) as f32
                })))
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
                        .h(px(40.))
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
                        .child(
                            button(
                                SharedString::from(format!("pin-{}", feature.id)),
                                if self.pinned.contains(feature.id) {
                                    "locked"
                                } else {
                                    "unlocked"
                                },
                                if self.pinned.contains(feature.id) {
                                    "Unpin from toolbar"
                                } else {
                                    "Pin to toolbar"
                                },
                                self.pinned.contains(feature.id),
                                true,
                            )
                            .size(px(32.))
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    cx.stop_propagation();
                                    if !this.pinned.remove(feature.id) {
                                        this.pinned.insert(feature.id);
                                    }
                                    if let Err(error) =
                                        crate::settings::store::save_toolbar_pins(&this.pinned)
                                    {
                                        this.error = Some(error);
                                    }
                                    cx.notify();
                                },
                            )),
                        )
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
            let d = self.display_design();
            let color = |id: Uuid| {
                let ids = crate::sketch::entities::curve_points(&d, id);
                let conflict = d.constraints.iter().any(|c| {
                    self.conflicts.contains(&c.id)
                        && c.kind
                            .references()
                            .iter()
                            .any(|r| *r == id || ids.contains(r))
                });
                let fixed = ids.iter().all(|p| {
                    d.constraints
                        .iter()
                        .any(|c| matches!(c.kind,ConstraintKind::Fixed{point,..} if point==*p))
                });
                let fully = ids.iter().all(|id| {
                    d.points
                        .iter()
                        .position(|p| p.id == *id)
                        .is_some_and(|i| self.point_dof.get(i) == Some(&0))
                });
                if conflict {
                    t::ERROR
                } else if fixed {
                    t::SUCCESS
                } else if fully {
                    t::TEXT
                } else {
                    t::ACCENT
                }
            };
            let lines = crate::sketch::entities::curve_ids(&d)
                .into_iter()
                .map(|id| {
                    (
                        id,
                        crate::sketch::entities::samples(&d, id),
                        color(id),
                        d.construction_geometry.contains(&id),
                    )
                })
                .collect();
            let point_colors = d
                .points
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    if self.selection.contains(&p.id) {
                        t::WARNING
                    } else if d.constraints.iter().any(
                        |c| matches!(c.kind, ConstraintKind::Fixed { point, .. } if point == p.id),
                    ) {
                        t::SUCCESS
                    } else if self.point_dof.get(i) == Some(&0) {
                        t::TEXT
                    } else {
                        t::ACCENT
                    }
                })
                .collect();
            let mut annotations = Vec::new();
            let mut dimensions = Vec::new();
            for c in d.constraints.iter().chain(&d.driven_dimensions) {
                let at = self.dimension_anchor(&d, c);
                if c.kind.parameter().is_some() {
                    let measured = crate::sketch::dimensions::value(&d, &c.kind).unwrap_or(0.);
                    let angular = matches!(c.kind, ConstraintKind::Angle { .. });
                    let driven = d.driven_dimensions.iter().any(|v| v.id == c.id);
                    let prefix = match c.kind {
                        ConstraintKind::Diameter { .. } => "Ø",
                        ConstraintKind::Radius { .. } => "R",
                        _ => "",
                    };
                    let text = if angular {
                        format!("{:.2}°", measured.to_degrees())
                    } else {
                        format!("{prefix}{:.3}", measured * 1000.)
                    };
                    annotations.push((at, if driven { format!("({text})") } else { text }));
                    let ends = match c.kind {
                        ConstraintKind::Length { line, .. } => {
                            d.lines.iter().find(|l| l.id == line).map(|l| l.ends)
                        }
                        ConstraintKind::Distance { points, .. }
                        | ConstraintKind::DistanceX { points, .. }
                        | ConstraintKind::DistanceY { points, .. }
                        | ConstraintKind::ProjectedDistance { points, .. } => Some(points),
                        ConstraintKind::Diameter { circle, .. }
                        | ConstraintKind::Radius { circle, .. } => d
                            .circles
                            .iter()
                            .find(|c| c.id == circle)
                            .map(|c| [c.center, c.rim]),
                        _ => None,
                    };
                    if let Some(ends) = ends {
                        let direction = match c.kind {
                            ConstraintKind::DistanceX { .. } => Some([1., 0.]),
                            ConstraintKind::DistanceY { .. } => Some([0., 1.]),
                            ConstraintKind::ProjectedDistance { direction, .. } => Some(direction),
                            _ => None,
                        };
                        dimensions.push((
                            ends.map(|id| crate::sketch::entities::point(&d, id)),
                            at,
                            direction,
                            matches!(
                                c.kind,
                                ConstraintKind::Diameter { .. } | ConstraintKind::Radius { .. }
                            ),
                        ))
                    }
                }
            }
            if !self.show_dimensions {
                annotations.clear();
                dimensions.clear();
            }
            let mut constraint_markers = Vec::new();
            if self.sketch && self.show_constraints && self.bounds.get().is_some() {
                let mut placed: Vec<[f64; 2]> = Vec::new();
                for c in &d.constraints {
                    if c.kind.parameter().is_some() {
                        continue;
                    }
                    let Some(first) = c.kind.references().first().copied() else {
                        continue;
                    };
                    let at = crate::sketch::entities::position(&d, first);
                    let offset = placed
                        .iter()
                        .filter(|p| (p[0] - at[0]).hypot(p[1] - at[1]) * self.scale < 20.)
                        .count() as f32;
                    placed.push(at);
                    let id = c.id;
                    let label = c.kind.label();
                    let [x, y] = self.sketch_screen(at).unwrap_or([-10000., -10000.]);
                    constraint_markers.push(
                        crate::ui::components::constraint_button(
                            SharedString::from(format!("constraint-{id}")),
                            c.kind.icon(),
                            label,
                            self.selection.contains(&id),
                        )
                        .absolute()
                        .left(px(x as f32 + 7. + offset * 18.))
                        .top(px(y as f32 - 20.))
                        .size(px(16.))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|_, _, _, cx| cx.stop_propagation()),
                        )
                        .on_click(cx.listener(
                            move |this, _, window, cx| {
                                this.selection = vec![id];
                                this.line = None;
                                this.tool = Tool::Select;
                                window.focus(&this.focus);
                                cx.notify();
                            },
                        )),
                    );
                }
            }
            let mut preview = self.tool_points.clone();
            if let Some((a, b, _)) = self.marquee {
                preview = vec![a, [b[0], a[1]], b, [a[0], b[1]], a];
            }
            if !preview.is_empty()
                && let Some(p) = self.hover
            {
                preview.push(p)
            }
            if let (Some(a), Some(b)) = (self.anchor, self.hover)
                && (self.tool == Tool::Circle || self.tool == Tool::Arc)
            {
                let start = self.arc_start.unwrap_or(b);
                let radius = (start[0] - a[0]).hypot(start[1] - a[1]);
                let angle = (start[1] - a[1]).atan2(start[0] - a[0]);
                let sweep = if self.arc_start.is_some() {
                    ((b[1] - a[1]).atan2(b[0] - a[0]) - angle).rem_euclid(std::f64::consts::TAU)
                } else {
                    std::f64::consts::TAU
                };
                preview = (0..=96)
                    .map(|i| {
                        let t = angle + sweep * i as f64 / 96.;
                        [a[0] + radius * t.cos(), a[1] + radius * t.sin()]
                    })
                    .collect();
            }
            let mut preview_curves = Vec::new();
            if let Some(hover) = self.hover {
                let mut points = self.tool_points.clone();
                points.push(hover);
                let mut draft = Design::default();
                match (self.tool, points.as_slice()) {
                    (Tool::CenterRectangle, [a, b]) => {
                        let c = [2. * a[0] - b[0], 2. * a[1] - b[1]];
                        preview = vec![c, [b[0], c[1]], *b, [c[0], b[1]], c];
                    }
                    (Tool::Circle2, [a, b]) => {
                        let center = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
                        let _ = crate::sketch::edit::circle(&mut draft, center, *b, None);
                    }
                    (Tool::Circle3, [a, b, c]) => {
                        if let Ok(center) = crate::sketch::edit::circumcenter(*a, *b, *c) {
                            let _ = crate::sketch::edit::circle(&mut draft, center, *a, None);
                        }
                    }
                    (Tool::Arc3, [a, b, c]) => {
                        if let Ok(center) = crate::sketch::edit::circumcenter(*a, *b, *c) {
                            let cross =
                                (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
                            let (start, end) = if cross > 0. { (*a, *c) } else { (*c, *a) };
                            let _ =
                                crate::sketch::edit::circle(&mut draft, center, start, Some(end));
                        }
                    }
                    (Tool::Spline, points) if points.len() >= 2 => {
                        let _ = crate::sketch::edit::spline(&mut draft, points, true);
                    }
                    (Tool::Ellipse, [a, b, c]) => {
                        let _ = crate::sketch::edit::ellipse(&mut draft, *a, *b, *c);
                    }
                    (Tool::Slot, [a, b, c]) => {
                        let _ = crate::sketch::edit::slot(&mut draft, *a, *b, *c);
                    }
                    (Tool::Polygon, [a, b]) => {
                        let _ = crate::sketch::edit::polygon(
                            &mut draft,
                            *a,
                            *b,
                            self.transform_count.read(cx).content.parse().unwrap_or(6),
                        );
                    }
                    _ => {}
                }
                let curves = crate::sketch::entities::curve_ids(&draft);
                if !curves.is_empty() {
                    preview.clear();
                    for id in curves {
                        preview_curves.push(crate::sketch::entities::samples(&draft, id));
                    }
                }
            }
            let region_wires = self
                .sketch_region
                .as_ref()
                .filter(|region| {
                    self.sketch && region.boundary.iter().all(|id| self.selection.contains(id))
                })
                .map(|region| {
                    region
                        .wires
                        .iter()
                        .map(|wire| wire.iter().flat_map(|edge| edge.samples()).collect())
                        .collect()
                })
                .unwrap_or_default();
            let overlay = SketchCanvas {
                bounds: self.bounds.clone(),
                camera: self.camera.clone(),
                frame: self.active_frame(),
                placement_preview: self.sketch
                    && !matches!(
                        self.tool,
                        Tool::Select
                            | Tool::Dimension
                            | Tool::Measure
                            | Tool::Trim
                            | Tool::Extend
                            | Tool::Break
                    ),
                active: self.mode != Mode::Drawing,
                visible: !self.hidden.contains("sketches")
                    && !self
                        .design
                        .current_sketch_id()
                        .is_some_and(|id| self.hidden_sketches.contains(&id)),
                grid: false,
                axes: false,
                coords,
                lines,
                annotations,
                selected: self.selection.clone(),
                preview,
                preview_curves,
                region_wires,
                dimensions,
                point_colors,
                anchor: if matches!(self.tool, Tool::Line | Tool::Rectangle) {
                    self.anchor
                } else {
                    None
                },
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
                .children(
                    self.world_sketches
                        .iter()
                        .filter(|(id, _, _)| {
                            Some(*id) != self.design.current_sketch_id()
                                && !self.hidden_sketches.contains(id)
                        })
                        .map(|(_, design, frame)| {
                            SketchCanvas {
                                bounds: self.bounds.clone(),
                                camera: self.camera.clone(),
                                frame: *frame,
                                placement_preview: false,
                                active: self.mode != Mode::Drawing
                                    && !self.hidden.contains("sketches"),
                                visible: true,
                                grid: false,
                                axes: false,
                                coords: vec![],
                                lines: crate::sketch::entities::curve_ids(design)
                                    .into_iter()
                                    .map(|id| {
                                        (
                                            id,
                                            crate::sketch::entities::samples(design, id),
                                            t::MUTED,
                                            false,
                                        )
                                    })
                                    .collect(),
                                preview: vec![],
                                preview_curves: vec![],
                                region_wires: vec![],
                                dimensions: vec![],
                                point_colors: vec![],
                                annotations: vec![],
                                selected: vec![],
                                anchor: None,
                                hover: None,
                                rectangle: false,
                                scale: self.scale,
                                center: self.center,
                            }
                            .element()
                        }),
                )
                .children(self.choosing_sketch_face.then(|| {
                    div()
                        .absolute()
                        .top(px(16.))
                        .left(px(280.))
                        .p_2()
                        .bg(rgb(t::PANEL))
                        .child("Select a planar face · ")
                        .child(
                            div()
                                .id("sketch-xy-plane")
                                .cursor_pointer()
                                .child("Use XY plane")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.choosing_sketch_face = false;
                                    this.selected = 0;
                                    this.create_sketch(cx);
                                })),
                        )
                }))
                .children(self.extrusion_preview(cx).map(|preview| preview.element()))
                .children(constraint_markers)
                .children(self.center_of_mass_marker())
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
                .child(self.browser(cx))
                .child(self.navigation(cx))
                .children(self.inspector(cx))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, event: &MouseDownEvent, window, cx| {
                        if this.mode == Mode::Drawing {
                            return;
                        }
                        window.focus(&this.focus);
                        this.menu = None;
                        if let (Some(preview), Some(bounds)) =
                            (this.extrusion_preview(cx), this.bounds.get())
                        {
                            let base = preview.frame.world(preview.center);
                            let project = |depth: f64| {
                                this.camera.project(
                                    nalgebra::Point3::from(
                                        (base + preview.frame.normal * depth).coords * 25.,
                                    ),
                                    [
                                        f64::from(bounds.size.width) as u32,
                                        f64::from(bounds.size.height) as u32,
                                    ],
                                )
                            };
                            if let (Some(handle), Some(next)) =
                                (project(preview.depth), project(preview.depth + 0.001))
                            {
                                let cursor = [
                                    f64::from(event.position.x - bounds.origin.x),
                                    f64::from(event.position.y - bounds.origin.y),
                                ];
                                if (cursor[0] - handle[0]).hypot(cursor[1] - handle[1]) < 15. {
                                    this.extrude_drag = Some((
                                        event.position,
                                        preview.depth,
                                        [next[0] - handle[0], next[1] - handle[1]],
                                    ));
                                    cx.notify();
                                    return;
                                }
                            }
                        }
                        if this.sketch {
                            this.sketch_click(
                                event.position,
                                event.modifiers.shift,
                                event.modifiers.control || event.modifiers.platform,
                                cx,
                            );
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
                    if let Some((start, initial, axis)) = this.extrude_drag {
                        if event.pressed_button != Some(MouseButton::Left) {
                            this.extrude_drag = None;
                        } else {
                            let delta = [
                                f64::from(event.position.x - start.x),
                                f64::from(event.position.y - start.y),
                            ];
                            let length = axis[0] * axis[0] + axis[1] * axis[1];
                            if length > 1e-8 {
                                let signed = initial
                                    + (delta[0] * axis[0] + delta[1] * axis[1]) / length * 0.001;
                                let depth = (signed * initial.signum()).max(0.0001);
                                this.depth.update(cx, |input, cx| {
                                    input.set_content(format!("{:.3} mm", depth * 1000.), cx)
                                });
                            }
                            cx.notify();
                            return;
                        }
                    }
                    if this.sketch {
                        this.hover = this.sketch_position(
                            event.position,
                            event.modifiers.control || event.modifiers.platform,
                        );
                        if event.pressed_button == Some(MouseButton::Left)
                            && let Some((id, before, moved)) = &mut this.dimension_drag
                            && let Some(at) = this.hover
                            && before.dimension_positions.get(id).is_some_and(|p| {
                                (p[0] - at[0]).hypot(p[1] - at[1]) * this.scale > 3.
                            })
                        {
                            this.design.dimension_positions.insert(*id, at);
                            this.dirty_cache.set((u64::MAX, false));
                            *moved = true;
                        }
                        if let Some((_, current, _)) = &mut this.marquee
                            && let Some(p) = this.hover
                        {
                            *current = p
                        }
                        if event.pressed_button == Some(MouseButton::Left)
                            && let Some(drag) = &this.sketch_drag
                            && let Some(at) = this.hover
                        {
                            let delta = [at[0] - drag.start[0], at[1] - drag.start[1]];
                            if delta[0].hypot(delta[1]) * this.scale > 2. {
                                let mut candidate = drag.before.clone();
                                for p in &mut candidate.points {
                                    if drag.points.contains(&p.id) {
                                        p.xy[0] += delta[0];
                                        p.xy[1] += delta[1];
                                    }
                                }
                                if let Ok(parameters) =
                                    crate::parameters::expression::evaluate(&candidate)
                                    && let Ok(solution) = crate::solver::nonlinear::solve_drag(
                                        &candidate,
                                        &parameters,
                                        &candidate
                                            .points
                                            .iter()
                                            .filter(|p| drag.points.contains(&p.id))
                                            .map(|p| (p.id, p.xy))
                                            .collect::<Vec<_>>(),
                                    )
                                    && solution.conflicts.is_empty()
                                {
                                    for (p, xy) in candidate.points.iter_mut().zip(&solution.points)
                                    {
                                        p.xy = *xy
                                    }
                                    this.design = candidate;
                                    this.solved = solution.points;
                                    this.point_dof = solution.point_dof;
                                    if !this.sketch_drag.as_ref().unwrap().moved {
                                        this.revision = this.revision.wrapping_add(1);
                                    }
                                    this.sketch_drag.as_mut().unwrap().moved = true;
                                }
                            }
                        }
                        cx.notify();
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
                    if event.modifiers.control {
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
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.extrude_drag = None;
                        this.finish_sketch_drag(false, cx);
                    }),
                )
                .on_mouse_up_out(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.extrude_drag = None;
                        this.finish_sketch_drag(false, cx);
                    }),
                )
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
                    this.camera.distance =
                        (this.camera.distance * (-delta * 0.0025).exp()).clamp(0.002, 100.);
                    let height = this.bounds.get().map_or(575., |b| f64::from(b.size.height));
                    this.scale = height * 25.
                        / (2. * this.camera.distance * (Camera::FIELD_OF_VIEW / 2.).tan());
                    this.changed_camera();
                    cx.notify();
                }))
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
            .when(selected, |el| el.bg(rgb(t::SELECTED)))
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
                screen(center) - point(px(face.label.len() as f32 * 2.6), px(6.)),
                t::MUTED,
                10.,
                window,
                cx,
            );
        }

        for axis in view_cube::axes(camera) {
            let mut path = PathBuilder::stroke(px(2.));
            path.move_to(screen(axis.origin));
            path.line_to(screen(axis.end));
            if let Ok(path) = path.build() {
                window.paint_path(path, rgb(axis.color));
            }
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
            if self
                .menu_leave_deadline
                .is_some_and(|deadline| std::time::Instant::now() >= deadline)
            {
                self.menu = None;
                self.menu_leave_deadline = None;
            }
            self.poll_recovery(cx);
            self.poll_step_export();
            if !self.close_hook_installed {
                let entity = cx.entity().downgrade();
                window.on_window_should_close(cx, move |window, cx| {
                    entity
                        .update(cx, |this, cx| this.request_window_close(window, cx))
                        .unwrap_or(true)
                });
                self.close_hook_installed = true;
            }
            let export_chosen = self.export_dialog.as_ref().and_then(|(format, receiver)| {
                match receiver.try_recv() {
                    Ok(path) => Some((*format, path)),
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => Some((*format, None)),
                    Err(_) => None,
                }
            });
            if let Some((format, path)) = export_chosen {
                self.export_dialog = None;
                if let Some(mut path) = path {
                    path.set_extension(format.extension());
                    let design = self.design.clone();
                    let (send, receive) = std::sync::mpsc::channel();
                    self.export_result = Some(receive);
                    self.status = "Exporting model…".into();
                    std::thread::spawn(move || {
                        let result = crate::exchange::export::export_design(&path, &design, format)
                            .map(|()| path);
                        let _ = send.send(result);
                    });
                }
            }
            let export_result =
                self.export_result
                    .as_ref()
                    .and_then(|receiver| match receiver.try_recv() {
                        Ok(result) => Some(result),
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            Some(Err("Export worker stopped".into()))
                        }
                        Err(_) => None,
                    });
            if let Some(result) = export_result {
                self.export_result = None;
                match result {
                    Ok(path) => {
                        self.status = format!("Exported {}", path.display());
                        self.error = None;
                    }
                    Err(error) => {
                        self.status = "Export failed".into();
                        self.error = Some(error);
                    }
                }
            }
            let chosen = self.native_dialog.as_ref().and_then(|(open, receiver)| {
                match receiver.try_recv() {
                    Ok(path) => Some((*open, path)),
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => Some((*open, None)),
                    Err(_) => None,
                }
            });
            if let Some((open, path)) = chosen {
                self.native_dialog = None;
                if let Some(mut path) = path {
                    if !open && path.extension().and_then(|x| x.to_str()) != Some("con") {
                        path.set_extension("con");
                    }
                    self.path = cx.new(|cx| TextInput::new(&path.display().to_string(), cx));
                    if open {
                        self.open(cx);
                    } else {
                        self.save(cx);
                        if self.error.is_none() {
                            self.finish_close(window, cx);
                        }
                    }
                }
            }
            if self.pending_candidate.as_ref().is_some_and(|pending| {
                !pending.is_current(&self.design, &self.candidate_context(cx))
            }) {
                self.rebuild(cx);
            }
            let evaluated = self
                .worker
                .poll()
                .filter(|result| result.revision == self.revision);
            let evaluated = evaluated.and_then(|result| {
                if let Some(pending) = self.pending_candidate.take() {
                    let success = result.solution.is_ok() && result.mesh.is_ok();
                    let context = self.candidate_context(cx);
                    if let Some(candidate) =
                        pending.finish(result.revision, success, &self.design, &context)
                    {
                        self.checkpoint();
                        self.design = candidate;
                        // Evaluation used this revision while the document was still
                        // unchanged, so its cached dirty flag must be invalidated.
                        self.dirty_cache.set((u64::MAX, false));
                        if self.panel == Some(Panel::Create) {
                            self.fit_pending = true;
                        }
                        self.create_editor = None;
                        self.panel = None;
                        self.construction_cursor = None;
                        self.before_construction = false;
                        self.selected = 0;
                        self.changed_camera();
                        self.inputs = self
                            .design
                            .parameters
                            .iter()
                            .map(|p| (p.id, cx.new(|cx| TextInput::new(&p.expression, cx))))
                            .collect();
                        window.focus(&self.focus);
                    } else {
                        self.status = "Validation failed".into();
                        self.error = result.solution.err().or_else(|| result.mesh.err());
                        return None;
                    }
                }
                Some(result)
            });
            if let Some(result) = evaluated {
                if let Err(error) = &result.solution {
                    self.status = "Evaluation failed".into();
                    self.error = Some(error.clone());
                } else {
                    self.evaluated_features = result.features;
                    let frames: Vec<_> = result
                        .mesh
                        .as_ref()
                        .ok()
                        .and_then(|m| m.as_ref())
                        .map_or_else(Vec::new, |m| {
                            m.planes
                                .iter()
                                .map(crate::sketch::workplane::Workplane::from)
                                .collect()
                        });
                    self.world_sketches = result
                        .sketches
                        .into_iter()
                        .enumerate()
                        .map(|(index, (id, design))| {
                            (id, design, frames.get(index).copied().unwrap_or_default())
                        })
                        .collect();
                    if self.align_sketch_pending && self.sketch {
                        self.align_to_sketch();
                    }
                    match result.solution {
                        Ok(solution) if result.sketch != self.design.current_sketch_id() => {
                            self.solved.clear();
                            self.point_dof.clear();
                            self.conflicts = solution.conflicts;
                            self.status = "Construction preview".into();
                        }
                        Ok(solution) => {
                            self.point_dof = solution.point_dof;
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
                            if solution.conflicts.is_empty() && !solution.redundant.is_empty() {
                                self.status.push_str(&format!(
                                    " · {} redundant",
                                    solution.redundant.len()
                                ));
                            }
                            self.conflicts = solution.conflicts;
                        }
                        Err(e) => self.error = Some(e),
                    }
                    match result.mesh {
                        Ok(Some(mesh)) => {
                            self.body_faces = mesh
                                .bodies
                                .iter()
                                .filter_map(|body| {
                                    self.evaluated_features
                                        .get(body.body as usize)
                                        .map(|id| (body.face, (*id, body.part)))
                                })
                                .collect();
                            self.face_anchors = mesh.anchors;
                            self.volume = Some(mesh.volume);
                            self.inspection = Some(mesh.inspection);
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
                                        color: self.face_color(v.face),
                                    })
                                    .collect(),
                                mesh.indices,
                            ));
                            self.refresh_mesh();
                            if self.fit_pending {
                                self.fit();
                            }
                        }
                        Ok(None) => {
                            self.mesh = None;
                            self.body_faces.clear();
                            self.face_anchors.clear();
                            self.refresh_mesh();
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
            }
            self.frame_rate.tick();
            if self.inspection_preview
                && (self.panel != Some(Panel::Inspect)
                    || !matches!(
                        self.inspection_action,
                        Action::SectionAnalysis | Action::CurvatureAnalysis | Action::DraftAnalysis
                    ))
            {
                self.refresh_mesh();
            }
            self.gpu.set_grid_frame(if self.sketch {
                self.active_frame()
            } else {
                Default::default()
            });
            self.gpu.set_grid(
                self.mode != Mode::Drawing && self.grid,
                !self.hidden.contains("origin"),
            );
            match self
                .gpu
                .draw(&self.camera, self.selected, self.pending_pick.take())
            {
                Ok(Some(pick)) if pick.revision == self.view_revision => {
                    self.selected = pick.face;
                    if self.choosing_sketch_face && self.selected != 0 {
                        self.choosing_sketch_face = false;
                        self.create_sketch(cx);
                    }
                    if matches!(self.panel, Some(Panel::SolidModify | Panel::Materials))
                        && let Some(body) = self.selected_body()
                    {
                        self.solid_editor.target = Some(body);
                    }
                }
                Ok(_) => {}
                Err(e) => self.error = Some(e),
            }
            let input = self.panel_fields().first().cloned().filter(|_| {
                self.panel != Some(Panel::Dimension)
                    || self.dimension_position.is_some()
                    || self.dimension_edit.is_some()
            });
            if input.as_ref().map(Entity::entity_id) != self.editor_input {
                self.editor_input = input.as_ref().map(Entity::entity_id);
                if let Some(input) = input {
                    // The editor must enter the focus tree before receiving keyboard input.
                    window.on_next_frame(move |window, cx| {
                        input.update(cx, |input, cx| input.focus_and_select(window, cx));
                    });
                }
            }
            let entity = cx.entity().downgrade();
            window.on_next_frame(move |_, cx| {
                let _ = entity.update(cx, |_, cx| cx.notify());
            });
            let main = div()
                .flex()
                .flex_1()
                .min_h_0()
                .child(self.scene_element(cx));

            let menu = self.menu_overlay(cx);
            div()
                .size_full()
                .relative()
                .flex()
                .flex_col()
                .bg(rgb(t::VIEWPORT))
                .text_color(rgb(t::TEXT))
                .font_family(self.ui_font_family.clone())
                .text_size(px(14.))
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
                                .child(self.error.clone().unwrap_or_else(|| {
                                    if self.conflicts.is_empty() {
                                        format!("{} · {}", self.tool_prompt(), self.status)
                                    } else {
                                        format!("{} conflicting constraints", self.conflicts.len())
                                    }
                                })),
                        )
                        .child(div().text_color(rgb(t::MUTED)).child("mm")),
                )
                .children(menu)
                .children(self.close_confirmation(cx))
                .children(self.recovery_prompt(cx))
        }
    }
}
#[cfg(feature = "desktop")]
pub use implementation::WorkspaceView;
