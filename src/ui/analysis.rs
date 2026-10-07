//! Non-destructive inspection of evaluated solids. Section previews are tessellation
//! based; mass, area, validity, intersections and surface samples come from OCCT.
#[cfg(feature = "desktop")]
pub fn clipped_mesh(
    vertices: &[crate::render::scene::DemoVertex],
    indices: &[u32],
    axis: usize,
    offset: f32,
) -> (Vec<crate::render::scene::DemoVertex>, Vec<u32>) {
    use crate::render::scene::DemoVertex;
    let mut output = Vec::new();
    let mut triangles = Vec::new();
    for triangle in indices.as_chunks::<3>().0 {
        let mut polygon: Vec<DemoVertex> = triangle.iter().map(|&i| vertices[i as usize]).collect();
        let input = std::mem::take(&mut polygon);
        for i in 0..input.len() {
            let a = input[i];
            let b = input[(i + 1) % input.len()];
            let inside_a = a.position[axis] <= offset;
            let inside_b = b.position[axis] <= offset;
            if inside_a {
                polygon.push(a);
            }
            if inside_a != inside_b {
                let t = (offset - a.position[axis]) / (b.position[axis] - a.position[axis]);
                let mut intersection = a;
                for j in 0..3 {
                    intersection.position[j] = a.position[j] + t * (b.position[j] - a.position[j]);
                }
                polygon.push(intersection);
            }
        }
        for i in 1..polygon.len().saturating_sub(1) {
            for vertex in [polygon[0], polygon[i], polygon[i + 1]] {
                triangles.push(output.len() as u32);
                output.push(vertex);
            }
        }
    }
    (output, triangles)
}

#[cfg(all(test, feature = "desktop"))]
mod tests {
    use super::*;
    #[test]
    fn section_clips_crossing_triangles_and_preserves_picking() {
        use crate::render::scene::DemoVertex;
        let vertices = [[-1., 0., 0.], [1., 0., 0.], [1., 1., 0.]].map(|position| DemoVertex {
            position,
            normal: [0., 0., 1.],
            face: 7,
            color: [0.5; 3],
        });
        let (v, i) = clipped_mesh(&vertices, &[0, 1, 2], 0, 0.);
        assert_eq!(i.len(), 3);
        assert!(v.iter().all(|v| v.position[0] <= 0. && v.face == 7));
        assert!(clipped_mesh(&vertices, &[0, 1, 2], 0, -2.).0.is_empty());
        assert_eq!(clipped_mesh(&vertices, &[0, 1, 2], 0, 2.).1.len(), 3);
    }
}
