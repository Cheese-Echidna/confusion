//! Desktop composition root for the planar sketch → exact extrusion workflow.
//!
//! Exports run_desktop behind desktop. Creates one GPUI-owned window and shared-device
//! texture surface, then hands it to ui/viewport. The workspace owns its local document and background evaluation worker.

#[cfg(feature = "desktop")]
pub fn run_desktop() {
    use crate::ui::viewport::WorkspaceView;
    use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size};

    Application::new()
        .with_assets(crate::ui::assets::Icons)
        .run(|cx: &mut App| {
            crate::ui::text_input::bind_keys(cx);
            cx.on_window_closed(|cx| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let bounds = Bounds::centered(None, size(px(1100.), px(760.)), cx);
            match cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    window.set_window_title("Confusion · Parametric design");
                    let surface = window
                    .create_wgpu_surface(1100, 670, wgpu::TextureFormat::Rgba8Unorm)
                    .expect(
                        "The pinned GPUI compositor must provide a shared-device viewport surface",
                    );
                    {
                        let view = cx.new(|cx| WorkspaceView::new(surface, cx));
                        view.read(cx).focus(window);
                        view
                    }
                },
            ) {
                Ok(_) => cx.activate(true),
                Err(error) => {
                    eprintln!("Could not open Confusion: {error}");
                    cx.quit();
                }
            }
        });
}
