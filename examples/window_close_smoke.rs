//! Native shutdown regression check. Run on Wayland/NVIDIA as well as X11.
//! Success requires a normal process exit after the rendered window closes.
use gpui::{App, AppContext, Application, WindowOptions};
use std::time::Duration;

fn main() {
    Application::new()
        .with_assets(confusion::ui::assets::Icons)
        .run(|cx: &mut App| {
            cx.on_window_closed(|cx| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let window = cx
                .open_window(WindowOptions::default(), |window, cx| {
                    window.set_window_title("Confusion shutdown regression check");
                    let surface = window
                        .create_wgpu_surface(640, 480, wgpu::TextureFormat::Rgba8Unorm)
                        .expect("shared-device surface");
                    cx.new(|cx| confusion::ui::viewport::WorkspaceView::new(surface, cx))
                })
                .expect("native window");
            cx.spawn(async move |cx| {
                cx.background_executor().timer(Duration::from_secs(2)).await;
                window
                    .update(cx, |view, window, cx| {
                        assert!(view.request_window_close(window, cx));
                        window.remove_window();
                    })
                    .expect("close window");
            })
            .detach();
        });
}
