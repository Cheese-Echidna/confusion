//! Camera-linked orientation cube geometry and picking, independent of GPUI state.
//! Exports faces/pick; the workspace paints these 3D faces as a gizmo, not icon artwork.
//! Face directions use Z-up; exact pole views and rolled free-orbit views remain valid.
use crate::render::camera::Camera;
use nalgebra::Vector3;
pub struct CubeFace {
    pub label: &'static str,
    pub direction: Vector3<f64>,
    pub up: Vector3<f64>,
    pub polygon: [[f64; 2]; 4],
}
pub fn faces(camera: &Camera) -> Vec<CubeFace> {
    let sides = [
        ("Right", Vector3::x(), Vector3::z()),
        ("Left", -Vector3::x(), Vector3::z()),
        ("Back", Vector3::y(), Vector3::z()),
        ("Front", -Vector3::y(), Vector3::z()),
        ("Top", Vector3::z(), Vector3::y()),
        ("Bottom", -Vector3::z(), -Vector3::y()),
    ];
    sides
        .into_iter()
        .filter(|(_, normal, _)| normal.dot(&camera.outward()) > 1e-6)
        .map(|(label, direction, up)| {
            let tangent = if direction.z.abs() > 0.5 {
                Vector3::x()
            } else {
                Vector3::z()
            };
            let bitangent = direction.cross(&tangent);
            let corners = [
                direction - tangent - bitangent,
                direction + tangent - bitangent,
                direction + tangent + bitangent,
                direction - tangent + bitangent,
            ];
            CubeFace {
                label,
                direction,
                up,
                polygon: corners.map(|p| {
                    [
                        62. + p.dot(&camera.right()) * 24.,
                        52. - p.dot(&camera.up()) * 24.,
                    ]
                }),
            }
        })
        .collect()
}
pub struct CubeAxis {
    pub origin: [f64; 2],
    pub end: [f64; 2],
    pub color: u32,
}
/// Use the nearest visible corner so all three colored edges remain attached to the cube.
pub fn axes(camera: &Camera) -> [CubeAxis; 3] {
    let out = camera.outward();
    let corner = out.map(|v| if v < 0. { -1. } else { 1. });
    let project = |p: Vector3<f64>| {
        [
            62. + p.dot(&camera.right()) * 24.,
            52. - p.dot(&camera.up()) * 24.,
        ]
    };
    std::array::from_fn(|axis| {
        let mut end = corner;
        end[axis] *= -1.;
        CubeAxis {
            origin: project(corner),
            end: project(end),
            color: [0xe86559, 0x8bbc6b, 0x5a9be6][axis],
        }
    })
}
pub fn pick(camera: &Camera, p: [f64; 2]) -> Option<CubeFace> {
    faces(camera).into_iter().find(|face| {
        let mut sign = 0.;
        for i in 0..4 {
            let a = face.polygon[i];
            let b = face.polygon[(i + 1) % 4];
            let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
            if cross.abs() < 1e-9 {
                continue;
            }
            if sign == 0. {
                sign = cross.signum();
            } else if sign * cross < 0. {
                return false;
            }
        }
        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn axes_share_a_corner_and_follow_visible_cube_edges() {
        let mut camera = Camera::default();
        for direction in [
            Vector3::new(1., 1., 1.),
            Vector3::new(-1., 2., 0.5),
            Vector3::z(),
        ] {
            camera.set_direction(
                direction,
                if direction == Vector3::z() {
                    Vector3::y()
                } else {
                    Vector3::z()
                },
            );
            let faces = faces(&camera);
            let axes = axes(&camera);
            for axis in &axes {
                assert_eq!(axis.origin, axes[0].origin);
                assert!(faces.iter().any(|face| face.polygon.contains(&axis.origin)));
                assert!(faces.iter().any(|face| face.polygon.contains(&axis.end)));
            }
            for face in faces {
                assert!(["Top", "Bottom", "Front", "Back", "Left", "Right"].contains(&face.label));
            }
        }
    }
}
