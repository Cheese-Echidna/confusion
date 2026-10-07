//! Headless GPU integration check using the same renderer and picker as the GPUI view.
//!
//! Checks depth-tested face IDs, background misses, selection shading and resize;
//! exports a PNG only on explicit invocation. Full-frame readback is test-only.

use confusion::render::{camera::Camera, picking::GpuPicker, renderer::ViewportRenderer};
use std::{
    error::Error,
    fs::File,
    io::BufWriter,
    path::Path,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn target(device: &wgpu::Device, size: [u32; 2], format: wgpu::TextureFormat) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Smoke test color"),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn read_color(renderer: &ViewportRenderer, texture: &wgpu::Texture) -> Result<Vec<u8>> {
    let [width, height] = renderer.size();
    let row_bytes = width * 4;
    let padded_row = row_bytes.div_ceil(256) * 256;
    let buffer = renderer.device().create_buffer(&wgpu::BufferDescriptor {
        label: Some("Smoke image readback"),
        size: padded_row as u64 * height as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = renderer
        .device()
        .create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_row),
                rows_per_image: Some(height),
            },
        },
        texture.size(),
    );
    renderer.queue().submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
    renderer.device().poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(Duration::from_secs(10)),
    })?;
    receiver.recv_timeout(Duration::from_secs(10))??;
    let mapping = buffer.slice(..).get_mapped_range();
    let mut pixels = Vec::with_capacity((row_bytes * height) as usize);
    for row in mapping.chunks_exact(padded_row as usize) {
        pixels.extend_from_slice(&row[..row_bytes as usize]);
    }
    drop(mapping);
    buffer.unmap();
    Ok(pixels)
}

