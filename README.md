This is confusion; a fully parametric constraint-based 3D modeling software built on GPUI.rs.
It is heavily inspired by Autodesk Fusion.
It has two main modes sketch mode and model mode files are saved with the .con extension in a parametric fashion and do not contain edit history but are still fully parametric.
Export exists to step and stl files.
The user interface is heavily inspired by Fusion and where applicable also the Zed IDE.
There are configurable context-depended keyboard shortcuts. The default for those shortcuts is based on fusion.
All settings for the application are in a single JSON file.
Model rendering is GPU based and contraint solving is multithreaded.


The scope is sketches, solids, components/assemblies and technical drawings. The
application is entirely local: there are no cloud features, CAM, sheet-metal,
simulation or electronics workbenches.

The repository contains the architecture scaffold and a working multi-sketch →
parametric solid workflow. Draw lines or rectangles, apply constraints, edit
unit-aware parameters, generate exact OCCT solids with joins and cuts, and save/reopen local `.con` files.

On NixOS:

```sh
nix-shell --run 'cargo run --features desktop --locked'
```

On other systems, install OCCT and GPUI's native prerequisites, set
`OCCT_INCLUDE_DIR` and `OCCT_LIB_DIR`, and run `cargo run --features desktop`.

The GPUI shell includes open-design tabs, a Solid/Sketch/Drawing workspace dropdown,
pinnable contextual icon ribbons, a floating document tree, a camera cube, view controls
and a construction timeline with a draggable marker.
Feature menus include future tools marked **Not implemented**.

Choose **Create sketch**, then use **R**, **L**, or **C** to draw geometry. Use **D**
to add driving dimensions, constraint tools to define relationships, and **I** to
measure selections. Drag points and lines to edit under constraints. The sketch
menus also include arcs, ellipses, splines, slots, trim, offset, and transforms.
Press **E** to extrude a closed line, circle, or arc region, including holes. Select a boundary curve first when choosing among multiple regions. Open and Save use a local
path editor. Select a planar extrusion cap, choose **Create sketch**, then extrude with **Cut** to create a face-attached pocket. Ellipse/spline extrusion and complete Fusion parity remain unfinished.
See the [UI guide](docs/user-interface.md) and
[workflow guide](docs/parametric-workflow.md) for controls and current limits.

`.con` files save current construction features and their dependencies. The bottom
timeline previews and edits those features. Undo and redo store parameter and other
edits in memory only; those edit records are discarded when the application closes.

The desktop experiment uses a pinned GPUI-compatible WGPUI fork for shared-device
GPU composition. See [viewport proof](docs/viewport-proof.md) for implementation,
validation, dependency details and current limits.

- [Architecture and implementation gates](docs/architecture.md)
- [Library selections and backend integration decisions](docs/libraries.md)
- [Feature parity checklist](docs/feature-parity.md)
- [Rust file map](docs/file-map.md)

Headless checks: `cargo test --lib` and `cargo fmt --check`. The GPU smoke check uses
the same shader and picker without opening a window:

```sh
cargo run --features gpu --example viewport_smoke -- /tmp/confusion-viewport.png
```

Headless solver/kernel integration checks:

```sh
nix-shell --run 'cargo test --features solver,kernel'
```

The desktop enables the numerical solver and native OCCT bridge; neither is
required for headless document validation or `.con` persistence.
