//! Fork-specific shared-device GPUI compositor adapter for the viewport proof.
//!
//! Exports GpuViewport behind desktop. UI owns this adapter; renderer and camera do
//! not import GPUI. Commands are submitted on the UI thread, on the compositor's queue,
//! before its sampling pass. No second swapchain or full-frame CPU transfer is used.

#[cfg(feature = "desktop")]
mod implementation {
    use crate::render::{
        camera::Camera,
        picking::{GpuPicker, PickResult, pick_pixel},
        renderer::ViewportRenderer,
    };
    use gpui::WgpuSurfaceHandle;

    pub struct GpuViewport {
        pub surface: WgpuSurfaceHandle,
        renderer: ViewportRenderer,
        picker: GpuPicker,
    }

    impl GpuViewport {
        pub fn new(surface: WgpuSurfaceHandle) -> Self {
            let (width, height) = surface.size();
            let renderer = ViewportRenderer::new(
                surface.device().clone(),
                surface.queue().clone(),
                surface.format(),
                [width, height],
            );
            Self {
                surface,
                renderer,
                picker: GpuPicker::default(),
            }
        }

        pub fn set_grid(&mut self, visible: bool) {
            self.renderer.set_grid(visible);
        }

        pub fn set_mesh(&mut self, vertices: &[crate::render::scene::DemoVertex], indices: &[u32]) {
            self.renderer.set_mesh(vertices, indices);
        }

        pub fn draw(
            &mut self,
            camera: &Camera,
            selected_face: u32,
            pick: Option<([f64; 2], u64)>,
        ) -> Result<Option<PickResult>, String> {
            let mut result = self.picker.poll(self.surface.device())?;
            if let Some((target, (width, height))) = self.surface.back_view_with_size() {
                if result
                    .as_ref()
                    .is_some_and(|pick| pick.target_size != [width, height])
                {
                    result = None;
                }
                self.renderer.resize([width, height]);
                self.renderer.render(&target, camera, selected_face);
                if let Some((normalized, revision)) = pick
                    && let Some(pixel) = pick_pixel(normalized, [width, height])
                {
                    self.picker.request(
                        self.surface.device(),
                        self.surface.queue(),
                        self.renderer.pick_texture(),
                        pixel,
                        revision,
                    );
                }
                drop(target);
                // Render and composition use one device/queue on the same thread.
                // Queue submission order provides the dependency before texture sampling.
                // present_synced would request a separate fast-blit redraw and bypass UI updates.
                self.surface.swap_buffers();
            }
            Ok(result)
        }
    }
}
#[cfg(feature = "desktop")]
pub use implementation::GpuViewport;
