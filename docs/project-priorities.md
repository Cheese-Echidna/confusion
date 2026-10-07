# Fusion parity and usability priorities

Assessment on 2026-10-07, based on the running workflow implementation, tests,
feature inventory, and documented limits. Module names alone are not evidence of
implemented functionality: many architecture modules remain documentation-only.

## Current implementation batch

1. **Protect unsaved work.** Implement local background autosave of current-state
   documents and a recover/discard workflow after an unexpected exit. Cover
   multiple tabs, invalid snapshots, intentional close, and save cleanup. The
   existing save/close prompts do not provide crash recovery.
2. **Exchange exact solid geometry.** Implement STEP export of freshly evaluated
   OCCT solids, with explicit units and atomic destination replacement. Printing
   meshes do not provide the exact geometry needed by other CAD applications.
   STEP source-body import is a separate follow-up.
3. **Keep sketch edits parametric.** Repair trim/extend/break ownership, stable IDs,
   shared endpoints, driving/driven constraints, and failure atomicity. Circular
   helpers already exist; retaining usable downstream references and connected
   geometry is the acceptance requirement.

Three agents implement these tasks with separate file ownership. Existing
uncommitted export, expression, persistence-limit, and performance-review changes
are excluded from their assignments.

All three agents ran `gpt-6.1-sol` with low reasoning. Their allowed file sets
were disjoint:

- STEP: `build.rs`, `src/kernel/bridge.rs`, `src/kernel/native/occt.hpp`,
  `src/kernel/native/occt.cpp`, `src/kernel/native/step.inc`,
  `src/evaluation/solid.rs`, `src/exchange/step.rs`, `tests/step_export.rs`,
  `docs/step-export.md`.
- Recovery: `src/persistence/recovery.rs`, `src/platform/paths.rs`,
  `src/application/session.rs`, `src/ui/viewport.rs`, `tests/crash_recovery.rs`,
  `docs/crash-recovery.md`.
- Sketch edits: `src/sketch/edit.rs`, `src/sketch/entities.rs`,
  `src/ui/sketch_workflow.rs`, `tests/curved_sketch_editing.rs`,
  `docs/curved-sketch-editing.md`.

The coordinating agent connected STEP to the desktop export menu after the
recovery agent released `viewport.rs`, added `src/ui/step_exchange.rs` and the
toolbar action, and updated the overview documentation.

## Next substantial parity work

| Priority | Work | Why it matters | Acceptance workflow |
| --- | --- | --- | --- |
| High | Stable topology naming and explicit reference repair | Modify tools still rely on evaluated face ordinals; upstream topology changes can require face reselection. This also blocks reliable linked projections, joints, and drawing anchors. | Resize a patterned/filleted bracket, identify unchanged faces by provenance, and offer repair when a face splits or disappears. |
| High | Linked sketch projection and richer profile extraction | Linked projection is scaffolded; ellipse/spline profiles are rejected, and intersecting/branching contours cannot become selectable subregions. | Project a model edge into another sketch, resize the source, and extrude selected regions from typical intersecting sketch geometry. |
| High | STEP source-body import and persistent source assets | Receiving existing CAD geometry is as important as sending it. Import must survive reopening without a live native cache. | Import a unit-tagged STEP solid, apply parametric modifications, save, reopen, and export exact geometry. |
| High | Reusable components and assembly positioning | Component, joint, and kinematic modules are scaffolds; multi-body solids do not constitute an assembly workflow. | Create two instances of one component, move/ground them, add a limited revolute joint, resize the definition, and retain both instances. |
| High | Associative technical drawings | A Drawing workspace exists in the shell, but sheet/view/annotation/export modules are scaffolds. | Create a dimensioned front/isometric sheet from a model, change a dimension, regenerate the drawing, and export correctly scaled vectors. |
| Medium | Broader feature options | Solid tools exist, but individual-edge picking, variable fillets, arbitrary-axis movement, advanced sweep/loft controls, and other documented Fusion options remain limited. | Complete the representative bracket and housing designs in the parity checklist using editable feature inputs. |
| Medium | Parameter rename and configuration workflows | Expression editing exists; dedicated parameter/configuration modules remain scaffolds. | Rename a parameter without changing its ID or breaking formulas, switch a configuration, and rebuild the whole graph. |
| Medium | Packaging and desktop acceptance coverage | Native dependencies and GPU composition need platform-specific verification beyond headless tests. | Open, model, save, recover, and export on supported Linux/macOS/Windows packages with keyboard and high-DPI checks. |

These are follow-up priorities, not completion claims. See
[the full parity checklist](feature-parity.md) for representative designs and
[the Solid/Modify limits](solid-modify.md) for current operation scope.

Fusion reference workflows: [export designs](https://help.autodesk.com/view/fusion360/ENU/?contextId=ASM-EXPORT-DESIGN)
and [trim/extend sketch geometry](https://help.autodesk.com/cloudhelp/ENU/Fusion-Sketch/files/SKT-TRIM-EXTEND.htm).