fn pick(renderer: &ViewportRenderer, pixel: [u32; 2], revision: u64) -> Result<u32> {
    let mut picker = GpuPicker::default();
    picker.request(
        renderer.device(),
        renderer.queue(),
        renderer.pick_texture(),
        pixel,
        revision,
    );
    let start = Instant::now();
    loop {
        if let Some(result) = picker.poll(renderer.device())? {
            assert_eq!(
                result.target_size,
                renderer.size(),
                "GPU pick lost its target-size stamp"
            );
            assert_eq!(
                result.revision, revision,
                "GPU pick lost its revision stamp"
            );
            return Ok(result.face);
        }
        if start.elapsed() > Duration::from_secs(10) {
            return Err("GPU pick timed out".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/confusion-viewport.png".into());
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))?;
    println!("GPU adapter: {:?}", adapter.get_info());
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;
    let mut renderer = ViewportRenderer::new(
        device.clone(),
        queue,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        [800, 600],
    );
    let color = target(
        &device,
        renderer.size(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );
    let view = color.create_view(&Default::default());
    let camera = Camera::default();
    renderer.render(&view, &camera, 0);
    let image = read_color(&renderer, &color)?;
    // Flat-shaded cube faces plus the background have four colors without AA.
    // Extra coverage colors prove that the resolved silhouette is antialiased.
    let coverage_colors: std::collections::BTreeSet<_> = image
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| [p[0], p[1], p[2]])
        .collect();
    assert!(
        coverage_colors.len() > 8,
        "MSAA silhouette has no fractional coverage"
    );
    use confusion::render::passes::ViewStyle;
    let original_pick = pick(&renderer, [400, 300], 70)?;
    let mut styles = Vec::new();
    for style in [
        ViewStyle::Shaded,
        ViewStyle::ShadedEdges,
        ViewStyle::ShadedHiddenEdges,
        ViewStyle::Wireframe,
        ViewStyle::VisibleEdges,
    ] {
        renderer.set_style(style);
        renderer.render(&view, &camera, 0);
        let pixels = read_color(&renderer, &color)?;
        assert!(
            styles.iter().all(|previous| previous != &pixels),
            "Styles produce identical output: {style:?}"
        );
        styles.push(pixels);
        assert_eq!(
            pick(&renderer, [400, 300], 71)?,
            original_pick,
            "Style changed picking"
        );
    }
    renderer.set_style(ViewStyle::Shaded);
    let (mut preview, preview_indices) = confusion::render::scene::demo_cube();
    for vertex in &mut preview {
        vertex.position = vertex.position.map(|p| p * 0.5);
        vertex.color = [0.8, 0.4, 0.1];
        vertex.face = 999;
    }
    renderer.set_preview(
        &preview,
        &preview_indices
            .iter()
            .map(|i| *i as u32)
            .collect::<Vec<_>>(),
    );
    renderer.render(&view, &camera, 0);
    assert_ne!(
        read_color(&renderer, &color)?,
        image,
        "Preview did not change display"
    );
    assert_eq!(
        pick(&renderer, [400, 300], 72)?,
        original_pick,
        "Preview replaced committed face IDs"
    );
    renderer.clear_preview();
    renderer.render(&view, &camera, 0);
    assert_eq!(
        read_color(&renderer, &color)?,
        image,
        "Cancel did not restore original mesh"
    );
    renderer.set_grid(true, true);
    renderer.render(&view, &camera, 0);
    let grid_image = read_color(&renderer, &color)?;
    assert!(grid_image != image, "Grid did not render in the MSAA pass");
    assert!(
        (1..=6).contains(&pick(&renderer, [400, 300], 41)?),
        "Grid interfered with solid picking"
    );
    renderer.set_grid(false, true);

    let mut compositor_renderer = ViewportRenderer::new(
        device.clone(),
        renderer.queue().clone(),
        wgpu::TextureFormat::Rgba8Unorm,
        [800, 600],
    );
    let compositor_texture = target(&device, [800, 600], wgpu::TextureFormat::Rgba8Unorm);
    compositor_renderer.render(
        &compositor_texture.create_view(&Default::default()),
        &camera,
        0,
    );
    let compositor_image = read_color(&compositor_renderer, &compositor_texture)?;
    assert!(
        image
            .iter()
            .zip(&compositor_image)
            .all(|(a, b)| a.abs_diff(*b) <= 2),
        "Compositor and standalone color encoding differ"
    );
    let face = pick(&renderer, [400, 300], 42)?;
    assert!(
        (1..=6).contains(&face),
        "Center must hit a visible cube face"
    );
    assert_eq!(
        pick(&renderer, [0, 0], 43)?,
        0,
        "Background must not select a solid"
    );
    renderer.render(&view, &camera, face);
    let selected_image = read_color(&renderer, &color)?;
    assert!(
        image
            .iter()
            .zip(&selected_image)
            .filter(|(a, b)| a != b)
            .count()
            > 1000,
        "Selection shading did not change the rendered face"
    );
    let (mut tinted, cube_indices) = confusion::render::scene::demo_cube();
    for vertex in &mut tinted {
        vertex.color = [0.8, 0.05, 0.05];
    }
    let indices: Vec<u32> = cube_indices.iter().map(|i| *i as u32).collect();
    renderer.set_mesh(&tinted, &indices);
    renderer.render(&view, &camera, 0);
    let tinted_image = read_color(&renderer, &color)?;
    let center = &tinted_image[(300 * 800 + 400) * 4..][..4];
    assert!(
        center[0] > center[1].saturating_mul(2),
        "Body appearance color did not reach the shader"
    );
    assert_eq!(
        pick(&renderer, [400, 300], 44)?,
        face,
        "Appearance changed face picking"
    );
    let (original, _) = confusion::render::scene::demo_cube();
    renderer.set_mesh(&original, &indices);
    for (yaw, elevation, expected) in [
        (0.0, 0.0, 1),
        (std::f64::consts::PI, 0.0, 2),
        (std::f64::consts::FRAC_PI_2, 0.0, 3),
        (-std::f64::consts::FRAC_PI_2, 0.0, 4),
        (0.0, 1.49, 5),
        (0.0, -1.49, 6),
    ] {
        renderer.render(
            &view,
            &Camera {
                yaw,
                elevation,
                ..Camera::default()
            },
            0,
        );
        assert_eq!(
            pick(&renderer, [400, 300], expected as u64)?,
            expected,
            "Depth/front-face picking failed"
        );
    }
    for size in [[311, 197], [1600, 1200]] {
        renderer.resize(size);
        let texture = target(&device, size, wgpu::TextureFormat::Rgba8UnormSrgb);
        renderer.render(&texture.create_view(&Default::default()), &camera, 0);
        assert!((1..=6).contains(&pick(&renderer, [size[0] / 2, size[1] / 2], 99)?));
    }
    if let Some(parent) = Path::new(&path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let mut encoder = png::Encoder::new(BufWriter::new(File::create(&path)?), 800, 600);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&image)?;
    println!(
        "Passed: five distinct visual styles, preview isolation and cancellation, six face IDs, depth occlusion, background miss, selection shading, appearance colors, color encoding, revision stamps and resize. Saved {path}"
    );
    Ok(())
}
