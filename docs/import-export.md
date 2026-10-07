# Import and export

Choose **Export → STEP** for exact solid geometry. The exporter evaluates the full
current design and writes flattened AP214 B-reps in millimetres on a background
worker. It preserves the previous destination if evaluation or writing fails.
STEP output excludes sketches, constraints, feature intent, materials and assembly
hierarchy. STEP document import remains unimplemented. See
[exact STEP export](step-export.md) for validation and current limits.

Choose **Export → STL (millimetres)** or **Export → 3MF** and select a destination.
The exporter evaluates the current complete design on a background worker before
writing. It does not export a stale viewport mesh or the timeline preview. Failed
evaluation, invalid solids, open meshes and inconsistent triangle winding stop the
export. A failed write preserves the previous destination.

STL is binary and contains millimetre coordinates. STL itself has no unit field;
choose millimetres in your slicer. 3MF explicitly records millimetres and packages
indexed geometry using the [3MF Core specification](https://github.com/3MFConsortium/spec_core).
Both formats contain tessellated geometry; neither preserves sketches, parameters,
constraints or construction features. All evaluated bodies are included in one
mesh object, without materials. The current exporter uses the kernel's existing
meshing tolerance, welds face seams at 1 nm, and has no quality selector yet.

## Editable Fusion transfer (limited support)

**Direct decoding of `.f3d` and `.f3z` archives is not implemented.** A Fusion-side
script reads the active design through Autodesk's API and writes an editable
`.fusion.json` transfer. This requires Autodesk Fusion on Windows or macOS. It does
not require an Autodesk connection from Confusion. Opening an archive directly in
Confusion explains this requirement without modifying the current document.

1. Open your archive in Fusion and make its design active.
2. Open Fusion's **Scripts and Add-Ins** dialog and add the repository's
   `tools/fusion/ConfusionTransfer` folder as a script.
3. Run **ConfusionTransfer**, then choose a `.fusion.json` destination.
4. In Confusion, choose **Open design** or **Solid → Insert → Import Fusion transfer**
   and select the transfer. Confusion validates its references, units and parameter
   expressions before creating a document tab.
5. Save the imported design as `.con`. The transfer stays unchanged; the imported
   design is unsaved and prompts for saving on close.

The converter currently supports a root component with sketches on its XY origin
plane, lines and full circles, construction curves, fixed points, horizontal and
vertical constraints, horizontal/vertical point constraints, point-to-point
coincidence, parallel/perpendicular/collinear lines, equal curves, concentric
circles, tangent curves and line midpoints. Dimensions include point-to-point
linear dimensions and circle radius/diameter. Driving and driven dimensions are
kept separately. Named user parameters and supported expression dependencies
remain editable, as do positive, untapered, one-sided new-body distance extrusions
from a single sketch profile. The supported expression language is Confusion's
arithmetic language with explicit units and parameter names.

Assemblies, arbitrary/face-attached planes, projections, fixed curves, arcs,
splines, sketch text, joins/cuts, patterns, fillets, timeline groups, suppressed
features and other unsupported construction abort conversion with a reason.
Expressions containing unsupported functions or Fusion's implicit unit arithmetic
may be rejected during import. The converter never substitutes evaluated parameter
values for unsupported formulas. This is a limited migration path and does not
provide general Fusion feature parity.

The script has headless adapter tests; it has **not yet been validated inside a
live Autodesk Fusion installation**. A real archive conversion is still needed to
validate the integration. The relevant API contracts are Autodesk's
[Sketch](https://help.autodesk.com/cloudhelp/ENU/Fusion-360-API/files/fusion_Sketch.htm),
[SketchLinearDimension](https://help.autodesk.com/cloudhelp/ENU/Fusion-360-API/files/fusion_SketchLinearDimension.htm)
and [ExtrudeFeature](https://help.autodesk.com/cloudhelp/ENU/Fusion-360-API/files/fusion_ExtrudeFeature.htm).

Checks:

```sh
cargo test --test printing_export --test fusion_transfer
python3 tools/fusion/test_transfer.py
nix-shell --run 'cargo test --features solver,kernel --test printing_export --test fusion_transfer'
nix-shell --run 'cargo check --features desktop'
```
