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

The repository currently contains an architecture scaffold, not an implemented CAD
application. Each Rust file describes its planned responsibilities, exported APIs,
connections and invariants.

- [Architecture and implementation gates](docs/architecture.md)
- [Library selections and backend integration decisions](docs/libraries.md)
- [Feature parity checklist](docs/feature-parity.md)
- [Rust file map](docs/file-map.md)

Run `cargo check --lib --locked` and `cargo fmt --check` to check the scaffold. Core
backend dependencies are optional: `solver`, `kernel`, `gpu` and `desktop`. These
features reserve library dependencies; they do not implement those backends yet.
GPUI GPU composition and native OCCT integration must be validated before the
scaffold becomes a functioning desktop application.
