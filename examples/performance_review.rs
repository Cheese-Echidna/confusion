//! Reproducible offscreen production-renderer and parametric rebuild measurements.
//! Run in release with --features gpu,solver,kernel. FPS excludes GPUI/presentation.
use confusion::{
    document::{
        model::{CapRole, ExtrudeFeature, ExtrudeOperation, SketchPlane},
        schema::{Design, Extrusion},
    },
    evaluation::solid,
    render::{camera::Camera, renderer::ViewportRenderer, scene::DemoVertex},
};
use std::time::{Duration, Instant};
fn stats(mut v: Vec<f64>) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[v.len() / 2], v[(v.len() * 95 / 100).min(v.len() - 1)])
}
fn design(pockets: usize) -> Design {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.08, 0.05]);
    let depth = d.parameter("thickness", "10 mm".into());
    let base = uuid::Uuid::new_v4();
    d.extrusion = Some(Extrusion {
        id: base,
        depth,
        boundary: vec![],
    });
    d.sync_construction();
    let mut target = base;
    for n in 0..pockets {
        let sketch = d
            .create_sketch(SketchPlane::Face {
                support: target,
                producer: base,
                role: CapRole::End,
            })
            .unwrap();
        let x = 0.004 + (n % 8) as f64 * 0.009;
        let y = 0.004 + (n / 8) as f64 * 0.012;
        d.rectangle([x, y], [x + 0.005, y + 0.007]);
        let boundary = confusion::sketch::regions::select(
            &d,
            &d.points.iter().map(|p| p.xy).collect::<Vec<_>>(),
            &[],
        )
        .unwrap()
        .boundary;
        let depth = d.parameter(&format!("pocket{n}"), "3 mm".into());
        let id = uuid::Uuid::new_v4();
        d.features.push(ExtrudeFeature {
            id,
            name: format!("Pocket {n}"),
            sketch,
            boundary,
            depth,
            operation: ExtrudeOperation::Cut,
            target: Some(target),
        });
        d.sync_construction();
        target = id;
    }
    d
}
fn wait(device: &wgpu::Device) {
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(30)),
        })
        .unwrap();
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))?;
    println!("adapter={:?}", adapter.get_info());
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))?;
    for count in [1, 10, 24, 32] {
        let mut sketch = Design::default();
        for n in 0..count {
            sketch.rectangle([n as f64 * 0.01, 0.], [n as f64 * 0.01 + 0.005, 0.005]);
        }
        let parameters = confusion::parameters::expression::evaluate(&sketch)?;
        let mut times = vec![];
        for _ in 0..5 {
            let start = Instant::now();
            confusion::solver::nonlinear::solve(&sketch, &parameters)?;
            times.push(start.elapsed().as_secs_f64() * 1000.);
        }
        sketch.parameters[0].expression = "6 mm".into();
        let edited_parameters = confusion::parameters::expression::evaluate(&sketch)?;
        let mut edited_times = vec![];
        for _ in 0..5 {
            let start = Instant::now();
            confusion::solver::nonlinear::solve(&sketch, &edited_parameters)?;
            edited_times.push(start.elapsed().as_secs_f64() * 1000.);
        }
        println!(
            "SKETCH rectangles={count} points={} constraints={} solve_ms={:?} edited_solve_ms={:?}",
            sketch.points.len(),
            sketch.constraints.len(),
            stats(times),
            stats(edited_times)
        );
    }
    let mut scenes = vec![];
    for pockets in [0, 1, 8, 24] {
        let d = design(pockets);
        let mut total = vec![];
        let mut solver = vec![];
        let mut mesh = None;
        for _ in 0..8 {
            let start = Instant::now();
            let m = solid::evaluate(&d, || false)?;
            total.push(start.elapsed().as_secs_f64() * 1000.);
            mesh = m.mesh;
            let start = Instant::now();
            let p = confusion::parameters::expression::evaluate(&d)?;
            for f in &d.construction {
                if matches!(
                    f.kind,
                    confusion::document::schema::ConstructionKind::Sketch
                ) {
                    confusion::solver::nonlinear::solve(&d.sketch_input(f.id)?, &p)?;
                }
            }
            solver.push(start.elapsed().as_secs_f64() * 1000.);
        }
        let mesh = mesh.unwrap();
        let vertices = mesh
            .vertices
            .iter()
            .map(|v| DemoVertex {
                position: [(v.x * 25.) as f32, (v.y * 25.) as f32, (v.z * 25.) as f32],
                normal: [v.nx as f32, v.ny as f32, v.nz as f32],
                face: v.face,
                color: [0.227, 0.376, 0.314],
            })
            .collect::<Vec<_>>();
        println!(
            "MODEL pockets={pockets} faces={} triangles={} rebuild_ms={:?} solve_ms={:?}",
            mesh.faces,
            mesh.indices.len() / 3,
            stats(total),
            stats(solver)
        );
        std::fs::create_dir_all("/tmp/confusion-performance")?;
        confusion::persistence::container::save(
            std::path::Path::new(&format!("/tmp/confusion-performance/pockets-{pockets}.con")),
            &d,
        )?;
        scenes.push((format!("pockets-{pockets}"), vertices, mesh.indices));
    }
    for count in [100, 10000] {
        let (cube, idx) = confusion::render::scene::demo_cube();
        let mut v = vec![];
        let mut i = vec![];
        let side = (count as f32).sqrt().ceil() as usize;
        for n in 0..count {
            let base = v.len() as u32;
            for vertex in &cube {
                let mut vertex = *vertex;
                let scale = 2. / side as f32;
                vertex.position = [
                    (n % side) as f32 * scale - 1. + vertex.position[0] * scale * 0.4,
                    (n / side) as f32 * scale - 1. + vertex.position[1] * scale * 0.4,
                    vertex.position[2] * scale * 0.4,
                ];
                v.push(vertex);
            }
            i.extend(idx.iter().map(|x| base + *x as u32));
        }
        scenes.push((format!("cubes-{count}"), v, i));
    }
    for size in [[800, 600], [1600, 1200], [2560, 1440]] {
        let mut r = ViewportRenderer::new(
            device.clone(),
            queue.clone(),
            wgpu::TextureFormat::Rgba8Unorm,
            size,
        );
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        for (name, v, i) in &scenes {
            let start = Instant::now();
            r.set_mesh(v, i);
            wait(&device);
            let upload = start.elapsed().as_secs_f64() * 1000.;
            for grid in [false, true] {
                r.set_grid(grid, true);
                let mut camera = Camera::default();
                let warmup = Instant::now();
                while warmup.elapsed() < Duration::from_millis(200) {
                    r.render(&view, &camera, 0);
                    wait(&device);
                }
                let mut latency = vec![];
                let mut cpu = vec![];
                for n in 0..120 {
                    camera.yaw = n as f64 * 0.01;
                    let start = Instant::now();
                    r.render(&view, &camera, 0);
                    cpu.push(start.elapsed().as_secs_f64() * 1000.);
                    wait(&device);
                    latency.push(start.elapsed().as_secs_f64() * 1000.);
                }
                let start = Instant::now();
                for n in 0..2400 {
                    camera.yaw = n as f64 * 0.01;
                    r.render(&view, &camera, 0);
                    if n % 3 == 2 {
                        wait(&device);
                    }
                }
                wait(&device);
                let fps = 2400. / start.elapsed().as_secs_f64();
                println!(
                    "RENDER {name} size={size:?} grid={grid} triangles={} cpu_ms={:?} completed_frame_ms={:?} throughput_fps={fps:.1} upload_ms={upload:.3}",
                    i.len() / 3,
                    stats(cpu),
                    stats(latency)
                );
            }
        }
    }
    Ok(())
}
