//! GPU-composited planar sketch overlay, separated from workspace chrome and gestures.
//! Exports SketchCanvas; consumes solved display values and captures viewport bounds.
//! Grid/axes/visibility are view settings, never persisted into design geometry.
#[cfg(feature = "desktop")]
mod implementation {
    use super::super::theme as t;
    use gpui::{prelude::*, *};
    use std::{cell::Cell, rc::Rc};
    use uuid::Uuid;
    pub type LinearDimension = ([[f64; 2]; 2], [f64; 2], Option<[f64; 2]>, bool);
    pub struct SketchCanvas {
        pub camera: crate::render::camera::Camera,
        pub frame: crate::sketch::workplane::Workplane,
        pub placement_preview: bool,
        pub bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
        pub active: bool,
        pub visible: bool,
        pub grid: bool,
        pub axes: bool,
        pub coords: Vec<[f64; 2]>,
        pub lines: Vec<(Uuid, Vec<[f64; 2]>, u32, bool)>,
        pub preview: Vec<[f64; 2]>,
        pub preview_curves: Vec<Vec<[f64; 2]>>,
        pub region_wires: Vec<Vec<[f64; 2]>>,
        pub dimensions: Vec<LinearDimension>,
        pub point_colors: Vec<u32>,
        pub annotations: Vec<([f64; 2], String)>,
        pub selected: Vec<Uuid>,
        pub anchor: Option<[f64; 2]>,
        pub hover: Option<[f64; 2]>,
        pub rectangle: bool,
        pub scale: f64,
        pub center: [f64; 2],
    }
    impl SketchCanvas {
        pub fn element(self) -> impl IntoElement {
            canvas(
                move |bounds, _, _| self.bounds.set(Some(bounds)),
                move |bounds, _, window, cx| {
                    if !self.active {
                        return;
                    }
                    let screen = |p: [f64; 2]| {
                        let world = self.frame.world(p);
                        let projected = self
                            .camera
                            .project(
                                nalgebra::Point3::from(world.coords * 25.),
                                [
                                    f64::from(bounds.size.width).max(1.) as u32,
                                    f64::from(bounds.size.height).max(1.) as u32,
                                ],
                            )
                            .unwrap_or([-10000., -10000.]);
                        bounds.origin + point(px(projected[0] as f32), px(projected[1] as f32))
                    };
                    let stroke = |a: Point<Pixels>,
                                  b: Point<Pixels>,
                                  color: u32,
                                  width: f32,
                                  window: &mut Window| {
                        let mut path = PathBuilder::stroke(px(width));
                        path.move_to(a);
                        path.line_to(b);
                        if let Ok(path) = path.build() {
                            window.paint_path(path, rgb(color));
                        }
                    };
                    let spacing = if self.scale > 3000. { 0.005 } else { 0.01 };
                    if self.grid || self.axes {
                        let min_x =
                            self.center[0] - f64::from(bounds.size.width) * 0.5 / self.scale;
                        let max_x =
                            self.center[0] + f64::from(bounds.size.width) * 0.5 / self.scale;
                        let min_y =
                            self.center[1] - f64::from(bounds.size.height) * 0.5 / self.scale;
                        let max_y =
                            self.center[1] + f64::from(bounds.size.height) * 0.5 / self.scale;
                        for i in (min_x / spacing).floor() as i32..=(max_x / spacing).ceil() as i32
                        {
                            if (i == 0 && !self.axes) || (i != 0 && !self.grid) {
                                continue;
                            }

                            stroke(
                                screen([f64::from(i) * spacing, min_y]),
                                screen([f64::from(i) * spacing, max_y]),
                                if i == 0 { t::SUCCESS } else { 0x343330 },
                                1.,
                                window,
                            );
                        }
                        for i in (min_y / spacing).floor() as i32..=(max_y / spacing).ceil() as i32
                        {
                            if (i == 0 && !self.axes) || (i != 0 && !self.grid) {
                                continue;
                            }

                            stroke(
                                screen([min_x, f64::from(i) * spacing]),
                                screen([max_x, f64::from(i) * spacing]),
                                if i == 0 { t::ERROR } else { 0x343330 },
                                1.,
                                window,
                            );
                        }
                    }
                    if !self.visible {
                        return;
                    }
                    if !self.region_wires.is_empty() {
                        let mut path = PathBuilder::fill();
                        for wire in &self.region_wires {
                            if let Some(first) = wire.first() {
                                path.move_to(screen(*first));
                                for p in wire.iter().skip(1) {
                                    path.line_to(screen(*p));
                                }
                                path.close();
                            }
                        }
                        if let Ok(path) = path.build() {
                            window.paint_path(path, rgba((t::WARNING << 8) | 0x28));
                        }
                    }
                    for (id, points, color, construction) in &self.lines {
                        for [a, b] in points.windows(2).map(|p| [p[0], p[1]]) {
                            let a = screen(a);
                            let b = screen(b);
                            let color = if self.selected.contains(id) {
                                t::WARNING
                            } else {
                                *color
                            };
                            if *construction {
                                let delta = b - a;
                                let length = f64::from(delta.x).hypot(f64::from(delta.y));
                                let segments = (length / 6.).ceil().max(1.) as usize;
                                for i in (0..segments).step_by(2) {
                                    let start = i as f32 / segments as f32;
                                    let end = ((i + 1) as f32 / segments as f32).min(1.);
                                    stroke(a + delta * start, a + delta * end, color, 1.5, window)
                                }
                            } else {
                                stroke(a, b, color, 1.5, window)
                            }
                        }
                    }
                    for (index, p) in self.coords.iter().enumerate() {
                        let p = screen(*p);
                        window.paint_quad(fill(
                            Bounds::new(p - point(px(2.5), px(2.5)), size(px(5.), px(5.))),
                            rgb(self.point_colors.get(index).copied().unwrap_or(t::ACCENT)),
                        ));
                    }
                    for (ends, at, direction, radial) in &self.dimensions {
                        let a = screen(ends[0]);
                        let b = screen(ends[1]);
                        if *radial {
                            let delta = [at[0] - ends[0][0], at[1] - ends[0][1]];
                            let length = delta[0].hypot(delta[1]).max(1e-12);
                            let radius = (ends[1][0] - ends[0][0]).hypot(ends[1][1] - ends[0][1]);
                            let rim = screen([
                                ends[0][0] + delta[0] / length * radius,
                                ends[0][1] + delta[1] / length * radius,
                            ]);
                            stroke(a, rim, t::MUTED, 1., window);
                            stroke(rim, screen(*at), t::MUTED, 1., window);
                            let tick = point(px(3.), px(-3.));
                            stroke(rim - tick, rim + tick, t::MUTED, 1., window);
                            continue;
                        }
                        let delta =
                            direction.unwrap_or([ends[1][0] - ends[0][0], ends[1][1] - ends[0][1]]);
                        let len = delta[0].hypot(delta[1]).max(1e-12);
                        let normal = [-delta[1] / len, delta[0] / len];
                        let project = |p: [f64; 2]| {
                            let offset = (at[0] - p[0]) * normal[0] + (at[1] - p[1]) * normal[1];
                            screen([p[0] + normal[0] * offset, p[1] + normal[1] * offset])
                        };
                        let aa = project(ends[0]);
                        let bb = project(ends[1]);
                        stroke(a, aa, t::MUTED, 1., window);
                        stroke(b, bb, t::MUTED, 1., window);
                        stroke(aa, bb, t::MUTED, 1., window);
                        let tick = point(px(4.), px(-4.));
                        stroke(aa - tick, aa + tick, t::MUTED, 1., window);
                        stroke(bb - tick, bb + tick, t::MUTED, 1., window);
                    }
                    for curve in &self.preview_curves {
                        for pair in curve.windows(2) {
                            stroke(screen(pair[0]), screen(pair[1]), t::WARNING, 1.5, window);
                        }
                    }
                    for pair in self.preview.windows(2) {
                        stroke(screen(pair[0]), screen(pair[1]), t::WARNING, 1.5, window);
                    }
                    for (index, (position, text)) in self.annotations.iter().enumerate() {
                        let offset = self.annotations[..index]
                            .iter()
                            .filter(|(p, _)| {
                                (p[0] - position[0]).abs() < 1e-7
                                    && (p[1] - position[1]).abs() < 1e-7
                            })
                            .count() as f32;
                        let style = window.text_style();
                        let text: SharedString = text.clone().into();
                        let run = TextRun {
                            len: text.len(),
                            font: style.font(),
                            color: rgb(t::MUTED).into(),
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                            letter_spacing: None,
                        };
                        let line = window.text_system().shape_line(text, px(11.), &[run], None);
                        let _ = line.paint(
                            screen(*position) + point(px(7.), px(-14. - offset * 15.)),
                            px(15.),
                            window,
                            cx,
                        );
                    }
                    if self.placement_preview
                        && let Some(at) = self.hover
                    {
                        let p = screen(at);
                        stroke(
                            p - point(px(7.), px(0.)),
                            p + point(px(7.), px(0.)),
                            t::WARNING,
                            1.,
                            window,
                        );
                        stroke(
                            p - point(px(0.), px(7.)),
                            p + point(px(0.), px(7.)),
                            t::WARNING,
                            1.,
                            window,
                        );
                        window.paint_quad(fill(
                            Bounds::new(p - point(px(2.), px(2.)), size(px(4.), px(4.))),
                            rgb(t::WARNING),
                        ));
                    }
                    if let (Some(a), Some(b)) = (self.anchor, self.hover) {
                        if self.rectangle {
                            let corners = [a, [b[0], a[1]], b, [a[0], b[1]]];
                            for i in 0..4 {
                                stroke(
                                    screen(corners[i]),
                                    screen(corners[(i + 1) % 4]),
                                    t::WARNING,
                                    1.5,
                                    window,
                                );
                            }
                        } else {
                            stroke(screen(a), screen(b), t::WARNING, 1.5, window);
                        }
                    }
                },
            )
            .absolute()
            .inset_0()
        }
    }
}
#[cfg(feature = "desktop")]
pub use implementation::*;
