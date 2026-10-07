//! Double-precision Z-up orbit camera for the viewport proof.
//!
//! Exports Camera and ProjectionMode. UI navigation supplies logical-pixel deltas;
//! renderer consumes its WebGPU view-projection matrix. Domain geometry stays f64.

use nalgebra::{
    Matrix3, Matrix4, Perspective3, Point3, Rotation3, UnitQuaternion, Vector3, Vector4,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionMode {
    Perspective,
    Orthographic,
}

#[derive(Clone, Debug)]
pub struct Camera {
    pub target: Point3<f64>,
    pub yaw: f64,
    pub elevation: f64,
    pub distance: f64,
    pub projection: ProjectionMode,
    pub orientation: Option<UnitQuaternion<f64>>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target: Point3::origin(),
            yaw: 45_f64.to_radians(),
            elevation: 30_f64.to_radians(),
            distance: 6.0,
            projection: ProjectionMode::Perspective,
            orientation: None,
        }
    }
}

impl Camera {
    pub const FIELD_OF_VIEW: f64 = std::f64::consts::FRAC_PI_4;

    pub fn outward(&self) -> Vector3<f64> {
        self.orientation.map_or_else(
            || {
                Vector3::new(
                    self.elevation.cos() * self.yaw.cos(),
                    self.elevation.cos() * self.yaw.sin(),
                    self.elevation.sin(),
                )
            },
            |q| q * Vector3::z(),
        )
    }
    pub fn up(&self) -> Vector3<f64> {
        self.orientation.map_or_else(
            || {
                let out = self.outward();
                let right = Vector3::z().cross(&out).normalize();
                out.cross(&right).normalize()
            },
            |q| q * Vector3::y(),
        )
    }
    pub fn right(&self) -> Vector3<f64> {
        self.up().cross(&self.outward()).normalize()
    }
    pub fn eye(&self) -> Point3<f64> {
        self.target + self.distance * self.outward()
    }
    pub fn set_direction(&mut self, direction: Vector3<f64>, up: Vector3<f64>) {
        let out = direction.normalize();
        let right = up.cross(&out).normalize();
        let up = out.cross(&right);
        self.orientation = Some(UnitQuaternion::from_rotation_matrix(
            &Rotation3::from_matrix_unchecked(Matrix3::from_columns(&[right, up, out])),
        ));
        self.yaw = out.y.atan2(out.x);
        self.elevation = out.z.asin();
    }
    pub fn free_orbit(&mut self, delta: [f64; 2]) {
        let q = self.orientation.unwrap_or_else(|| {
            UnitQuaternion::from_rotation_matrix(&Rotation3::from_matrix_unchecked(
                Matrix3::from_columns(&[self.right(), self.up(), self.outward()]),
            ))
        });
        self.orientation = Some(
            q * UnitQuaternion::from_scaled_axis(Vector3::new(
                delta[1] * 0.008,
                -delta[0] * 0.008,
                0.,
            )),
        );
    }
    pub fn orbit(&mut self, delta: [f64; 2]) {
        let out = self.outward();
        self.yaw = out.y.atan2(out.x);
        self.elevation = out.z.asin();
        self.orientation = None;
        self.yaw = (self.yaw - delta[0] * 0.008).rem_euclid(std::f64::consts::TAU);
        self.elevation = (self.elevation + delta[1] * 0.008).clamp(-1.5, 1.5);
    }

    pub fn pan(&mut self, delta: [f64; 2], viewport_height: f64) {
        let right = self.right();
        let up = self.up();
        let scale =
            2.0 * self.distance * (Self::FIELD_OF_VIEW / 2.0).tan() / viewport_height.max(1.0);
        self.target += (-right * delta[0] + up * delta[1]) * scale;
    }

    pub fn zoom(&mut self, scroll_pixels: f64) {
        self.distance = (self.distance * (-scroll_pixels * 0.0025).exp()).clamp(1.5, 100.0);
    }

