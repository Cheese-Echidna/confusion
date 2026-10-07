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

Version 5 `.con` files preserve geometry, constraints, expressions, construction flags, annotations, and driven dimensions; older versions remain readable. Undo history and derived meshes are not serialized.

This is not complete Fusion parity. Sketches use XY or planar extrusion caps; arbitrary face/edge attachments and general construction planes remain unfinished. Projection, sketch text, general conics, curvature continuity, complete spline/ellipse constraints, associative editable pattern definitions, and a graphical shortcut editor remain unfinished. Patterns currently create copies with internal constraints and shared parameter references. Each sketch is bounded to 128 points, 128 curves, and 256 constraints. A design supports up to 64 sketches, 64 solid features, and 128 shared parameters.

Exact OCCT extrusion accepts regions bounded by lines, circles, circular arcs, ellipses, and fit/control splines, including nested holes and slots. Crossings and shared edges split the sketch into bounded faces. Click inside the desired face to select and shade it, then press **E**. Points and edges retain picking priority. Press **E** with no selection to extrude the single outer region; an edge selection works when it identifies only one region. An existing extrusion retains its selected boundary when edited without a selection.

Extrusion stores source curve IDs for unsplit boundaries and deterministic segment tokens for split boundaries in `.con` files. Tokens include the source curve and crossing partners; replacing a boundary or changing its intersection topology requires explicit reselection. Parameter edits that retain that topology and save/reopen retain the region. Open branches do not invalidate other closed faces. Dividing an inner contour preserves a single hole in the surrounding face. Desktop intersections and solid edges use exact OCCT curves; polygon samples serve display and region containment only. Kernel-free headless queries use numerical refinement for ellipse/spline intersections.

The complex acceptance sketch can now be extruded as a plate with four circular holes and the center slot. Width changes regenerate the exact solid, including hole positions and slot length. Region classification uses sampled curves; the native solid uses exact OCCT lines and circular arcs and is checked for validity.


## Multiple sketches and solid features

1. Create and constrain the base sketch, then press **E** and apply **New body**.
2. In Solid mode, select its top or bottom planar cap and choose **Create sketch**. With no face selected, a new sketch uses XY. Side faces and curved faces are not supported yet.
3. Draw a closed pocket profile and press **E**. Face sketches default to **Cut**, directed inward from the selected cap. **Join** adds material outward; **New body** creates a separate solid. Join/Cut list available target bodies in the extrusion editor.
4. Click a sketch in the browser or timeline to edit it. Click a solid feature to edit its depth and operation. Return to the end of the timeline to regenerate all downstream features.

Open [plate-pocket.con](assets/plate-pocket.con) for an 80 × 50 × 10 mm plate with a face-attached 20 × 10 × 3 mm pocket. Change `width` or `thickness` in Parameters; the pocket plane follows the plate's top cap. The source sketch geometry remains in its own local coordinates. Regenerate the example with:

```sh
nix-shell --run 'cargo run --features solver,kernel --example model_workflow --locked -- docs/assets/plate-pocket.con'
```

Version 5 stores separate sketch geometry, plane dependencies, feature IDs, target bodies, regions and depth parameters. The active sketch occupies the original geometry fields for compatibility; inactive sketches each have one stored geometry definition. Parameters are shared across sketches. Versions 1–4 remain readable and retain their original sketch and extrusion.

Face anchors store the support feature, producing feature and Start/End cap role. OCCT Boolean history propagates those roles through joins and cuts. Deleted or split supports produce an explicit reattachment error, and failed features do not publish a successful mesh. This is the initial topology-reference implementation; arbitrary face naming and a graphical reattachment editor remain unfinished. No-op Booleans and operations producing multiple disconnected solids are rejected. New bodies remain independent, and explicit targets must refer to an unconsumed upstream body. Parameter/expression and feature dependency cycles are rejected.
