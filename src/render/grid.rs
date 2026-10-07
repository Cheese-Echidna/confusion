//! Depth-tested XY ground grid on the shared GPU render pass, with Z-up axis colors.
//! Exports GridRenderer behind gpu; viewport renderer controls visibility independently
//! of sketch snapping. It shares the 4x MSAA color pass and never writes face IDs.
#[cfg(feature = "gpu")]
mod implementation {
    use wgpu::util::DeviceExt;
    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
    struct Vertex {
        position: [f32; 3],
        color: [f32; 3],
    }
    pub struct GridRenderer {
        pipeline: wgpu::RenderPipeline,
        vertices: wgpu::Buffer,
        count: u32,
        local_vertices: Vec<Vertex>,
        frame: crate::sketch::workplane::Workplane,
    }
    impl GridRenderer {
        pub fn new(
            device: &wgpu::Device,
            format: wgpu::TextureFormat,
            uniform_layout: &wgpu::BindGroupLayout,
        ) -> Self {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("CAD ground grid"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/grid.wgsl").into()),
            });
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Grid layout"),
                bind_group_layouts: &[Some(uniform_layout)],
                immediate_size: 0,
            });
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Depth tested grid"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex_main"),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3],
                    }],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragment_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 4,
                    ..Default::default()
                },
                multiview_mask: None,
                cache: None,
            });
            let linear = |hex: u32| -> [f32; 3] {
                [16, 8, 0].map(|shift| {
                    let s = ((hex >> shift) & 255) as f32 / 255.;
                    if s <= 0.04045 {
                        s / 12.92
                    } else {
                        ((s + 0.055) / 1.055).powf(2.4)
                    }
                })
            };
            let mut vertices = Vec::new();
            for i in -40..=40 {
                let coordinate = i as f32 * 0.25;
                let color = if i == 0 {
                    linear(0xb7bb26)
                } else {
                    linear(0x383531)
                };
                vertices.extend([
                    Vertex {
                        position: [coordinate, -10., -0.001],
                        color,
                    },
                    Vertex {
                        position: [coordinate, 10., -0.001],
                        color,
                    },
                ]);
                let color = if i == 0 {
                    linear(0xfb4a35)
                } else {
                    linear(0x383531)
                };
                vertices.extend([
                    Vertex {
                        position: [-10., coordinate, -0.001],
                        color,
                    },
                    Vertex {
                        position: [10., coordinate, -0.001],
                        color,
                    },
                ]);
            }
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("XY grid lines"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
            Self {
                pipeline,
                vertices: buffer,
                count: vertices.len() as u32,
                local_vertices: vertices,
                frame: Default::default(),
            }
        }
        pub fn set_frame(
            &mut self,
            queue: &wgpu::Queue,
            frame: crate::sketch::workplane::Workplane,
        ) {
            if self.frame == frame {
                return;
            }
            let vertices: Vec<_> = self
                .local_vertices
                .iter()
                .map(|v| {
                    let position = frame.origin.coords * 25.
                        + frame.x * v.position[0] as f64
                        + frame.y * v.position[1] as f64
                        + frame.normal * v.position[2] as f64;
                    Vertex {
                        position: [position.x as f32, position.y as f32, position.z as f32],
                        color: v.color,
                    }
                })
                .collect();
            queue.write_buffer(&self.vertices, 0, bytemuck::cast_slice(&vertices));
            self.frame = frame;
        }
        pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>, axes: bool) {
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            if axes {
                pass.draw(0..self.count, 0..1);
            } else {
                pass.draw(0..160, 0..1);
                pass.draw(164..self.count, 0..1);
            }
        }
    }
}
#[cfg(feature = "gpu")]
pub use implementation::*;
