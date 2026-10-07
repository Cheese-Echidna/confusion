//! Bounded asynchronous one-pixel GPU face-ID readback for the viewport proof.
//!
//! Exports GpuPicker, PickResult and pick_pixel. The UI stamps requests with the
//! camera revision and rejects obsolete results. There is no full-frame CPU readback.

pub fn pick_pixel(normalized: [f64; 2], size: [u32; 2]) -> Option<[u32; 2]> {
    if size.contains(&0)
        || normalized
            .iter()
            .any(|value| !value.is_finite() || !(0.0..1.0).contains(value))
    {
        return None;
    }
    Some(std::array::from_fn(|index| {
        (normalized[index] * size[index] as f64).floor() as u32
    }))
}

#[cfg(feature = "gpu")]
mod implementation {
    use std::sync::mpsc::{self, Receiver, TryRecvError};

    pub struct PickResult {
        pub face: u32,
        pub revision: u64,
        pub target_size: [u32; 2],
    }
    struct PendingPick {
        buffer: wgpu::Buffer,
        completion: Receiver<Result<(), wgpu::BufferAsyncError>>,
        revision: u64,
        target_size: [u32; 2],
    }
    #[derive(Default)]
    pub struct GpuPicker {
        pending: Option<PendingPick>,
    }

    impl GpuPicker {
        pub fn request(
            &mut self,
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            texture: &wgpu::Texture,
            pixel: [u32; 2],
            revision: u64,
        ) {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("One-pixel face ID readback"),
                size: 256,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            let mut encoder = device.create_command_encoder(&Default::default());
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: pixel[0],
                        y: pixel[1],
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(256),
                        rows_per_image: Some(1),
                    },
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
            queue.submit([encoder.finish()]);
            let (sender, completion) = mpsc::channel();
            buffer
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |result| {
                    // Replaced requests drop their receiver; that is intentional cancellation.
                    let _ = sender.send(result);
                });
            self.pending = Some(PendingPick {
                buffer,
                completion,
                revision,
                target_size: [texture.width(), texture.height()],
            });
        }

        pub fn is_pending(&self) -> bool {
            self.pending.is_some()
        }

        pub fn poll(&mut self, device: &wgpu::Device) -> Result<Option<PickResult>, String> {
            device
                .poll(wgpu::PollType::Poll)
                .map_err(|error| error.to_string())?;
            let Some(pending) = &self.pending else {
                return Ok(None);
            };
            match pending.completion.try_recv() {
                Err(TryRecvError::Empty) => Ok(None),
                Err(TryRecvError::Disconnected) => {
                    self.pending = None;
                    Err("GPU picking completion channel closed".into())
                }
                Ok(completion) => {
                    let pending = self
                        .pending
                        .take()
                        .expect("Pending pick exists in this branch");
                    completion.map_err(|error| error.to_string())?;
                    let mapping = pending.buffer.slice(..).get_mapped_range();
                    let face = u32::from_le_bytes(
                        mapping[..4]
                            .try_into()
                            .map_err(|_| "Invalid GPU pick buffer")?,
                    );
                    drop(mapping);
                    pending.buffer.unmap();
                    Ok(Some(PickResult {
                        face,
                        revision: pending.revision,
                        target_size: pending.target_size,
                    }))
                }
            }
        }
    }
}
#[cfg(feature = "gpu")]
pub use implementation::{GpuPicker, PickResult};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalized_coordinates_match_high_dpi_and_resized_targets() {
        assert_eq!(pick_pixel([0.5, 0.25], [800, 600]), Some([400, 150]));
        assert_eq!(pick_pixel([0.5, 0.25], [1600, 1200]), Some([800, 300]));
        assert_eq!(pick_pixel([0.99, 0.99], [3, 5]), Some([2, 4]));
    }
    #[test]
    fn outside_minimized_and_nonfinite_coordinates_are_not_picked() {
        for position in [
            [1.0, 0.0],
            [-0.1, 0.5],
            [f64::NAN, 0.0],
            [0.0, f64::INFINITY],
        ] {
            assert_eq!(pick_pixel(position, [800, 600]), None);
        }
        assert_eq!(pick_pixel([0.5, 0.5], [0, 600]), None);
    }
}
