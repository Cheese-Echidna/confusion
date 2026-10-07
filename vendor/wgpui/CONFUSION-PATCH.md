Vendored from https://github.com/nestrilabs/wgpui at baa3f3b00839ee9567b81450f45a58c3a6c43cb8.

Local change: cross/window.rs passes an owned Arc<winit::window::Window> to
cross/renderer.rs, which uses Instance::create_surface instead of raw handles.
The surface retains the native window until swapchain destruction completes,
preventing a Wayland/NVIDIA SIGSEGV during close.

Keep this patch until the pinned fork includes equivalent window ownership.

Validation: `cargo run --features desktop --example window_close_smoke` must
exit with status 0. Run it in a Wayland session with the NVIDIA Vulkan driver;
X11 alone does not reproduce the invalid Wayland proxy destruction. The example
creates the real Confusion workspace, renders, and closes its native window.
Also check close confirmation → Cancel → close again → Discard on a dirty design.
