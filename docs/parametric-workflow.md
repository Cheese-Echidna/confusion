# Sketch workflow

Run `nix-shell --run 'cargo run --features desktop --locked'`, then choose **Create sketch** to enter the XY sketch workspace.

- **L**, **R**, and **C** create line chains, rectangles, and center circles. Creation menus also offer center rectangles, arcs, ellipses, polygons, slots, fit splines, and points. Escape ends the current tool.
- Select points or curves; Shift adds to the selection. Drag endpoints or selected geometry to reshape it while preserving constraints. Left-to-right box selection contains geometry; right-to-left selection crosses it. Ctrl+A selects all sketch geometry.
- **D** creates a dimension from the selection. Place its annotation, enter a value or expression, and press Enter. Bare values use millimetres or degrees. Two points support horizontal, vertical, or aligned distances; lines support length, separation, and angle; circles and arcs support diameter and radius. Annotations can be moved and existing dimensions switched between driving and driven.
- Constraint menus provide coincidence, horizontal/vertical, parallel, perpendicular, equal, collinear, midpoint, point-on-curve, concentric, tangent, symmetry, and Fix/Unfix. Constraint icons can be selected and deleted. Available relations depend on the geometry selected.
- Free geometry is blue, constrained geometry uses the light foreground color of the dark theme, fixed geometry is green, and conflicting geometry is red. The footer reports remaining degrees of freedom and local redundancy.
- **I** measures selected geometry. **T** trims, **O** offsets, **F** fillets, **M** moves, and **X** toggles construction. Modify menus include break, extend, mirror, scale, and rectangular/circular copies. Offsets remain parametrically linked to their source geometry.
- Hold Ctrl/Cmd while placing geometry to suppress snapping and inferred constraints. Middle/right drag pans; the wheel zooms. **S** searches tools. Ctrl+Z and Ctrl+Shift+Z undo and redo.

Parameters support named dependencies, length and angle units, arithmetic, integer powers, square roots, and trigonometric functions. Invalid units, cycles, and conflicting dimensions produce errors. Editing uses candidate documents so failed edits do not corrupt the saved intent.

## Complex acceptance sketch

Open [complex-sketch.con](assets/complex-sketch.con): an 80 × 50 mm plate, four Ø6 mm holes positioned by margin expressions, and a centered slot. It has zero remaining degrees of freedom. Change `width` to `100 mm` in Parameters; the plate, hole positions, and slot regenerate.

Rebuild and validate the saved example with:

```sh
nix-shell --run 'cargo run --features solver --example sketch_workflow --locked -- docs/assets/complex-sketch.con'
```

## Configurable shortcuts

The existing single settings JSON file accepts command-to-chord overrides. Restart to load changes. A null value disables a command's shortcut. Modeling shortcuts are suppressed when a text field has focus.

```json
{
  "keybindings": {
    "sketch": { "sketch-line": "shift-l", "sketch-circle": null },
    "global": { "undo": "ctrl-z" }
  }
}
```

## Implementation and current limits

The solver uses damped least-squares QR, analytic and numerical Jacobians, and SVD to report local point freedom. Drag targets are temporary soft objectives; permanent constraints take precedence. Redundancy is a local rank diagnostic, not a minimal conflict explanation. Curves are sampled for display and some measurements are approximate.

Version 4 `.con` files preserve geometry, constraints, expressions, construction flags, annotations, and driven dimensions; older versions remain readable. Undo history and derived meshes are not serialized.

This is not complete Fusion parity. Sketches currently use a single XY plane. Projection, sketch text, general conics, curvature continuity, complete spline/ellipse constraints, associative editable pattern definitions, and a graphical shortcut editor remain unfinished. Patterns currently create copies with internal constraints and shared parameter references. Sketch documents are bounded to 128 points, 128 curves, 256 constraints, and 128 parameters.

Exact OCCT extrusion accepts closed regions made of lines, circles, and circular arcs, including nested holes and slots. Press **E** with no selection to extrude the single outer region. For multiple outlines, select one boundary curve and press **E**; select a hole boundary to extrude its interior instead. An existing extrusion retains its selected boundary when edited without a selection.

Extrusion stores the selected outer boundary's curve IDs in version 4 `.con` files. Parameter edits and save/reopen retain that region; deleting or replacing its boundary requires explicit reselection. Earlier file versions remain readable and use automatic selection. Open, branching, intersecting, or touching contours are rejected. Ellipses and splines must be construction geometry for this extrusion workflow.

The complex acceptance sketch can now be extruded as a plate with four circular holes and the center slot. Width changes regenerate the exact solid, including hole positions and slot length. Region classification uses sampled curves; the native solid uses exact OCCT lines and circular arcs and is checked for validity.
