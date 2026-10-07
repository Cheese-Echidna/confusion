//! GPU-composited planar sketch overlay, separated from workspace chrome and gestures.
//! Exports SketchCanvas; consumes solved display values and captures viewport bounds.
//! Grid/axes/visibility are view settings, never persisted into design geometry.
#[cfg(feature = "desktop")]
mod implementation {
    use super::super::theme as t;
    use gpui::{prelude::*, *};
    use std::{cell::Cell, rc::Rc};
    use uuid::Uuid;
    pub struct SketchCanvas {
        pub bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
        pub active: bool,
        pub visible: bool,
        pub grid: bool,
        pub axes: bool,
        pub coords: Vec<[f64; 2]>,
        pub lines: Vec<(Uuid, [[f64; 2]; 2])>,
        pub annotations: Vec<([f64; 2], String)>,
        pub selected: Option<Uuid>,
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
                        point(
                            bounds.origin.x
                                + bounds.size.width * 0.5
                                + px(((p[0] - self.center[0]) * self.scale) as f32),
                            bounds.origin.y + bounds.size.height * 0.5
                                - px(((p[1] - self.center[1]) * self.scale) as f32),
                        )
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
                            let x = screen([f64::from(i) * spacing, self.center[1]]).x;
                            stroke(
                                point(x, bounds.top()),
                                point(x, bounds.bottom()),
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
                            let y = screen([self.center[0], f64::from(i) * spacing]).y;
                            stroke(
                                point(bounds.left(), y),
                                point(bounds.right(), y),
                                if i == 0 { t::ERROR } else { 0x343330 },
                                1.,
                                window,
                            );
                        }
                    }
                    if !self.visible {
                        return;
                    }
                    for (id, [a, b]) in &self.lines {
                        stroke(
                            screen(*a),
                            screen(*b),
                            if Some(*id) == self.selected {
                                t::WARNING
                            } else {
                                t::ACCENT
                            },
                            1.5,
                            window,
                        );
                    }
                    for p in &self.coords {
                        let p = screen(*p);
                        window.paint_quad(fill(
                            Bounds::new(p - point(px(2.), px(2.)), size(px(4.), px(4.))),
                            rgb(t::TEXT),
                        ));
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
                            screen(*position) + point(px(7.), px(-18. - offset * 15.)),
                            px(15.),
                            window,
                            cx,
                        );
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
