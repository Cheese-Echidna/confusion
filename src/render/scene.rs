//! Immutable proof-scene mesh: six independently identifiable faces of a Z-up cube.
//!
//! Exports DemoVertex, demo_cube and face_name for the renderer and UI. This mesh is
//! a viewport fixture, not parametric document data or a substitute for OCCT geometry.

#[cfg(feature = "gpu")]
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DemoVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub face: u32,
}

pub fn face_name(id: u32) -> &'static str {
    match id {
        1 => "+X face",
        2 => "−X face",
        3 => "+Y face",
        4 => "−Y face",
        5 => "Top (+Z)",
        6 => "Bottom (−Z)",
        _ => "Nothing selected",
    }
}

#[cfg(feature = "gpu")]
pub fn demo_cube() -> (Vec<DemoVertex>, Vec<u16>) {
    let faces = [
        (
            [1., 0., 0.],
            [[1., -1., -1.], [1., 1., -1.], [1., 1., 1.], [1., -1., 1.]],
        ),
        (
            [-1., 0., 0.],
            [
                [-1., 1., -1.],
                [-1., -1., -1.],
                [-1., -1., 1.],
                [-1., 1., 1.],
            ],
        ),
        (
            [0., 1., 0.],
            [[1., 1., -1.], [-1., 1., -1.], [-1., 1., 1.], [1., 1., 1.]],
        ),
        (
            [0., -1., 0.],
            [
                [-1., -1., -1.],
                [1., -1., -1.],
                [1., -1., 1.],
                [-1., -1., 1.],
            ],
        ),
        (
            [0., 0., 1.],
            [[-1., -1., 1.], [1., -1., 1.], [1., 1., 1.], [-1., 1., 1.]],
        ),
        (
            [0., 0., -1.],
            [
                [-1., 1., -1.],
                [1., 1., -1.],
                [1., -1., -1.],
                [-1., -1., -1.],
            ],
        ),
    ];
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    for (index, (normal, positions)) in faces.into_iter().enumerate() {
        let base = vertices.len() as u16;
        vertices.extend(positions.map(|position| DemoVertex {
            position: position.map(|value| value * 0.75),
            normal,
            face: index as u32 + 1,
        }));
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    (vertices, indices)
}

#[cfg(all(test, feature = "gpu"))]
mod tests {
    use super::*;
    #[test]
    fn faces_are_outward_wound_and_have_distinct_pick_ids() {
        let (vertices, indices) = demo_cube();
        for triangle in indices.as_chunks::<3>().0 {
            let a = vertices[triangle[0] as usize];
            let b = vertices[triangle[1] as usize];
            let c = vertices[triangle[2] as usize];
            let point = |p: [f32; 3]| nalgebra::Vector3::from(p);
            assert!(
                (point(b.position) - point(a.position))
                    .cross(&(point(c.position) - point(a.position)))
                    .dot(&point(a.normal))
                    > 0.0
            );
            assert!((1..=6).contains(&a.face));
        }
    }
}
