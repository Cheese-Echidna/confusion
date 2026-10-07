# Exact STEP export

`confusion::exchange::step::export_design(path, &design)` exports the complete
current design as flattened AP214 STEP solids. It returns `StepExportReport` with
`solids` and `volume_m3`. The solver and kernel Cargo features are required;
headless builds return an explicit unsupported-feature error.

Export validates the document and parameters, solves sketches, and evaluates the
whole feature history with a fresh owning cache. It retains the final native
OCCT compound after create features, booleans and solid edits, including pattern
instances and cut-created bodies. Consumed bodies are excluded. The STEP writer
transfers these exact B-reps directly; viewport triangles are never used to
reconstruct geometry. Evaluation currently also produces its usual display mesh.

Design coordinates are SI meters. The kernel converts them to millimeters during
construction. The STEP writer explicitly uses millimeter model and file units,
so export performs no additional scale conversion. Report volume remains in
cubic meters. The OCCT 7.9.3 integration has native readback tests for curved
solids, booleans, patterns and final scale edits, checking validity, solid counts,
physical volumes and the millimeter file unit declaration.

An export with no solids, invalid document/parameters, sketch conflicts, a failed
feature, or failed file operation returns an error. A temporary file in the
same destination directory is written and synced before atomic replacement;
failures preserve the previous destination and clean up the temporary file.
Parent directories must already exist. Native temporary paths must be UTF-8.
Atomic replacement does not promise directory-entry durability across power loss.

This exports geometry, not Confusion feature history, sketch constraints or
parameters. Assembly hierarchy, component names, body appearance and AP242
metadata are not implemented or promised. `ffi::inspect_step` is a native
readback helper for validation, not a document import workflow. Compatibility is
verified against OCCT readback; interactive Autodesk Fusion import has not been
tested. Autodesk describes supported design export formats in its
[export documentation](https://help.autodesk.com/view/fusion360/ENU/?contextId=ASM-EXPORT-DESIGN).

Run the focused checks with:

```sh
nix-shell --run 'cargo test --features solver,kernel --test step_export'
cargo test --test step_export
```
