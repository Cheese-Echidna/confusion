# In-scope feature parity checklist

Target: a moderate Fusion user can carry out familiar sketch, solid, component and
drawing tasks without significant workflow loss. This is an acceptance plan, not
a claim that any listed tool exists today. Validate detailed options, default
shortcuts and reference behaviors against Autodesk primary documentation and
recorded user workflows during implementation.

## Sketches

- Planar sketch attachment to origin/construction planes and named planar faces;
  active component context; linked projection/intersection geometry.
- Points, polylines, rectangles, slots, circles, arcs, ellipses, conics, fit/control
  splines and shaped text outlines; construction geometry and local coordinate frames.
- Coincidence, horizontal/vertical, parallel, perpendicular, tangency, smoothness,
  equal, concentric, collinear, midpoint, symmetry and fix/unfix constraints.
- Driving/driven dimensions; length, angle, radius and diameter; named parameters,
  unit-aware expressions and parameter rename/reference safety.
- Trim, extend, break, offset, mirror, move/copy, sketch patterns and scale.
- Constraint inference, grid/object snapping, temporary drag objectives, stable
  solution branches and numeric entry from the canvas.
- Profiles with holes and nested regions; precise open/self-intersecting/overlapping
  curve diagnostics; editable profile references after parameter changes.
- Underconstrained/fully constrained/redundant/inconsistent feedback, DOF indicators,
  visual conflict location and repair without discarding unrelated geometry.

## Solids and parameters

- Extrusion/revolution with one/two-sided and symmetric extents, to-object limits,
  taper, join/cut/intersect/new-body/new-component options.
- Sweeps with path/orientation/guide options and lofts with sections/rails/continuity
  where needed to construct solids; thin-wall variants.
- Primitives, holes (simple/counterbore/countersink/tapped), threads, fillet/chamfer
  (including variable rules), shell, draft, rib/web and emboss/deboss.
- Body Boolean/combine, split body/face, press-pull, offset/move/replace/delete face
  with declarative parametric modifiers and repairable references.
- Construction planes/axes/points, including offset/angle/mid/tangent/through-reference
  variants; body/feature mirror and rectangular/circular/path patterns.
- User parameters, expressions, suppression and configurations; change values and
  reevaluate the complete graph without a saved edit timeline.
- Stable face/edge/profile associations through dimension changes, pattern count
  changes, split/merge topology and upstream feature edits; explicit ambiguity repair.
- Exact distance/angle/area/volume/inertia measurement; section, draft, curvature
  and thickness inspection appropriate to solid modeling.
- STEP/IGES source bodies and optional STL reference meshes; units, import diagnostics,
  STEP assembly export and watertight tolerance-controlled STL export.

## Components and assemblies

- Reusable component definitions, separate instances, nested component trees,
  body ownership and active-component editing.
- Visibility, isolate, selection by body/face/edge/component, grounding, suppression,
  physical material assignment and component properties/part numbers.
- Create components from bodies, instantiate/copy, move/reparent, local linked
  component snapshots and explicit offline source refresh.
- Joint origins; rigid/revolute/slider/cylindrical/pin-slot/planar/ball joints,
  as-built joints, rigid groups, limits and motion links.
- Joint drive and assembly motion preview, remaining DOF/conflict explanation,
  exact interference and contact/collision checking appropriate to assembly design.
- Assembly configurations, exploded view definitions, quantities and BOM derivation.

## Technical drawings

- Associative model/component/configuration references, sheets, standard/custom
  sizes, templates, title blocks and drawing properties.
- Base/projected/orthographic/auxiliary/detail/section and exploded assembly views;
  perspective/isometric representation, scale and alignment controls.
- Hidden/visible edges, centerlines/marks, section hatching and view clipping.
- Linear/angular/radial/diameter/ordinate dimensions, tolerances, leaders, notes,
  thread/hole callouts, datums, GD&T, weld and surface-finish symbols.
- ISO/ASME presentation rules, dimension styles, annotation editing and collision
  avoidance; associative anchor repair after topology changes.
- BOM/parts lists, balloons, hole tables and document-derived title block fields.
- Accurate vector PDF/DXF/SVG output, font handling, line weights, sheet scale
  and consistent results after save/reopen and parameter changes.

## Interaction and reliability

- Fusion-style default shortcuts and navigation with platform modifiers; all
  bindings configurable by mode/tool/focus context in the one settings JSON file.
- Fusion-inspired tool organization and selection workflows; Zed-style searchable
  commands, document tabs, panels and keyboard focus behavior.
- Two primary modes; tools support preview/accept/cancel, on-canvas manipulators,
  numeric expressions and predictable selection filters.
- In-memory grouped undo/redo; atomic current-state `.con` saves, autosave snapshots,
  crash recovery, schema migration and reference-safe load validation.
- Entire application works offline. No cloud features, accounts, telemetry upload
  or remote model dependency is required or planned.
- Large-document/assembly interaction, bounded multithreaded solving, cancellable
  evaluation, stale result rejection, GPU rendering and meaningful failure reports.
- Linux/macOS/Windows packaging, high-DPI behavior, keyboard accessibility and
  theme/contrast support verified against the selected GPUI backend.

## Representative acceptance designs

1. Fully constrained mounting bracket: expressions, projected holes, variable fillet,
   shell and pattern; resize and retain downstream face/profile references.
2. Multi-part hinge: reusable instances, rigid/revolute joints, limits and driven
   motion; resize a component and retain joint anchors and assembly interference results.
3. Housing: loft/sweep-based solids, rib/web and threaded holes; source body import,
   direct face edits and failure repair under parameter changes.
4. Drawing package: assembly front/isometric/section/detail views, BOM and balloons,
   GD&T and hole callouts; modify a model dimension, regenerate, export and reopen.
5. Persistence check: save each design, terminate the process, reopen with caches
   removed and regenerate using only current `.con` intent/assets. Verify the file
   contains no undo stack or command/edit history and settings remain in one JSON file.

Use a per-operation option matrix and recorded reference workflows to evaluate
completion. A successfully rendered extrusion alone does not meet this target.
