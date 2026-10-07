//! Derived orthonormal sketch frame in the same SI world as evaluated solids.
use nalgebra::{Point3, Vector3};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Workplane {
    pub origin: Point3<f64>,
    pub x: Vector3<f64>,
    pub y: Vector3<f64>,
    pub normal: Vector3<f64>,
}
impl Default for Workplane {
    fn default() -> Self {
        Self {
            origin: Point3::origin(),
            x: Vector3::x(),
            y: Vector3::y(),
            normal: Vector3::z(),
        }
    }
}
impl Workplane {
    pub fn world(&self, local: [f64; 2]) -> Point3<f64> {
        self.origin + self.x * local[0] + self.y * local[1]
    }
    pub fn local(&self, world: Point3<f64>) -> [f64; 2] {
        let d = world - self.origin;
        [d.dot(&self.x), d.dot(&self.y)]
    }
    pub fn intersect(&self, origin: Point3<f64>, direction: Vector3<f64>) -> Option<[f64; 2]> {
        let denominator = direction.dot(&self.normal);
        if denominator.abs() < 1e-8 {
            return None;
        }
        let distance = (self.origin - origin).dot(&self.normal) / denominator;
        (distance >= 0.).then(|| self.local(origin + direction * distance))
    }
}
#[cfg(feature = "kernel")]
impl From<&crate::kernel::bridge::ffi::PlaneFrame> for Workplane {
    fn from(p: &crate::kernel::bridge::ffi::PlaneFrame) -> Self {
        Self {
            origin: Point3::new(p.ox, p.oy, p.oz),
            x: Vector3::new(p.xx, p.xy, p.xz),
            y: Vector3::new(p.yx, p.yy, p.yz),
            normal: Vector3::new(p.nx, p.ny, p.nz),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotated_plane_round_trip_and_ray_hit() {
        let p = Workplane {
            origin: Point3::new(0.1, 0.2, 0.3),
            x: Vector3::y(),
            y: Vector3::z(),
            normal: Vector3::x(),
        };
        let world = p.world([0.04, 0.05]);
        assert!((p.local(world)[0] - 0.04).abs() < 1e-12);
        let hit = p.intersect(world + Vector3::x(), -Vector3::x()).unwrap();
        assert!((hit[1] - 0.05).abs() < 1e-12);
        assert!(p.intersect(world, Vector3::y()).is_none());
    }
}
