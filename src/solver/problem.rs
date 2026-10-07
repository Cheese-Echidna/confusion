//! Constraint residuals in millimetres. Circle rim orientation is a parameterization gauge,
//! not design intent; arcs retain endpoint angles as physical degrees of freedom.
use crate::document::schema::{ConstraintKind as C, Design};
use std::collections::HashMap;
use uuid::Uuid;
pub fn residual(d: &Design, c: &C, x: &[f64], parameters: &HashMap<Uuid, f64>) -> Vec<f64> {
    let point = |id| {
        let i = d.points.iter().position(|p| p.id == id).unwrap() * 2;
        [x[i], x[i + 1]]
    };
    let line = |id| d.lines.iter().find(|l| l.id == id).unwrap().ends.map(point);
    let circle = |id| {
        let c = d.circles.iter().find(|c| c.id == id).unwrap();
        let center = point(c.center);
        (center, distance(center, point(c.rim)))
    };
    let direction = |id| {
        let [a, b] = line(id);
        sub(b, a)
    };
    let length = |id| {
        if d.lines.iter().any(|l| l.id == id) {
            let [a, b] = line(id);
            distance(a, b)
        } else {
            circle(id).1
        }
    };
    match c {
        C::ProjectedAlignment { points, direction } => {
            vec![dot(sub(point(points[1]), point(points[0])), *direction)]
        }
        C::HorizontalPoints { points } => vec![point(points[1])[1] - point(points[0])[1]],
        C::VerticalPoints { points } => vec![point(points[1])[0] - point(points[0])[0]],
        C::LineOffset {
            lines,
            parameter,
            side,
        } => {
            let [a, b] = line(lines[0]);
            let [c, e] = line(lines[1]);
            vec![
                cross(sub(b, a), sub(c, a)) / distance(a, b).max(1e-8)
                    - parameters[parameter] * 1000. * side,
                cross(sub(b, a), sub(e, c)) / distance(a, b).max(1e-8),
            ]
        }
        C::OffsetRadius { circles, parameter } => {
            vec![circle(circles[1]).1 - circle(circles[0]).1 - parameters[parameter] * 1000.]
        }
        C::Direction { line, angle } => {
            let a = direction(*line);
            let current = a[1].atan2(a[0]);
            vec![
                ((current - angle + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
                    - std::f64::consts::PI)
                    * 10.,
            ]
        }
        C::ProjectedDistance {
            points,
            parameter,
            direction,
        } => vec![
            dot(sub(point(points[1]), point(points[0])), *direction)
                - parameters[parameter] * 1000.,
        ],
        C::Horizontal { line: id } => vec![direction(*id)[1]],
        C::Vertical { line: id } => vec![direction(*id)[0]],
        C::Fixed { point: id, xy } => sub(point(*id), xy.map(|v| v * 1000.)).to_vec(),
        C::Coincident { points } => sub(point(points[0]), point(points[1])).to_vec(),
        C::DistanceX { points, parameter } => {
            vec![point(points[1])[0] - point(points[0])[0] - parameters[parameter] * 1000.]
        }
        C::DistanceY { points, parameter } => {
            vec![point(points[1])[1] - point(points[0])[1] - parameters[parameter] * 1000.]
        }
        C::Distance { points, parameter } => {
            vec![distance(point(points[0]), point(points[1])) - parameters[parameter] * 1000.]
        }
        C::Length { line, parameter } => vec![length(*line) - parameters[parameter] * 1000.],
        C::Parallel { lines } => vec![
            cross(direction(lines[0]), direction(lines[1])) / norm(direction(lines[0])).max(1e-8),
        ],
        C::Perpendicular { lines } => vec![
            dot(direction(lines[0]), direction(lines[1])) / norm(direction(lines[0])).max(1e-8),
        ],
        C::Equal { curves } => vec![length(curves[0]) - length(curves[1])],
        C::Collinear { lines } => {
            let a = direction(lines[0]);
            let b = direction(lines[1]);
            vec![
                cross(a, b) / norm(a).max(1e-8),
                cross(a, sub(line(lines[1])[0], line(lines[0])[0])) / norm(a).max(1e-8),
            ]
        }
        C::Midpoint { point: p, line: l } => {
            let [a, b] = line(*l);
            sub(point(*p), [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]).to_vec()
        }
        C::PointOnLine { point: p, line: l } => {
            let [a, b] = line(*l);
            vec![cross(sub(b, a), sub(point(*p), a)) / distance(a, b).max(1e-8)]
        }
        C::PointOnCircle {
            point: p,
            circle: id,
        } => {
            let (a, r) = circle(*id);
            vec![distance(a, point(*p)) - r]
        }
        C::Concentric { circles } => sub(circle(circles[0]).0, circle(circles[1]).0).to_vec(),
        C::Tangent { curves } => {
            if let Some(l) = curves
                .iter()
                .find(|id| d.lines.iter().any(|l| l.id == **id))
            {
                let id = *curves.iter().find(|id| *id != l).unwrap();
                let (c, r) = circle(id);
                let [a, b] = line(*l);
                vec![cross(sub(b, a), sub(c, a)).abs() / distance(a, b).max(1e-8) - r]
            } else {
                let (a, r) = circle(curves[0]);
                let (b, s) = circle(curves[1]);
                // Preserve the internal/external branch from the current sketch seed.
                let seed = |id| {
                    let c = d.circles.iter().find(|c| c.id == id).unwrap();
                    let p = |id| d.points.iter().find(|p| p.id == id).unwrap().xy;
                    (p(c.center), distance(p(c.center), p(c.rim)))
                };
                let (sa, sr) = seed(curves[0]);
                let (sb, ss) = seed(curves[1]);
                let internal =
                    (distance(sa, sb) - (sr - ss).abs()).abs() < (distance(sa, sb) - sr - ss).abs();
                vec![distance(a, b) - if internal { (r - s).abs() } else { r + s }]
            }
        }
        C::Symmetry { points, axis } => {
            let [a, b] = line(*axis);
            let p = point(points[0]);
            let q = point(points[1]);
            let v = sub(b, a);
            vec![
                cross(v, sub([(p[0] + q[0]) * 0.5, (p[1] + q[1]) * 0.5], a)) / norm(v).max(1e-8),
                dot(v, sub(q, p)) / norm(v).max(1e-8),
            ]
        }
        C::LineDistance { lines, parameter } => {
            let [a, b] = line(lines[0]);
            let [c, _] = line(lines[1]);
            vec![
                cross(sub(b, a), sub(c, a)).abs() / distance(a, b).max(1e-8)
                    - parameters[parameter] * 1000.,
                cross(sub(b, a), direction(lines[1])) / distance(a, b).max(1e-8),
            ]
        }
        C::Angle { lines, parameter } => {
            let a = direction(lines[0]);
            let b = direction(lines[1]);
            let angle = cross(a, b).atan2(dot(a, b));
            let target = parameters[parameter];
            vec![
                ((angle - target + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
                    - std::f64::consts::PI)
                    * 10.,
            ]
        }
        C::Diameter {
            circle: id,
            parameter,
        } => vec![circle(*id).1 * 2. - parameters[parameter] * 1000.],
        C::Radius {
            circle: id,
            parameter,
        } => vec![circle(*id).1 - parameters[parameter] * 1000.],
    }
}
pub fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
pub fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
pub fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
pub fn norm(a: [f64; 2]) -> f64 {
    a[0].hypot(a[1])
}
pub fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    norm(sub(a, b))
}
