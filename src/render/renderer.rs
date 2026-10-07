//! Depth-tested solid rendering on an externally supplied wgpu device and queue.
//!
//! Exports ViewportRenderer. The GPUI bridge provides the target texture; the smoke
//! example provides an offscreen target. Neither path creates a second window swapchain.
//! Mesh/pipeline resources persist between frames; only uniforms and resized targets change.

#[cfg(feature = "gpu")]
mod implementation {
    use super::super::{
        camera::Camera,
        scene::{DemoVertex, demo_cube},
    };
    use wgpu::util::DeviceExt;

    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
    struct Uniforms {
        view_projection: [[f32; 4]; 4],
        selection: [u32; 4],
    }

    pub struct ViewportRenderer {
        device: wgpu::Device,
        queue: wgpu::Queue,
        pipeline: wgpu::RenderPipeline,
        vertices: wgpu::Buffer,
        indices: wgpu::Buffer,
        index_count: u32,
        uniforms: wgpu::Buffer,
        bindings: wgpu::BindGroup,
        depth: wgpu::TextureView,
        pick_texture: wgpu::Texture,
        pick_view: wgpu::TextureView,
        size: [u32; 2],
        encode_srgb: bool,
        grid: super::super::grid::GridRenderer,
        show_grid: bool,
        show_axes: bool,
    }

    impl ViewportRenderer {
        pub fn new(
            device: wgpu::Device,
            queue: wgpu::Queue,
            format: wgpu::TextureFormat,
            size: [u32; 2],
        ) -> Self {
            let size = size.map(|value| value.max(1));
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Confusion solid shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/solid.wgsl").into()),
            });
            let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Camera and selection"),
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Viewport uniforms layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
            let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Viewport uniforms"),
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms.as_entire_binding(),
                }],
            });
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Viewport pipeline layout"),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            });
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Shaded solid + GPU face IDs"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex_main"),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<DemoVertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Uint32],
                    }],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragment_main"),
                    targets: &[
                        Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL }),
                        Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::R32Uint, blend: None, write_mask: wgpu::ColorWrites::ALL }),
                    ],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState { cull_mode: Some(wgpu::Face::Back), ..Default::default() },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            });
            let grid = super::super::grid::GridRenderer::new(&device, format, &layout);
            let (mesh_vertices, mesh_indices) = demo_cube();
            let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Demo solid vertices"),
                contents: bytemuck::cast_slice(&mesh_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Demo solid indices"),
                contents: bytemuck::cast_slice(
                    &mesh_indices.iter().map(|i| *i as u32).collect::<Vec<_>>(),
                ),
                usage: wgpu::BufferUsages::INDEX,
            });
            let (depth, pick_texture, pick_view) = Self::targets(&device, size);
            Self {
                device,
                queue,
                pipeline,
                vertices,
                indices,
                index_count: mesh_indices.len() as u32,
                uniforms,
                bindings,
                depth,
                pick_texture,
                pick_view,
                size,
                encode_srgb: !format.is_srgb(),
                grid,
                show_grid: false,
                show_axes: true,
            }
        }

        pub fn set_grid(&mut self, visible: bool, axes: bool) {
            self.show_axes = axes;
            self.show_grid = visible;
        }

        pub fn set_mesh(&mut self, vertices: &[DemoVertex], indices: &[u32]) {
            self.index_count = indices.len() as u32;
            if indices.is_empty() {
                return;
            }
            self.vertices = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Evaluated solid vertices"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            self.indices = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Evaluated solid indices"),
                    contents: bytemuck::cast_slice(indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
        }

        fn targets(
            device: &wgpu::Device,
            size: [u32; 2],
        ) -> (wgpu::TextureView, wgpu::Texture, wgpu::TextureView) {
            let texture = |label, format, usage| {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: size[0],
                        height: size[1],
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                })
            };
            let depth = texture(
                "Viewport depth",
                wgpu::TextureFormat::Depth32Float,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            )
            .create_view(&Default::default());
            let pick_texture = texture(
                "Viewport face IDs",
                wgpu::TextureFormat::R32Uint,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            );
            let pick_view = pick_texture.create_view(&Default::default());
            (depth, pick_texture, pick_view)
        }

        pub fn resize(&mut self, size: [u32; 2]) {
            let size = size.map(|value| value.max(1));
            if self.size != size {
                (self.depth, self.pick_texture, self.pick_view) = Self::targets(&self.device, size);
                self.size = size;
            }
        }

        pub fn size(&self) -> [u32; 2] {
            self.size
        }
        pub fn pick_texture(&self) -> &wgpu::Texture {
            &self.pick_texture
        }
        pub fn device(&self) -> &wgpu::Device {
            &self.device
        }
        pub fn queue(&self) -> &wgpu::Queue {
            &self.queue
        }

        fn output_channel(&self, linear: f64) -> f64 {
            if !self.encode_srgb {
                linear
            } else if linear <= 0.0031308 {
                linear * 12.92
            } else {
                1.055 * linear.powf(1.0 / 2.4) - 0.055
            }
        }

        pub fn render(
            &mut self,
            color: &wgpu::TextureView,
            camera: &Camera,
            selected_face: u32,
        ) -> wgpu::SubmissionIndex {
            self.queue.write_buffer(
                &self.uniforms,
                0,
                bytemuck::bytes_of(&Uniforms {
                    view_projection: camera.gpu_matrix(self.size[0], self.size[1]),
                    selection: [selected_face, u32::from(self.encode_srgb), 0, 0],
                }),
            );
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Confusion viewport frame"),
                });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Solid shading and picking"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: color,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: self.output_channel(0.021219),
                                    g: self.output_channel(0.021219),
                                    b: self.output_channel(0.021219),
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: &self.pick_view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        }),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Discard,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bindings, &[]);
                pass.set_vertex_buffer(0, self.vertices.slice(..));
                pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.index_count, 0, 0..1);
                if self.show_grid {
                    self.grid.draw(&mut pass, self.show_axes);
                }
            }
            self.queue.submit([encoder.finish()])
        }
    }
}
#[cfg(feature = "gpu")]
pub use implementation::ViewportRenderer;
