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
        eye: [f32; 4],
        outward: [f32; 4],
    }

    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
    struct EdgeVertex {
        position: [f32; 3],
        first: [f32; 3],
        second: [f32; 3],
        boundary: u32,
    }
    struct DisplayMesh {
        vertices: wgpu::Buffer,
        indices: wgpu::Buffer,
        count: u32,
        edges: wgpu::Buffer,
        edge_count: u32,
    }
    fn edge_vertices(vertices: &[DemoVertex], indices: &[u32]) -> Vec<EdgeVertex> {
        use std::collections::BTreeMap;
        let mut edges =
            BTreeMap::<([i64; 3], [i64; 3]), Vec<([f32; 3], [f32; 3], [f32; 3], u32)>>::new();
        let key = |p: [f32; 3]| p.map(|x| (x as f64 * 1e7).round() as i64);
        for triangle in indices.chunks_exact(3) {
            let a = vertices[triangle[0] as usize];
            let b = vertices[triangle[1] as usize];
            let c = vertices[triangle[2] as usize];
            let pa = nalgebra::Vector3::from(a.position);
            let pb = nalgebra::Vector3::from(b.position);
            let pc = nalgebra::Vector3::from(c.position);
            let n = (pb - pa)
                .cross(&(pc - pa))
                .try_normalize(1e-12)
                .unwrap_or_default();
            let normal = [n.x, n.y, n.z];
            for (mut p, mut q) in [
                (a.position, b.position),
                (b.position, c.position),
                (c.position, a.position),
            ] {
                if key(p) > key(q) {
                    std::mem::swap(&mut p, &mut q);
                }
                edges
                    .entry((key(p), key(q)))
                    .or_default()
                    .push((p, q, normal, a.face));
            }
        }
        let mut result = Vec::new();
        for adjacent in edges.values() {
            let (p, q, n, face) = adjacent[0];
            let boundary = adjacent.len() != 2 || adjacent.iter().any(|e| e.3 != face);
            let second = adjacent.get(1).map_or(n, |e| e.2);
            if !boundary
                && nalgebra::Vector3::from(n).dot(&nalgebra::Vector3::from(second)) > 0.99999
            {
                continue;
            }
            result.push(EdgeVertex {
                position: p,
                first: n,
                second,
                boundary: u32::from(boundary),
            });
            result.push(EdgeVertex {
                position: q,
                first: n,
                second,
                boundary: u32::from(boundary),
            });
        }
        result
    }
    pub struct ViewportRenderer {
        device: wgpu::Device,
        queue: wgpu::Queue,
        pipeline: wgpu::RenderPipeline,
        pick_pipeline: wgpu::RenderPipeline,
        depth_pipeline: wgpu::RenderPipeline,
        edge_pipeline: wgpu::RenderPipeline,
        hidden_pipeline: wgpu::RenderPipeline,
        wire_pipeline: wgpu::RenderPipeline,
        edges: wgpu::Buffer,
        edge_count: u32,
        preview: Option<DisplayMesh>,
        style: crate::render::passes::ViewStyle,
        multisample_color: wgpu::TextureView,
        multisample_depth: wgpu::TextureView,
        resolve_view: wgpu::TextureView,
        resolve_pipeline: wgpu::RenderPipeline,
        resolve_layout: wgpu::BindGroupLayout,
        resolve_bindings: wgpu::BindGroup,
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
            let output_format = format;
            let format = wgpu::TextureFormat::Rgba16Float;
            let descriptor = wgpu::RenderPipelineDescriptor {
                label: Some("4x MSAA shaded solid"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex_main"),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<DemoVertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Uint32, 3 => Float32x3],
                    }],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragment_color"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(true),
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
            };
            let pipeline = device.create_render_pipeline(&descriptor);
            let mut depth_descriptor = descriptor.clone();
            depth_descriptor.label = Some("Wireframe depth prepass");
            let depth_targets = [Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::empty(),
            })];
            depth_descriptor.fragment.as_mut().unwrap().targets = &depth_targets;
            let depth_pipeline = device.create_render_pipeline(&depth_descriptor);
            let edge_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Face boundaries and silhouettes"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/edges.wgsl").into()),
            });
            let make_edge = |entry, compare, bias| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
                label:Some(entry),layout:Some(&pipeline_layout),
                vertex:wgpu::VertexState{module:&edge_shader,entry_point:Some("vertex_main"),buffers:&[wgpu::VertexBufferLayout{array_stride:std::mem::size_of::<EdgeVertex>() as u64,step_mode:wgpu::VertexStepMode::Vertex,attributes:&wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x3,3=>Uint32]}],compilation_options:Default::default()},
                fragment:Some(wgpu::FragmentState{module:&edge_shader,entry_point:Some(entry),targets:&[Some(wgpu::ColorTargetState{format,blend:None,write_mask:wgpu::ColorWrites::ALL})],compilation_options:Default::default()}),
                primitive:wgpu::PrimitiveState{topology:wgpu::PrimitiveTopology::LineList,..Default::default()},
                depth_stencil:Some(wgpu::DepthStencilState{format:wgpu::TextureFormat::Depth32Float,depth_write_enabled:Some(false),depth_compare:Some(compare),stencil:Default::default(),bias:wgpu::DepthBiasState{constant:bias,..Default::default()}}),
                multisample:wgpu::MultisampleState{count:4,..Default::default()},multiview_mask:None,cache:None})
            };
            let edge_pipeline = make_edge("visible", wgpu::CompareFunction::LessEqual, 0);
            let hidden_pipeline = make_edge("hidden", wgpu::CompareFunction::Greater, 0);
            let wire_pipeline = make_edge("wire", wgpu::CompareFunction::Always, 0);
            let pick_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Single sample face picking"),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragment_pick"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::R32Uint,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multisample: Default::default(),
                ..descriptor
            });
            let (multisample_color, multisample_depth) =
                Self::multisample_targets(&device, size, format);
            let resolve_view = Self::resolve_target(&device, size);
            let resolve_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Linear resolve layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                });
            let resolve_bindings =
                Self::resolve_bindings(&device, &resolve_layout, &uniforms, &resolve_view);
            let resolve_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Linear light MSAA resolve"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/resolve.wgsl").into()),
            });
            let resolve_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Viewport presentation"),
                layout: Some(
                    &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[Some(&resolve_layout)],
                        immediate_size: 0,
                    }),
                ),
                vertex: wgpu::VertexState {
                    module: &resolve_shader,
                    entry_point: Some("vertex_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &resolve_shader,
                    entry_point: Some("fragment_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: output_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: Default::default(),
                depth_stencil: None,
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
            let edge_data = edge_vertices(
                &mesh_vertices,
                &mesh_indices.iter().map(|i| *i as u32).collect::<Vec<_>>(),
            );
            let edges = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Solid boundaries"),
                contents: bytemuck::cast_slice(&edge_data),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let (depth, pick_texture, pick_view) = Self::targets(&device, size);
            Self {
                device,
                queue,
                pipeline,
                pick_pipeline,
                depth_pipeline,
                edge_pipeline,
                hidden_pipeline,
                wire_pipeline,
                edges,
                edge_count: edge_data.len() as u32,
                preview: None,
                style: Default::default(),
                multisample_color,
                multisample_depth,
                resolve_view,
                resolve_pipeline,
                resolve_layout,
                resolve_bindings,
                vertices,
                indices,
                index_count: mesh_indices.len() as u32,
                uniforms,
                bindings,
                depth,
                pick_texture,
                pick_view,
                size,
                encode_srgb: !output_format.is_srgb(),
                grid,
                show_grid: false,
                show_axes: true,
            }
        }

        pub fn set_grid_frame(&mut self, frame: crate::sketch::workplane::Workplane) {
            self.grid.set_frame(&self.queue, frame);
        }
        pub fn set_grid(&mut self, visible: bool, axes: bool) {
            self.show_axes = axes;
            self.show_grid = visible;
        }

        pub fn set_mesh(&mut self, vertices: &[DemoVertex], indices: &[u32]) {
            self.index_count = indices.len() as u32;
            let edges = edge_vertices(vertices, indices);
            self.edge_count = edges.len() as u32;
            if !edges.is_empty() {
                self.edges = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Solid edges"),
                        contents: bytemuck::cast_slice(&edges),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
            }
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

        pub fn set_style(&mut self, style: crate::render::passes::ViewStyle) {
            self.style = style;
        }
        pub fn clear_preview(&mut self) {
            self.preview = None;
        }
        pub fn set_preview(&mut self, vertices: &[DemoVertex], indices: &[u32]) {
            let edges = edge_vertices(vertices, indices);
            let buffer = |label, data: &[u8], usage| {
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(label),
                        contents: if data.is_empty() { &[0; 4] } else { data },
                        usage,
                    })
            };
            self.preview = Some(DisplayMesh {
                vertices: buffer(
                    "Preview vertices",
                    bytemuck::cast_slice(vertices),
                    wgpu::BufferUsages::VERTEX,
                ),
                indices: buffer(
                    "Preview indices",
                    bytemuck::cast_slice(indices),
                    wgpu::BufferUsages::INDEX,
                ),
                count: indices.len() as u32,
                edges: buffer(
                    "Preview edges",
                    bytemuck::cast_slice(&edges),
                    wgpu::BufferUsages::VERTEX,
                ),
                edge_count: edges.len() as u32,
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

        fn multisample_targets(
            device: &wgpu::Device,
            size: [u32; 2],
            format: wgpu::TextureFormat,
        ) -> (wgpu::TextureView, wgpu::TextureView) {
            let target = |format, label| {
                device
                    .create_texture(&wgpu::TextureDescriptor {
                        label: Some(label),
                        size: wgpu::Extent3d {
                            width: size[0],
                            height: size[1],
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 4,
                        dimension: wgpu::TextureDimension::D2,
                        format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[],
                    })
                    .create_view(&Default::default())
            };
            (
                target(format, "4x MSAA viewport color"),
                target(wgpu::TextureFormat::Depth32Float, "4x MSAA viewport depth"),
            )
        }

        fn resolve_target(device: &wgpu::Device, size: [u32; 2]) -> wgpu::TextureView {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("Resolved linear viewport"),
                    size: wgpu::Extent3d {
                        width: size[0],
                        height: size[1],
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba16Float,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        }
        fn resolve_bindings(
            device: &wgpu::Device,
            layout: &wgpu::BindGroupLayout,
            uniforms: &wgpu::Buffer,
            view: &wgpu::TextureView,
        ) -> wgpu::BindGroup {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Resolved viewport bindings"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniforms.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                ],
            })
        }
        pub fn resize(&mut self, size: [u32; 2]) {
            let size = size.map(|value| value.max(1));
            if self.size != size {
                (self.depth, self.pick_texture, self.pick_view) = Self::targets(&self.device, size);
                (self.multisample_color, self.multisample_depth) =
                    Self::multisample_targets(&self.device, size, wgpu::TextureFormat::Rgba16Float);
                self.resolve_view = Self::resolve_target(&self.device, size);
                self.resolve_bindings = Self::resolve_bindings(
                    &self.device,
                    &self.resolve_layout,
                    &self.uniforms,
                    &self.resolve_view,
                );
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
                    selection: [
                        if self.preview.is_some() {
                            0
                        } else {
                            selected_face
                        },
                        u32::from(self.encode_srgb),
                        u32::from(matches!(
                            self.style,
                            crate::render::passes::ViewStyle::Wireframe
                                | crate::render::passes::ViewStyle::VisibleEdges
                        )),
                        0,
                    ],
                    eye: [
                        camera.eye().x as f32,
                        camera.eye().y as f32,
                        camera.eye().z as f32,
                        if camera.projection == crate::render::camera::ProjectionMode::Perspective {
                            1.
                        } else {
                            0.
                        },
                    ],
                    outward: [
                        camera.outward().x as f32,
                        camera.outward().y as f32,
                        camera.outward().z as f32,
                        0.,
                    ],
                }),
            );
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Confusion viewport frame"),
                });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("4x MSAA solid and grid"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.multisample_color,
                        resolve_target: Some(&self.resolve_view),
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.021219,
                                g: 0.021219,
                                b: 0.021219,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.multisample_depth,
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
                use crate::render::passes::ViewStyle;
                let wire = matches!(self.style, ViewStyle::Wireframe | ViewStyle::VisibleEdges);
                let (vertices, indices, count, edges, edge_count) = self.preview.as_ref().map_or(
                    (
                        &self.vertices,
                        &self.indices,
                        self.index_count,
                        &self.edges,
                        self.edge_count,
                    ),
                    |p| (&p.vertices, &p.indices, p.count, &p.edges, p.edge_count),
                );
                pass.set_pipeline(if wire {
                    &self.depth_pipeline
                } else {
                    &self.pipeline
                });
                pass.set_bind_group(0, &self.bindings, &[]);
                pass.set_vertex_buffer(0, vertices.slice(..));
                pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
                if self.show_grid {
                    self.grid.draw(&mut pass, self.show_axes);
                }
                if self.style != ViewStyle::Shaded {
                    pass.set_bind_group(0, &self.bindings, &[]);
                    pass.set_vertex_buffer(0, edges.slice(..));
                    if self.style == ViewStyle::ShadedHiddenEdges {
                        pass.set_pipeline(&self.hidden_pipeline);
                        pass.draw(0..edge_count, 0..1);
                    }
                    pass.set_pipeline(if self.style == ViewStyle::Wireframe {
                        &self.wire_pipeline
                    } else {
                        &self.edge_pipeline
                    });
                    pass.draw(0..edge_count, 0..1);
                }
            }
            // Integer face IDs stay single-sampled: never average selectable IDs.
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Exact face picking"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.pick_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.),
                            store: wgpu::StoreOp::Discard,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&self.pick_pipeline);
                pass.set_bind_group(0, &self.bindings, &[]);
                pass.set_vertex_buffer(0, self.vertices.slice(..));
                pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..self.index_count, 0, 0..1);
            }
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Encode resolved viewport"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: color,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&self.resolve_pipeline);
                pass.set_bind_group(0, &self.resolve_bindings, &[]);
                pass.draw(0..3, 0..1);
            }
            self.queue.submit([encoder.finish()])
        }
    }
}
#[cfg(feature = "gpu")]
pub use implementation::ViewportRenderer;
