# Native user interface

The GPUI shell follows the supplied Fusion references, using Zed's Gruvbox Dark
palette. The supplied LibreCAD CC0 SVGs are embedded so launching from another
working directory works. Runtime color adaptation changes their existing color
slots for contrast on the dark palette; SVG geometry and source assets are retained.
Future tools reuse those icons as placeholders.

## Layout and controls

- Top left: Open, New, Export, Save, Undo and Redo. Open and initial Save show a local
  path editor; confirm using its icon. Export lists STEP and STL as not implemented.
- Solid, Sketch and Drawing tabs choose the tool context. The ribbon shows compact
  tool groups; each group menu exposes its complete named tool list. Hover tooltips
  identify icons and unsupported tools. Unsupported commands also report their status
  when clicked. Drawing currently displays a sheet placeholder.
- The tree includes collapsible Construction, Sketches, Bodies, Components, Origin
  and Analysis categories. Components are organizational collections of separate
  bodies; their creation and editing tools are placeholders pending the backend.
- The upper right has the orientation cube, Home, projection, Fit, Grid, Snap and view
  options. Click a visible cube face or right-click it for all six cardinal views.
  Sketch remains on XY. Middle drag pans, Shift-middle drag uses constrained orbit,
  and Ctrl-middle drag uses free orbit. Wheel zooms. Grid snapping uses a 1 mm step.
- Contextual panels contain document paths, current parameters, extrusion depth,
  dimensions or view options. Escape closes panels and menus.

The catalog covers sketch creation and constraints; solid creation, modification,
components, construction and inspection; and drawing views, annotation and sheets.
Only the existing line/rectangle, supported constraints, parameter and extrusion
workflow is active. Unimplemented features remain visible and clearly identified.

## Construction versus edits

The bottom timeline is **persistent construction**, separate from undo and redo.
For the supported backend it contains Sketch 1 followed by Extrude 1. Feature IDs
remain stable when parameters change. Extrude references its source sketch by ID.
Clicking a sketch edits its parameters and evaluates the construction through that
sketch; downstream extrusion intent remains in the canonical design. Restoring the
end regenerates the extrusion using the current parameters. Clicking the extrusion
opens its depth editor. Navigation buttons move the evaluation marker.

Saving writes current feature definitions and dependencies to version 2 `.con`.
Version 1 files load with a synthesized construction sequence. No previous parameter
values, undo stack or derived meshes are saved. Undo/redo hold session snapshots in
memory. A future multi-feature evaluator must extend the same dependency model;
patterns and multiple sketches are not yet supported by the backend.

## Modules and checks

`ui/viewport.rs` owns interactions and composition; `toolbar.rs` owns the feature
catalog; `components.rs`, `theme.rs` and `assets.rs` provide shared controls and styling.
`sketch_canvas.rs` paints the sketch and `view_cube.rs` supplies camera-linked cube
geometry/picking. `render/grid.rs` paints the depth-tested model grid.

Validation uses solver/kernel integration tests, cardinal camera and cube-picking
tests, desktop Clippy, the GPU renderer/picker smoke example, and native X11/Xvfb
interaction checks. Screenshots in `docs/assets/ui-*.png` show the actual GPUI window.

The native verification created a 40 × 20 × 10 mm extrusion, edited width through
the sketch timeline entry to 60 mm, and observed volume change from 8,000 to
12,000 mm³. Saving and reopening retained the same two construction entries.
Projection, cube selection, constrained/free orbit and a 900 × 650 window were
also exercised. Automated checks passed 18 tests, desktop Clippy and the GPU smoke
check on an NVIDIA RTX 3080 Ti.
