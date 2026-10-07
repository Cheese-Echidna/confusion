# Solid / Modify

All entries in Solid → Modify have working actions. Open a tool, choose its target
body, select a face when requested, enter values, and Apply (or Enter). Failed
geometry stays in the inspector without changing the document. Successful edits
support undo/redo, `.con` persistence, construction-marker previews, and reopening
from the timeline. Distance and angle inputs accept units and parameter expressions;
scale factors and cutting-plane normals accept scalar expressions.

| Tool | Controls and current scope |
| --- | --- |
| Press pull / Offset face | Select a planar face and enter a signed normal displacement. Positive adds material; negative cuts material. |
| Fillet / Chamfer | Radius/distance on all boundary edges of the selected face; clear face selection for all body edges. |
| Shell | Select the opening face and enter an inward wall thickness. |
| Draft | Select a side face, enter an angle and neutral-plane Z. Pull direction is global +Z. |
| Scale | Uniform factor around an XYZ center; optionally retain the original. |
| Combine | Choose different target/tool bodies, then Join, Cut, or Intersect. Optionally keep the tool. |
| Replace face | Select a planar face and enter the ordinal of a parallel planar face on a different tool body. The target extends/trims to its plane. |
| Split body | Cutting plane defined by signed distance and XYZ normal. Both resulting portions remain in the target body group. |
| Split face | Select a face and define the cutting plane. The section subdivides that face without changing volume. |
| Silhouette split | Project a different tool body's visible outline onto a selected planar face along that face's normal. |
| Move / copy | XYZ translation and rotation around global Z; optionally retain the original. |
| Align | Translate the target center of mass to the tool center of mass; optionally retain the original. |
| Delete / Remove | Clear face selection to remove the body from evaluation. Select a feature face to remove/heal its geometry. Blind-pocket/boss caps also try adjoining feature walls when needed. |
| Simplify | Merge same-domain faces and edges for a whole body, or remove/heal a selected feature. |
| Physical material | Assign a name, density in kg/m³, and material color to a body. Single-body documents show mass from exact volume. |
| Appearance | Change a body's RGB display color while retaining an existing material/density assignment. |
| Manage materials | Create/update document materials by name, reload them, and remove saved entries. Aluminium, steel, ABS, and copper presets are available. |
| Change parameters | Edit document expressions, including the parameters created for Modify tools. |

Edits evaluate after extrusion and Solid/Create features, in their stored order.
Copies and split portions remain grouped under their source body. Face references
now persist feature/boundary provenance names. Native Boolean, fillet, shell, draft,
transform and split history carries those names through regeneration. Deleted or
ambiguous split/merged faces report an explicit repair error. Reopen the Modify
feature from the timeline to view the model immediately before it, select the
replacement face, and Apply. Plane splits expose positive/negative branch names
for that repair. Legacy ordinals are upgraded to names after successful desktop
evaluation when their provenance is unique. Operations without a face input do
not create a face dependency. Generic generated-face naming is conservative: faces
with indistinguishable provenance remain ambiguous. These controls do not yet offer individual-edge picking,
variable-radius fillets, arbitrary-axis movement, face-to-face alignment, curved
face replacement, or general surface deformation. Material density is document
metadata; appearance uses a solid color without texture or reflectance controls.

Validation:

```sh
cargo test --test solid_modify
nix-shell --run 'cargo test --features solver,kernel --test solid_modify'
nix-shell --run 'cargo check --features desktop'
```

Apply validates geometry in the background and shows **Validating…**. Success
commits one undo step; failure keeps the editor and current design intact. See
[background Solid Apply](async-solid-apply.md) for cancellation and validation.
