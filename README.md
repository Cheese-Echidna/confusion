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

The repository contains the architecture scaffold and a working planar sketch →
parametric extrusion workflow. Draw lines or rectangles, apply constraints, edit
unit-aware parameters, generate an exact OCCT solid, and save/reopen local `.con` files.

On NixOS:

```sh
nix-shell --run 'cargo run --features desktop --locked'
```

On other systems, install OCCT and GPUI's native prerequisites, set
`OCCT_INCLUDE_DIR` and `OCCT_LIB_DIR`, and run `cargo run --features desktop`.

Press **R**, then click opposite corners to draw a dimensioned rectangle. Change
`width` or `height` in the sidebar and click **Apply**. Set an extrusion depth
(for example `10 mm`) and click **Extrude / update**. Enter a local filename in the
Document field to **Save** or **Open** a `.con` file. See the
[workflow guide](docs/parametric-workflow.md) for controls and implementation limits.

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
