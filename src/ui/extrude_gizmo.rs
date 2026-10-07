//! Extrusion preview and projected depth handle share the viewport camera.
#[cfg(feature = "desktop")]
use gpui::{prelude::*, *};
#[cfg(feature = "desktop")]
pub struct ExtrudeGizmo {
    pub camera: crate::render::camera::Camera,
    pub frame: crate::sketch::workplane::Workplane,
    pub curves: Vec<Vec<[f64; 2]>>,
    pub center: [f64; 2],
    pub depth: f64,
}
#[cfg(feature = "desktop")]
impl ExtrudeGizmo {
    pub fn element(self) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let screen = |at: [f64; 2], depth: f64| {
                    let p = self.frame.world(at) + self.frame.normal * depth;
                    self.camera
                        .project(
                            nalgebra::Point3::from(p.coords * 25.),
                            [
                                f64::from(bounds.size.width) as u32,
                                f64::from(bounds.size.height) as u32,
                            ],
                        )
                        .map(|p| bounds.origin + point(px(p[0] as f32), px(p[1] as f32)))
                };
                let stroke = |a, b, width, window: &mut Window| {
                    let mut path = PathBuilder::stroke(px(width));
                    path.move_to(a);
                    path.line_to(b);
                    if let Ok(path) = path.build() {
                        window.paint_path(path, rgb(super::theme::WARNING));
                    }
                };
                for curve in &self.curves {
                    for pair in curve.windows(2) {
                        if let (Some(a), Some(b)) =
                            (screen(pair[0], self.depth), screen(pair[1], self.depth))
                        {
                            stroke(a, b, 1., window);
                        }
                    }
                    for at in curve.iter().step_by((curve.len() / 8).max(1)) {
                        if let (Some(a), Some(b)) = (screen(*at, 0.), screen(*at, self.depth)) {
                            stroke(a, b, 1., window);
                        }
                    }
                }
                if let (Some(a), Some(b)) =
                    (screen(self.center, 0.), screen(self.center, self.depth))
                {
                    stroke(a, b, 4., window);
                    let delta = b - a;
                    let len = f64::from(delta.x).hypot(f64::from(delta.y)).max(1.) as f32;
                    let tangent = delta / len;
                    let normal = point(-tangent.y, tangent.x);
                    let mut head = PathBuilder::fill();
                    head.move_to(b);
                    head.line_to(b - tangent * 22. + normal * 11.);
                    head.line_to(b - tangent * 22. - normal * 11.);
                    head.line_to(b);
                    if let Ok(head) = head.build() {
                        window.paint_path(head, rgb(super::theme::WARNING));
                    }
                }
            },
        )
        .absolute()
        .inset_0()
    }
}
