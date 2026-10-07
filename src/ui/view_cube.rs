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
