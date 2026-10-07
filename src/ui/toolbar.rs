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
    Export(crate::exchange::export::ExportFormat),
    ExportStep,
    ImportFusion,
    SolidCreate(crate::model::solid_create::CreateKind),
    SolidModify(crate::model::modify::ModifyKind),
    PhysicalMaterial,
    Appearance,
    ManageMaterials,
    Sketch,
    Extrude,
    Parameters,
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
    Move,
    Mirror,
    Scale,
    RectangularPattern,
    CircularPattern,
    Fillet,
    CenterArc,
    Point,
    Construction,
    Delete,
    Break,
    Trim,
    Extend,
    Offset,
    Measure,
    SectionAnalysis,
    Interference,
    CenterOfMass,
    CurvatureAnalysis,
    DraftAnalysis,
    ValidateSolid,
    Coincident,
    Parallel,
    Perpendicular,
    Equal,
    Collinear,
    Concentric,
    Tangent,
    Midpoint,
    Symmetry,
    Horizontal,
    Vertical,
    Fixed,
    Dimension,
    Finish,
    ViewFront,
    ViewBack,
    ViewLeft,
    ViewRight,
    ViewTop,
    ViewBottom,
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
        icon: "Sketcher_NewSketch",
        action: Some(Action::Sketch),
    },
    Feature {
        id: "solid-extrude",
        name: "Extrude",
        icon: "PartDesign_Pad",
        action: Some(Action::Extrude),
    },
    Feature {
        id: "solid-revolve",
        name: "Revolve",
        icon: "PartDesign_Revolution",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Revolve,
        )),
    },
    Feature {
        id: "solid-sweep",
        name: "Sweep",
        icon: "PartDesign_AdditivePipe",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Sweep,
        )),
    },
    Feature {
        id: "solid-loft",
        name: "Loft",
        icon: "PartDesign_AdditiveLoft",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Loft,
        )),
    },
    Feature {
        id: "solid-rib",
        name: "Rib",
        icon: "line_perpendicular",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Rib,
        )),
    },
    Feature {
        id: "solid-web",
        name: "Web",
        icon: "line_parallel",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Web,
        )),
    },
    Feature {
        id: "solid-emboss",
        name: "Emboss",
        icon: "text",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Emboss,
        )),
    },
    Feature {
        id: "solid-hole",
        name: "Hole",
        icon: "PartDesign_Hole",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Hole,
        )),
    },
    Feature {
        id: "solid-thread",
        name: "Thread",
        icon: "spline_points",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Thread,
        )),
    },
    Feature {
        id: "solid-box",
        name: "Box",
        icon: "PartDesign_AdditiveBox",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Box,
        )),
    },
    Feature {
        id: "solid-cylinder",
        name: "Cylinder",
        icon: "PartDesign_AdditiveCylinder",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Cylinder,
        )),
    },
    Feature {
        id: "solid-sphere",
        name: "Sphere",
        icon: "PartDesign_AdditiveSphere",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Sphere,
        )),
    },
    Feature {
        id: "solid-torus",
        name: "Torus",
        icon: "PartDesign_AdditiveTorus",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Torus,
        )),
    },
    Feature {
        id: "solid-coil",
        name: "Coil",
        icon: "PartDesign_AdditiveHelix",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Coil,
        )),
    },
    Feature {
        id: "solid-pipe",
        name: "Pipe",
        icon: "PartDesign_AdditivePipe",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Pipe,
        )),
    },
    Feature {
        id: "solid-rectangular-pattern",
        name: "Rectangular pattern",
        icon: "PartDesign_LinearPattern",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::RectangularPattern,
        )),
    },
    Feature {
        id: "solid-circular-pattern",
        name: "Circular pattern",
        icon: "PartDesign_PolarPattern",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::CircularPattern,
        )),
    },
    Feature {
        id: "solid-pattern-on-path",
        name: "Pattern on path",
        icon: "spline",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::PatternOnPath,
        )),
    },
    Feature {
        id: "solid-mirror",
        name: "Mirror",
        icon: "PartDesign_Mirrored",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Mirror,
        )),
    },
    Feature {
        id: "solid-thicken",
        name: "Thicken",
        icon: "Part_Thickness",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Thicken,
        )),
    },
    Feature {
        id: "solid-boundary-fill",
        name: "Boundary fill",
        icon: "hatch",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::BoundaryFill,
        )),
    },
    Feature {
        id: "solid-gear",
        name: "Gear",
        icon: "circle",
        action: Some(Action::SolidCreate(
            crate::model::solid_create::CreateKind::Gear,
        )),
    },
];
const SOLID_1: &[Feature] = &[
    Feature {
        id: "solid-press-pull",
        name: "Press pull",
        icon: "Part_Offset",
        action: Some(Action::SolidModify(
            crate::model::modify::ModifyKind::PressPull,
        )),
    },
    Feature {
        id: "solid-fillet",
        name: "Fillet",
        icon: "PartDesign_Fillet",
        action: Some(Action::SolidModify(
            crate::model::modify::ModifyKind::Fillet,
        )),
    },
    Feature {
        id: "solid-chamfer",
        name: "Chamfer",
        icon: "PartDesign_Chamfer",
        action: Some(Action::SolidModify(
            crate::model::modify::ModifyKind::Chamfer,
        )),
    },
    Feature {
        id: "solid-shell",
        name: "Shell",
        icon: "PartDesign_Thickness",
        action: Some(Action::SolidModify(crate::model::modify::ModifyKind::Shell)),
    },
    Feature {
        id: "solid-draft",
        name: "Draft",
        icon: "draft",
        action: Some(Action::SolidModify(crate::model::modify::ModifyKind::Draft)),
    },
    Feature {
        id: "solid-scale",
        name: "Scale",
        icon: "scale",
        action: Some(Action::SolidModify(crate::model::modify::ModifyKind::Scale)),
    },
    Feature {
        id: "solid-combine",
        name: "Combine",
        icon: "PartDesign_Boolean",
        action: Some(Action::SolidModify(
            crate::model::modify::ModifyKind::Combine,
        )),
    },
    Feature {
        id: "solid-offset-face",
        name: "Offset face",
        icon: "Part_Offset",
        action: Some(Action::SolidModify(
            crate::model::modify::ModifyKind::OffsetFace,
        )),
    },
    Feature {
        id: "solid-replace-face",
        name: "Replace face",
        icon: "Part_Shapebuilder",
        action: Some(Action::SolidModify(
            crate::model::modify::ModifyKind::ReplaceFace,
        )),
    },
    Feature {
        id: "solid-split-body",
        name: "Split body",
        icon: "Part_SliceApart",
        action: Some(Action::SolidModify(
            crate::model::modify::ModifyKind::SplitBody,
        )),
    },
    Feature {
        id: "solid-split-face",
        name: "Split face",
        icon: "Part_Slice",
        action: Some(Action::SolidModify(
            crate::model::modify::ModifyKind::SplitFace,
        )),
    },
    Feature {
        id: "solid-silhouette-split",
        name: "Silhouette split",
        icon: "Part_Section",
        action: Some(Action::SolidModify(
            crate::model::modify::ModifyKind::SilhouetteSplit,
        )),
    },
    Feature {
        id: "solid-move-copy",
        name: "Move / copy",
        icon: "move_copy",
        action: Some(Action::SolidModify(
            crate::model::modify::ModifyKind::MoveCopy,
        )),
    },
    Feature {
        id: "solid-align",
        name: "Align",
        icon: "PartDesign_CoordinateSystem",
        action: Some(Action::SolidModify(crate::model::modify::ModifyKind::Align)),
    },
    Feature {
        id: "solid-delete",
        name: "Delete",
        icon: "delete",
        action: Some(Action::SolidModify(
            crate::model::modify::ModifyKind::Delete,
        )),
    },
    Feature {
        id: "solid-remove",
        name: "Remove",
        icon: "remove",
        action: Some(Action::SolidModify(
            crate::model::modify::ModifyKind::Remove,
        )),
    },
    Feature {
        id: "solid-simplify",
        name: "Simplify",
        icon: "Part_Defeaturing",
        action: Some(Action::SolidModify(
            crate::model::modify::ModifyKind::Simplify,
        )),
    },
    Feature {
        id: "solid-physical-material",
        name: "Physical material",
        icon: "hatch",
        action: Some(Action::PhysicalMaterial),
    },
    Feature {
        id: "solid-appearance",
        name: "Appearance",
        icon: "attributes",
        action: Some(Action::Appearance),
    },
    Feature {
        id: "solid-manage-materials",
        name: "Manage materials",
        icon: "properties",
        action: Some(Action::ManageMaterials),
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
        icon: "Part_Compound",
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
        icon: "Group",
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
        icon: "PartDesign_Plane",
        action: None,
    },
    Feature {
        id: "solid-plane-at-angle",
        name: "Plane at angle",
        icon: "PartDesign_Plane",
        action: None,
    },
    Feature {
        id: "solid-tangent-plane",
        name: "Tangent plane",
        icon: "PartDesign_Plane",
        action: None,
    },
    Feature {
        id: "solid-midplane",
        name: "Midplane",
        icon: "PartDesign_Plane",
        action: None,
    },
    Feature {
        id: "solid-perpendicular-plane",
        name: "Perpendicular plane",
        icon: "PartDesign_Plane",
        action: None,
    },
    Feature {
        id: "solid-plane-through-two-edges",
        name: "Plane through two edges",
        icon: "PartDesign_Plane",
        action: None,
    },
    Feature {
        id: "solid-plane-through-three-points",
        name: "Plane through three points",
        icon: "PartDesign_Plane",
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
        action: Some(Action::Measure),
    },
    Feature {
        id: "solid-section-analysis",
        name: "Section analysis",
        icon: "Part_CrossSections",
        action: Some(Action::SectionAnalysis),
    },
    Feature {
        id: "solid-interference",
        name: "Interference",
        icon: "Part_Common",
        action: Some(Action::Interference),
    },
    Feature {
        id: "solid-center-of-mass",
        name: "Center of mass",
        icon: "snap_center",
        action: Some(Action::CenterOfMass),
    },
    Feature {
        id: "solid-curvature-analysis",
        name: "Curvature analysis",
        icon: "spline",
        action: Some(Action::CurvatureAnalysis),
    },
    Feature {
        id: "solid-draft-analysis",
        name: "Draft analysis",
        icon: "draft",
        action: Some(Action::DraftAnalysis),
    },
    Feature {
        id: "solid-validate-solid",
        name: "Validate solid",
        icon: "Part_CheckGeometry",
        action: Some(Action::ValidateSolid),
    },
];
const SOLID_5: &[Feature] = &[
    Feature {
        id: "solid-import-fusion",
        name: "Import Fusion transfer",
        icon: "import",
        action: Some(Action::ImportFusion),
    },
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
        icon: "Sketcher_CreateLine",
        action: Some(Action::Line),
    },
    Feature {
        id: "sketch-rectangle",
        name: "Rectangle",
        icon: "Sketcher_CreateRectangle",
        action: Some(Action::Rectangle),
    },
    Feature {
        id: "sketch-center-rectangle",
        name: "Center rectangle",
        icon: "Sketcher_CreateRectangle_Center",
        action: Some(Action::CenterRectangle),
    },
    Feature {
        id: "sketch-circle",
        name: "Circle",
        icon: "Sketcher_CreateCircle",
        action: Some(Action::Circle),
    },
    Feature {
        id: "sketch-two-point-circle",
        name: "Two-point circle",
        icon: "Sketcher_CreateCircle",
        action: Some(Action::Circle2),
    },
    Feature {
        id: "sketch-three-point-circle",
        name: "Three-point circle",
        icon: "Sketcher_Create3PointCircle",
        action: Some(Action::Circle3),
    },
    Feature {
        id: "sketch-arc",
        name: "Arc",
        icon: "Sketcher_Create3PointArc",
        action: Some(Action::Arc3),
    },
    Feature {
        id: "sketch-center-arc",
        name: "Center arc",
        icon: "Sketcher_CreateArc",
        action: Some(Action::CenterArc),
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
        icon: "Sketcher_CreateEllipse",
        action: Some(Action::Ellipse),
    },
    Feature {
        id: "sketch-polygon",
        name: "Polygon",
        icon: "Sketcher_CreateRegularPolygon",
        action: Some(Action::Polygon),
    },
    Feature {
        id: "sketch-slot",
        name: "Slot",
        icon: "Sketcher_CreateSlot",
        action: Some(Action::Slot),
    },
    Feature {
        id: "sketch-spline",
        name: "Spline",
        icon: "Sketcher_CreateBSpline",
        action: Some(Action::Spline),
    },
    Feature {
        id: "sketch-point",
        name: "Point",
        icon: "Sketcher_CreatePoint",
        action: Some(Action::Point),
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
        id: "sketch-construction",
        name: "Normal / Construction",
        icon: "Sketcher_RenderingOrder_Construction",
        action: Some(Action::Construction),
    },
    Feature {
        id: "sketch-sketch-fillet",
        name: "Sketch fillet",
        icon: "Sketcher_CreateFillet",
        action: Some(Action::Fillet),
    },
    Feature {
        id: "sketch-trim",
        name: "Trim",
        icon: "Sketcher_Trimming",
        action: Some(Action::Trim),
    },
    Feature {
        id: "sketch-extend",
        name: "Extend",
        icon: "Sketcher_Extend",
        action: Some(Action::Extend),
    },
    Feature {
        id: "sketch-break",
        name: "Break",
        icon: "Sketcher_Split",
        action: Some(Action::Break),
    },
    Feature {
        id: "sketch-offset",
        name: "Offset",
        icon: "Part_Offset2D",
        action: Some(Action::Offset),
    },
    Feature {
        id: "sketch-move-copy-sketch",
        name: "Move / copy sketch",
        icon: "PartDesign_MoveFeature",
        action: Some(Action::Move),
    },
    Feature {
        id: "sketch-mirror-sketch",
        name: "Mirror sketch",
        icon: "PartDesign_Mirrored",
        action: Some(Action::Mirror),
    },
    Feature {
        id: "sketch-rectangular-sketch-pattern",
        name: "Rectangular sketch pattern",
        icon: "PartDesign_LinearPattern",
        action: Some(Action::RectangularPattern),
    },
    Feature {
        id: "sketch-circular-sketch-pattern",
        name: "Circular sketch pattern",
        icon: "PartDesign_PolarPattern",
        action: Some(Action::CircularPattern),
    },
    Feature {
        id: "sketch-scale-sketch",
        name: "Scale sketch",
        icon: "PartDesign_Scaled",
        action: Some(Action::Scale),
    },
    Feature {
        id: "sketch-delete-geometry",
        name: "Delete geometry",
        icon: "Sketcher_DeleteGeometry",
        action: Some(Action::Delete),
    },
];
const SKETCH_2: &[Feature] = &[
    Feature {
        id: "sketch-horizontal",
        name: "Horizontal",
        icon: "Constraint_Horizontal",
        action: Some(Action::Horizontal),
    },
    Feature {
        id: "sketch-vertical",
        name: "Vertical",
        icon: "Constraint_Vertical",
        action: Some(Action::Vertical),
    },
    Feature {
        id: "sketch-fix-point",
        name: "Fix / Unfix",
        icon: "Constraint_Block",
        action: Some(Action::Fixed),
    },
    Feature {
        id: "sketch-coincident",
        name: "Coincident",
        icon: "Constraint_PointOnPoint",
        action: Some(Action::Coincident),
    },
    Feature {
        id: "sketch-collinear",
        name: "Collinear",
        icon: "Constraint_PointOnObject",
        action: Some(Action::Collinear),
    },
    Feature {
        id: "sketch-concentric",
        name: "Concentric",
        icon: "Constraint_PointOnPoint",
        action: Some(Action::Concentric),
    },
    Feature {
        id: "sketch-parallel",
        name: "Parallel",
        icon: "Constraint_Parallel",
        action: Some(Action::Parallel),
    },
    Feature {
        id: "sketch-perpendicular",
        name: "Perpendicular",
        icon: "Constraint_Perpendicular",
        action: Some(Action::Perpendicular),
    },
    Feature {
        id: "sketch-tangent",
        name: "Tangent",
        icon: "Constraint_Tangent",
        action: Some(Action::Tangent),
    },
    Feature {
        id: "sketch-equal",
        name: "Equal",
        icon: "Constraint_EqualLength",
        action: Some(Action::Equal),
    },
    Feature {
        id: "sketch-symmetry",
        name: "Symmetry",
        icon: "Constraint_Symmetric",
        action: Some(Action::Symmetry),
    },
    Feature {
        id: "sketch-midpoint",
        name: "Midpoint",
        icon: "Constraint_PointOnObject",
        action: Some(Action::Midpoint),
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
        action: Some(Action::Measure),
    },
];
const SKETCH_4: &[Feature] = &[Feature {
    id: "sketch-finish-sketch",
    name: "Finish sketch",
    icon: "Sketcher_LeaveSketch",
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
