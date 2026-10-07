//! Complete scoped feature inventory for mode-specific ribbons and menus.
//! Exports Mode, Action, Feature and groups. Implemented actions route to the workspace;
//! every other feature has an explicit unavailable state. No out-of-scope workbenches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Solid,
    Sketch,
    Drawing,
}
impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Solid => "Solid",
            Self::Sketch => "Sketch",
            Self::Drawing => "Drawing",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Sketch,
    Extrude,
    Parameters,
    Select,
    Line,
    Rectangle,
    Horizontal,
    Vertical,
    Fixed,
    Dimension,
    Finish,
}
#[derive(Clone, Copy, Debug)]
pub struct Feature {
    pub id: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    pub action: Option<Action>,
}
impl Feature {
    pub fn available(self) -> bool {
        self.action.is_some()
    }
}
pub struct Group {
    pub name: &'static str,
    pub features: &'static [Feature],
}
const SOLID_0: &[Feature] = &[
    Feature {
        id: "solid-create-sketch",
        name: "Create sketch",
        icon: "line_rectangle",
        action: Some(Action::Sketch),
    },
    Feature {
        id: "solid-extrude",
        name: "Extrude",
        icon: "up",
        action: Some(Action::Extrude),
    },
    Feature {
        id: "solid-revolve",
        name: "Revolve",
        icon: "rotate",
        action: None,
    },
    Feature {
        id: "solid-sweep",
        name: "Sweep",
        icon: "spline",
        action: None,
    },
    Feature {
        id: "solid-loft",
        name: "Loft",
        icon: "polylines",
        action: None,
    },
    Feature {
        id: "solid-rib",
        name: "Rib",
        icon: "line_perpendicular",
        action: None,
    },
    Feature {
        id: "solid-web",
        name: "Web",
        icon: "line_parallel",
        action: None,
    },
    Feature {
        id: "solid-emboss",
        name: "Emboss",
        icon: "text",
        action: None,
    },
    Feature {
        id: "solid-hole",
        name: "Hole",
        icon: "circle",
        action: None,
    },
    Feature {
        id: "solid-thread",
        name: "Thread",
        icon: "spline_points",
        action: None,
    },
    Feature {
        id: "solid-box",
        name: "Box",
        icon: "create_block",
        action: None,
    },
    Feature {
        id: "solid-cylinder",
        name: "Cylinder",
        icon: "circle_center_radius",
        action: None,
    },
    Feature {
        id: "solid-sphere",
        name: "Sphere",
        icon: "circle_3_points",
        action: None,
    },
    Feature {
        id: "solid-torus",
        name: "Torus",
        icon: "circle_concentric",
        action: None,
    },
    Feature {
        id: "solid-coil",
        name: "Coil",
        icon: "arc_continuation",
        action: None,
    },
    Feature {
        id: "solid-pipe",
        name: "Pipe",
        icon: "offset",
        action: None,
    },
    Feature {
        id: "solid-rectangular-pattern",
        name: "Rectangular pattern",
        icon: "select_window",
        action: None,
    },
    Feature {
        id: "solid-circular-pattern",
        name: "Circular pattern",
        icon: "rotate2",
        action: None,
    },
    Feature {
        id: "solid-pattern-on-path",
        name: "Pattern on path",
        icon: "spline",
        action: None,
    },
    Feature {
        id: "solid-mirror",
        name: "Mirror",
        icon: "mirror",
        action: None,
    },
    Feature {
        id: "solid-thicken",
        name: "Thicken",
        icon: "offset",
        action: None,
    },
    Feature {
        id: "solid-boundary-fill",
        name: "Boundary fill",
        icon: "hatch",
        action: None,
    },
    Feature {
        id: "solid-gear",
        name: "Gear",
        icon: "circle",
        action: None,
    },
];
const SOLID_1: &[Feature] = &[
    Feature {
        id: "solid-press-pull",
        name: "Press pull",
        icon: "stretch",
        action: None,
    },
    Feature {
        id: "solid-fillet",
        name: "Fillet",
        icon: "fillet",
        action: None,
    },
    Feature {
        id: "solid-chamfer",
        name: "Chamfer",
        icon: "bevel",
        action: None,
    },
    Feature {
        id: "solid-shell",
        name: "Shell",
        icon: "offset",
        action: None,
    },
    Feature {
        id: "solid-draft",
        name: "Draft",
        icon: "draft",
        action: None,
    },
    Feature {
        id: "solid-scale",
        name: "Scale",
        icon: "scale",
        action: None,
    },
    Feature {
        id: "solid-combine",
        name: "Combine",
        icon: "create_block",
        action: None,
    },
    Feature {
        id: "solid-offset-face",
        name: "Offset face",
        icon: "offset",
        action: None,
    },
    Feature {
        id: "solid-replace-face",
        name: "Replace face",
        icon: "modify",
        action: None,
    },
    Feature {
        id: "solid-split-body",
        name: "Split body",
        icon: "divide",
        action: None,
    },
    Feature {
        id: "solid-split-face",
        name: "Split face",
        icon: "trim2",
        action: None,
    },
    Feature {
        id: "solid-silhouette-split",
        name: "Silhouette split",
        icon: "trim",
        action: None,
    },
    Feature {
        id: "solid-move-copy",
        name: "Move / copy",
        icon: "move_copy",
        action: None,
    },
    Feature {
        id: "solid-align",
        name: "Align",
        icon: "center_to_page",
        action: None,
    },
    Feature {
        id: "solid-delete",
        name: "Delete",
        icon: "delete",
        action: None,
    },
    Feature {
        id: "solid-remove",
        name: "Remove",
        icon: "remove",
        action: None,
    },
    Feature {
        id: "solid-simplify",
        name: "Simplify",
        icon: "modify",
        action: None,
    },
    Feature {
        id: "solid-physical-material",
        name: "Physical material",
        icon: "hatch",
        action: None,
    },
    Feature {
        id: "solid-appearance",
        name: "Appearance",
        icon: "attributes",
        action: None,
    },
    Feature {
        id: "solid-manage-materials",
        name: "Manage materials",
        icon: "properties",
        action: None,
    },
    Feature {
        id: "solid-change-parameters",
        name: "Change parameters",
        icon: "properties",
        action: Some(Action::Parameters),
    },
];
const SOLID_2: &[Feature] = &[
    Feature {
        id: "solid-new-component",
        name: "New component",
        icon: "create_block",
        action: None,
    },
    Feature {
        id: "solid-move-to-component",
        name: "Move to component",
        icon: "move_copy",
        action: None,
    },
    Feature {
        id: "solid-organize-bodies",
        name: "Organize bodies",
        icon: "dockwidgets_left",
        action: None,
    },
    Feature {
        id: "solid-activate-component",
        name: "Activate component",
        icon: "select_entity",
        action: None,
    },
    Feature {
        id: "solid-duplicate-component",
        name: "Duplicate component",
        icon: "copy",
        action: None,
    },
];
const SOLID_3: &[Feature] = &[
    Feature {
        id: "solid-offset-plane",
        name: "Offset plane",
        icon: "construction_layer",
        action: None,
    },
    Feature {
        id: "solid-plane-at-angle",
        name: "Plane at angle",
        icon: "line_angle",
        action: None,
    },
    Feature {
        id: "solid-tangent-plane",
        name: "Tangent plane",
        icon: "line_tangent_pc",
        action: None,
    },
    Feature {
        id: "solid-midplane",
        name: "Midplane",
        icon: "line_parallel",
        action: None,
    },
    Feature {
        id: "solid-perpendicular-plane",
        name: "Perpendicular plane",
        icon: "line_perpendicular",
        action: None,
    },
    Feature {
        id: "solid-plane-through-two-edges",
        name: "Plane through two edges",
        icon: "line_2p",
        action: None,
    },
    Feature {
        id: "solid-plane-through-three-points",
        name: "Plane through three points",
        icon: "points",
        action: None,
    },
    Feature {
        id: "solid-plane-along-path",
        name: "Plane along path",
        icon: "spline",
        action: None,
    },
    Feature {
        id: "solid-plane-through-cylinder-cone-torus",
        name: "Plane through cylinder / cone / torus",
        icon: "circle",
        action: None,
    },
    Feature {
        id: "solid-axis-through-cylinder-cone-torus",
        name: "Axis through cylinder / cone / torus",
        icon: "circle_center_point",
        action: None,
    },
    Feature {
        id: "solid-axis-perpendicular-to-face",
        name: "Axis perpendicular to face",
        icon: "line_perpendicular",
        action: None,
    },
    Feature {
        id: "solid-axis-through-two-points",
        name: "Axis through two points",
        icon: "line_2p",
        action: None,
    },
    Feature {
        id: "solid-axis-through-two-planes",
        name: "Axis through two planes",
        icon: "line_bisector",
        action: None,
    },
    Feature {
        id: "solid-axis-through-edge",
        name: "Axis through edge",
        icon: "line",
        action: None,
    },
    Feature {
        id: "solid-point-at-vertex",
        name: "Point at vertex",
        icon: "points",
        action: None,
    },
    Feature {
        id: "solid-point-through-two-edges",
        name: "Point through two edges",
        icon: "snap_intersection",
        action: None,
    },
    Feature {
        id: "solid-point-through-three-planes",
        name: "Point through three planes",
        icon: "points",
        action: None,
    },
    Feature {
        id: "solid-point-at-circle-center",
        name: "Point at circle center",
        icon: "snap_center",
        action: None,
    },
    Feature {
        id: "solid-point-at-edge-and-plane",
        name: "Point at edge and plane",
        icon: "snap_intersection",
        action: None,
    },
    Feature {
        id: "solid-point-along-path",
        name: "Point along path",
        icon: "snap_distance",
        action: None,
    },
];
const SOLID_4: &[Feature] = &[
    Feature {
        id: "solid-measure",
        name: "Measure",
        icon: "measure",
        action: None,
    },
    Feature {
        id: "solid-section-analysis",
        name: "Section analysis",
        icon: "hatch",
        action: None,
    },
    Feature {
        id: "solid-interference",
        name: "Interference",
        icon: "exclusive",
        action: None,
    },
    Feature {
        id: "solid-center-of-mass",
        name: "Center of mass",
        icon: "snap_center",
        action: None,
    },
    Feature {
        id: "solid-curvature-analysis",
        name: "Curvature analysis",
        icon: "spline",
        action: None,
    },
    Feature {
        id: "solid-draft-analysis",
        name: "Draft analysis",
        icon: "draft",
        action: None,
    },
    Feature {
        id: "solid-validate-solid",
        name: "Validate solid",
        icon: "properties",
        action: None,
    },
];
const SOLID_5: &[Feature] = &[
    Feature {
        id: "solid-import-step",
        name: "Import STEP",
        icon: "import",
        action: None,
    },
    Feature {
        id: "solid-import-stl",
        name: "Import STL",
        icon: "import",
        action: None,
    },
    Feature {
        id: "solid-insert-image",
        name: "Insert image",
        icon: "export_image",
        action: None,
    },
    Feature {
        id: "solid-insert-design",
        name: "Insert design",
        icon: "insert_active_block",
        action: None,
    },
];
const SOLID_6: &[Feature] = &[
    Feature {
        id: "solid-select",
        name: "Select",
        icon: "cursor",
        action: Some(Action::Select),
    },
    Feature {
        id: "solid-select-all",
        name: "Select all",
        icon: "select_all",
        action: None,
    },
    Feature {
        id: "solid-selection-filters",
        name: "Selection filters",
        icon: "select",
        action: None,
    },
    Feature {
        id: "solid-select-body",
        name: "Select body",
        icon: "select_entity",
        action: None,
    },
    Feature {
        id: "solid-select-component",
        name: "Select component",
        icon: "create_block",
        action: None,
    },
];
const SKETCH_0: &[Feature] = &[
    Feature {
        id: "sketch-line",
        name: "Line",
        icon: "line",
        action: Some(Action::Line),
    },
    Feature {
        id: "sketch-rectangle",
        name: "Rectangle",
        icon: "line_rectangle",
        action: Some(Action::Rectangle),
    },
    Feature {
        id: "sketch-center-rectangle",
        name: "Center rectangle",
        icon: "line_rectangle",
        action: None,
    },
    Feature {
        id: "sketch-circle",
        name: "Circle",
        icon: "circle_center_radius",
        action: None,
    },
    Feature {
        id: "sketch-two-point-circle",
        name: "Two-point circle",
        icon: "circle_2_points",
        action: None,
    },
    Feature {
        id: "sketch-three-point-circle",
        name: "Three-point circle",
        icon: "circle_3_points",
        action: None,
    },
    Feature {
        id: "sketch-arc",
        name: "Arc",
        icon: "arc_3_points",
        action: None,
    },
    Feature {
        id: "sketch-center-arc",
        name: "Center arc",
        icon: "arc_center_point_angle",
        action: None,
    },
    Feature {
        id: "sketch-tangent-arc",
        name: "Tangent arc",
        icon: "arc_continuation",
        action: None,
    },
    Feature {
        id: "sketch-ellipse",
        name: "Ellipse",
        icon: "ellipse_axis",
        action: None,
    },
    Feature {
        id: "sketch-polygon",
        name: "Polygon",
        icon: "line_polygon_cen_cor",
        action: None,
    },
    Feature {
        id: "sketch-slot",
        name: "Slot",
        icon: "offset",
        action: None,
    },
    Feature {
        id: "sketch-spline",
        name: "Spline",
        icon: "spline",
        action: None,
    },
    Feature {
        id: "sketch-point",
        name: "Point",
        icon: "points",
        action: None,
    },
    Feature {
        id: "sketch-text",
        name: "Text",
        icon: "text",
        action: None,
    },
    Feature {
        id: "sketch-project-geometry",
        name: "Project geometry",
        icon: "create_polyline_from_existing_segments",
        action: None,
    },
    Feature {
        id: "sketch-intersect-geometry",
        name: "Intersect geometry",
        icon: "snap_intersection",
        action: None,
    },
    Feature {
        id: "sketch-include-3d-geometry",
        name: "Include 3D geometry",
        icon: "import",
        action: None,
    },
];
const SKETCH_1: &[Feature] = &[
    Feature {
        id: "sketch-sketch-fillet",
        name: "Sketch fillet",
        icon: "fillet",
        action: None,
    },
    Feature {
        id: "sketch-trim",
        name: "Trim",
        icon: "trim",
        action: None,
    },
    Feature {
        id: "sketch-extend",
        name: "Extend",
        icon: "stretch",
        action: None,
    },
    Feature {
        id: "sketch-break",
        name: "Break",
        icon: "divide",
        action: None,
    },
    Feature {
        id: "sketch-offset",
        name: "Offset",
        icon: "offset",
        action: None,
    },
    Feature {
        id: "sketch-move-copy-sketch",
        name: "Move / copy sketch",
        icon: "move_copy",
        action: None,
    },
    Feature {
        id: "sketch-mirror-sketch",
        name: "Mirror sketch",
        icon: "mirror",
        action: None,
    },
    Feature {
        id: "sketch-rectangular-sketch-pattern",
        name: "Rectangular sketch pattern",
        icon: "select_window",
        action: None,
    },
    Feature {
        id: "sketch-circular-sketch-pattern",
        name: "Circular sketch pattern",
        icon: "rotate2",
        action: None,
    },
    Feature {
        id: "sketch-scale-sketch",
        name: "Scale sketch",
        icon: "scale",
        action: None,
    },
    Feature {
        id: "sketch-delete-geometry",
        name: "Delete geometry",
        icon: "delete",
        action: None,
    },
];
const SKETCH_2: &[Feature] = &[
    Feature {
        id: "sketch-horizontal",
        name: "Horizontal",
        icon: "line_horizontal",
        action: Some(Action::Horizontal),
    },
    Feature {
        id: "sketch-vertical",
        name: "Vertical",
        icon: "line_vertical",
        action: Some(Action::Vertical),
    },
    Feature {
        id: "sketch-fix-point",
        name: "Fix point",
        icon: "locked",
        action: Some(Action::Fixed),
    },
    Feature {
        id: "sketch-coincident",
        name: "Coincident",
        icon: "snap_endpoints",
        action: None,
    },
    Feature {
        id: "sketch-collinear",
        name: "Collinear",
        icon: "line",
        action: None,
    },
    Feature {
        id: "sketch-concentric",
        name: "Concentric",
        icon: "circle_concentric",
        action: None,
    },
    Feature {
        id: "sketch-parallel",
        name: "Parallel",
        icon: "line_parallel",
        action: None,
    },
    Feature {
        id: "sketch-perpendicular",
        name: "Perpendicular",
        icon: "line_perpendicular",
        action: None,
    },
    Feature {
        id: "sketch-tangent",
        name: "Tangent",
        icon: "line_tangent_pc",
        action: None,
    },
    Feature {
        id: "sketch-equal",
        name: "Equal",
        icon: "restr_ortho",
        action: None,
    },
    Feature {
        id: "sketch-symmetry",
        name: "Symmetry",
        icon: "mirror",
        action: None,
    },
    Feature {
        id: "sketch-midpoint",
        name: "Midpoint",
        icon: "snap_middle",
        action: None,
    },
    Feature {
        id: "sketch-dimension",
        name: "Dimension",
        icon: "dim_linear",
        action: Some(Action::Dimension),
    },
];
const SKETCH_3: &[Feature] = &[
    Feature {
        id: "sketch-sketch-dimensions",
        name: "Sketch dimensions",
        icon: "dim_linear",
        action: Some(Action::Dimension),
    },
    Feature {
        id: "sketch-sketch-parameters",
        name: "Sketch parameters",
        icon: "properties",
        action: Some(Action::Parameters),
    },
    Feature {
        id: "sketch-measure-sketch",
        name: "Measure sketch",
        icon: "measure",
        action: None,
    },
];
const SKETCH_4: &[Feature] = &[Feature {
    id: "sketch-finish-sketch",
    name: "Finish sketch",
    icon: "exclusive",
    action: Some(Action::Finish),
}];
const DRAWING_0: &[Feature] = &[
    Feature {
        id: "drawing-base-view",
        name: "Base view",
        icon: "create_block",
        action: None,
    },
    Feature {
        id: "drawing-projected-view",
        name: "Projected view",
        icon: "copy",
        action: None,
    },
    Feature {
        id: "drawing-section-view",
        name: "Section view",
        icon: "hatch",
        action: None,
    },
    Feature {
        id: "drawing-detail-view",
        name: "Detail view",
        icon: "zoom_window",
        action: None,
    },
    Feature {
        id: "drawing-auxiliary-view",
        name: "Auxiliary view",
        icon: "rotate",
        action: None,
    },
    Feature {
        id: "drawing-break-view",
        name: "Break view",
        icon: "divide",
        action: None,
    },
];
const DRAWING_1: &[Feature] = &[
    Feature {
        id: "drawing-drawing-dimension",
        name: "Drawing dimension",
        icon: "dim_linear",
        action: None,
    },
    Feature {
        id: "drawing-aligned-dimension",
        name: "Aligned dimension",
        icon: "dim_aligned",
        action: None,
    },
    Feature {
        id: "drawing-angular-dimension",
        name: "Angular dimension",
        icon: "dim_angular",
        action: None,
    },
    Feature {
        id: "drawing-diameter-dimension",
        name: "Diameter dimension",
        icon: "dim_diametric",
        action: None,
    },
    Feature {
        id: "drawing-radius-dimension",
        name: "Radius dimension",
        icon: "dim_radial",
        action: None,
    },
    Feature {
        id: "drawing-ordinate-dimension",
        name: "Ordinate dimension",
        icon: "dim_horizontal",
        action: None,
    },
    Feature {
        id: "drawing-center-mark",
        name: "Center mark",
        icon: "snap_center",
        action: None,
    },
    Feature {
        id: "drawing-centerline",
        name: "Centerline",
        icon: "line_bisector",
        action: None,
    },
    Feature {
        id: "drawing-hole-thread-note",
        name: "Hole / thread note",
        icon: "text",
        action: None,
    },
    Feature {
        id: "drawing-leader-note",
        name: "Leader note",
        icon: "dim_leader",
        action: None,
    },
    Feature {
        id: "drawing-surface-texture",
        name: "Surface texture",
        icon: "attributes",
        action: None,
    },
    Feature {
        id: "drawing-feature-control-frame",
        name: "Feature control frame",
        icon: "properties",
        action: None,
    },
    Feature {
        id: "drawing-datum",
        name: "Datum",
        icon: "text",
        action: None,
    },
    Feature {
        id: "drawing-revision-cloud",
        name: "Revision cloud",
        icon: "spline",
        action: None,
    },
];
const DRAWING_2: &[Feature] = &[
    Feature {
        id: "drawing-new-sheet",
        name: "New sheet",
        icon: "new",
        action: None,
    },
    Feature {
        id: "drawing-sheet-settings",
        name: "Sheet settings",
        icon: "drawing_settings",
        action: None,
    },
    Feature {
        id: "drawing-title-block",
        name: "Title block",
        icon: "create_block",
        action: None,
    },
    Feature {
        id: "drawing-parts-list",
        name: "Parts list",
        icon: "create_menu",
        action: None,
    },
    Feature {
        id: "drawing-balloon",
        name: "Balloon",
        icon: "circle",
        action: None,
    },
    Feature {
        id: "drawing-revision-table",
        name: "Revision table",
        icon: "properties",
        action: None,
    },
];
const DRAWING_3: &[Feature] = &[
    Feature {
        id: "drawing-export-drawing-pdf",
        name: "Export drawing PDF",
        icon: "export_pdf",
        action: None,
    },
    Feature {
        id: "drawing-export-drawing-dxf",
        name: "Export drawing DXF",
        icon: "export",
        action: None,
    },
    Feature {
        id: "drawing-print-drawing",
        name: "Print drawing",
        icon: "print",
        action: None,
    },
];
const DRAWING_4: &[Feature] = &[Feature {
    id: "drawing-select-drawing",
    name: "Select drawing",
    icon: "cursor",
    action: None,
}];
pub fn groups(mode: Mode) -> Vec<Group> {
    match mode {
        Mode::Solid => vec![
            Group {
                name: "Create",
                features: SOLID_0,
            },
            Group {
                name: "Modify",
                features: SOLID_1,
            },
            Group {
                name: "Components",
                features: SOLID_2,
            },
            Group {
                name: "Construct",
                features: SOLID_3,
            },
            Group {
                name: "Inspect",
                features: SOLID_4,
            },
            Group {
                name: "Insert",
                features: SOLID_5,
            },
            Group {
                name: "Select",
                features: SOLID_6,
            },
        ],
        Mode::Sketch => vec![
            Group {
                name: "Create",
                features: SKETCH_0,
            },
            Group {
                name: "Modify",
                features: SKETCH_1,
            },
            Group {
                name: "Constrain",
                features: SKETCH_2,
            },
            Group {
                name: "Inspect",
                features: SKETCH_3,
            },
            Group {
                name: "Finish",
                features: SKETCH_4,
            },
        ],
        Mode::Drawing => vec![
            Group {
                name: "Views",
                features: DRAWING_0,
            },
            Group {
                name: "Annotate",
                features: DRAWING_1,
            },
            Group {
                name: "Sheet",
                features: DRAWING_2,
            },
            Group {
                name: "Export",
                features: DRAWING_3,
            },
            Group {
                name: "Select",
                features: DRAWING_4,
            },
        ],
    }
}