    pub fn view_projection(&self, width: u32, height: u32) -> Matrix4<f64> {
        let aspect = width.max(1) as f64 / height.max(1) as f64;
        let view = Matrix4::look_at_rh(&self.eye(), &self.target, &self.up());
        let projection = match self.projection {
            ProjectionMode::Perspective => {
                Perspective3::new(aspect, Self::FIELD_OF_VIEW, 0.05, 500.0).to_homogeneous()
            }
            ProjectionMode::Orthographic => {
                let half_height = self.distance * (Self::FIELD_OF_VIEW / 2.0).tan();
                nalgebra::Orthographic3::new(
                    -half_height * aspect,
                    half_height * aspect,
                    -half_height,
                    half_height,
                    0.05,
                    500.0,
                )
                .to_homogeneous()
            }
        };
        // nalgebra projections use OpenGL [-1, 1] depth; WebGPU requires [0, 1].
        let mut depth_conversion = Matrix4::identity();
        depth_conversion[(2, 2)] = 0.5;
        depth_conversion[(2, 3)] = 0.5;
        depth_conversion * projection * view
    }

    pub fn gpu_matrix(&self, width: u32, height: u32) -> [[f32; 4]; 4] {
        let matrix = self.view_projection(width, height);
        std::array::from_fn(|column| std::array::from_fn(|row| matrix[(row, column)] as f32))
    }

    /// Used by geometric tests and future exact refinement, never as the GPU pick result.
    pub fn ray(&self, pixel: [f64; 2], size: [u32; 2]) -> Option<(Point3<f64>, Vector3<f64>)> {
        if size.contains(&0) {
            return None;
        }
        let inverse = self.view_projection(size[0], size[1]).try_inverse()?;
        let x = 2.0 * pixel[0] / size[0] as f64 - 1.0;
        let y = 1.0 - 2.0 * pixel[1] / size[1] as f64;
        let near = inverse * Vector4::new(x, y, 0.0, 1.0);
        let far = inverse * Vector4::new(x, y, 1.0, 1.0);
        let origin = Point3::from(near.xyz() / near.w);
        let direction = (far.xyz() / far.w - origin.coords).normalize();
        Some((origin, direction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_projects_to_viewport_center_in_both_modes() {
        for projection in [ProjectionMode::Perspective, ProjectionMode::Orthographic] {
            let camera = Camera {
                projection,
                ..Camera::default()
            };
            let clip = camera.view_projection(800, 600) * camera.target.to_homogeneous();
            assert!((clip.x / clip.w).abs() < 1e-12);
            assert!((clip.y / clip.w).abs() < 1e-12);
            assert!((0.0..1.0).contains(&(clip.z / clip.w)));
            let (origin, direction) = camera.ray([400.0, 300.0], [800, 600]).unwrap();
            assert!((camera.target - origin).normalize().dot(&direction) > 0.999999);
        }
    }

    #[test]
    fn pan_is_screen_aligned_and_independent_of_dpi() {
        let mut camera = Camera::default();
        let eye_offset = camera.eye() - camera.target;
        camera.pan([100.0, 0.0], 600.0);
        assert!(camera.target.coords.norm() > 0.1);
        assert!((camera.eye() - camera.target - eye_offset).norm() < 1e-12);
        assert!(camera.target.coords.dot(&eye_offset).abs() < 1e-12);
    }

    #[test]
    fn extreme_navigation_stays_finite_and_avoids_pole_singularity() {
        let mut camera = Camera::default();
        camera.orbit([1e6, 1e6]);
        camera.zoom(1e6);
        camera.pan([12.0, -5.0], 0.0);
        assert!(
            camera
                .view_projection(0, 0)
                .iter()
                .all(|value| value.is_finite())
        );
        assert!(camera.ray([0.0, 0.0], [0, 0]).is_none());
        camera.zoom(-1e6);
        assert_eq!(camera.distance, 100.0);
    }
}
