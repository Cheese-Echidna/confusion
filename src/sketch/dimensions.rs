//! Dimension selection and read-only measurements share exact sketch queries.
use crate::document::schema::{ConstraintKind as C, Design};
use crate::sketch::entities::{point, radius};
use uuid::Uuid;
pub fn kind(d: &Design, selected: &[Uuid], parameter: Uuid) -> Result<C, String> {
    if selected.len() == 1 {
        let id = selected[0];
        if d.lines.iter().any(|l| l.id == id) {
            return Ok(C::Length {
                line: id,
                parameter,
            });
        }
        if let Some(c) = d.circles.iter().find(|c| c.id == id) {
            return Ok(if c.end.is_some() {
                C::Radius {
                    circle: id,
                    parameter,
                }
            } else {
                C::Diameter {
                    circle: id,
                    parameter,
                }
            });
        }
    }
    if selected.len() == 2 {
        if selected
            .iter()
            .all(|id| d.points.iter().any(|p| p.id == *id))
        {
            return Ok(C::Distance {
                points: [selected[0], selected[1]],
                parameter,
            });
        }
        if selected
            .iter()
            .all(|id| d.lines.iter().any(|l| l.id == *id))
        {
            let directions: Vec<_> = selected
                .iter()
                .map(|id| {
                    let l = d.lines.iter().find(|l| l.id == *id).unwrap();
                    let a = point(d, l.ends[0]);
                    let b = point(d, l.ends[1]);
                    [b[0] - a[0], b[1] - a[1]]
                })
                .collect();
            let cross = directions[0][0] * directions[1][1] - directions[0][1] * directions[1][0];
            return Ok(if cross.abs() < 1e-8 {
                C::LineDistance {
                    lines: [selected[0], selected[1]],
                    parameter,
                }
            } else {
                C::Angle {
                    lines: [selected[0], selected[1]],
                    parameter,
                }
            });
        }
    }
    if selected.len() == 2
        && selected
            .iter()
            .all(|id| d.circles.iter().any(|c| c.id == *id))
    {
        let points = selected
            .iter()
            .map(|id| d.circles.iter().find(|c| c.id == *id).unwrap().center)
            .collect::<Vec<_>>();
        return Ok(C::Distance {
            points: [points[0], points[1]],
            parameter,
        });
    }
    Err("Select a line, circle, arc, two points, or two lines".into())
}
pub fn value(d: &Design, c: &C) -> Option<f64> {
    let distance = |a: Uuid, b: Uuid| {
        let a = point(d, a);
        let b = point(d, b);
        (a[0] - b[0]).hypot(a[1] - b[1])
    };
    match c {
        C::LineOffset { lines, side, .. } => {
            let [a, b] = d
                .lines
                .iter()
                .find(|l| l.id == lines[0])
                .unwrap()
                .ends
                .map(|id| point(d, id));
            let c = point(
                d,
                d.lines.iter().find(|l| l.id == lines[1]).unwrap().ends[0],
            );
            let v = [b[0] - a[0], b[1] - a[1]];
            Some((v[0] * (c[1] - a[1]) - v[1] * (c[0] - a[0])) / v[0].hypot(v[1]) / side)
        }
        C::OffsetRadius { circles, .. } => Some(
            radius(d, d.circles.iter().find(|c| c.id == circles[1]).unwrap())
                - radius(d, d.circles.iter().find(|c| c.id == circles[0]).unwrap()),
        ),
        C::ProjectedDistance {
            points, direction, ..
        } => {
            let a = point(d, points[0]);
            let b = point(d, points[1]);
            Some((b[0] - a[0]) * direction[0] + (b[1] - a[1]) * direction[1])
        }
        C::Length { line, .. } => d
            .lines
            .iter()
            .find(|l| l.id == *line)
            .map(|l| distance(l.ends[0], l.ends[1])),
        C::Distance { points, .. } => Some(distance(points[0], points[1])),
        C::DistanceX { points, .. } => Some(point(d, points[1])[0] - point(d, points[0])[0]),
        C::DistanceY { points, .. } => Some(point(d, points[1])[1] - point(d, points[0])[1]),
        C::Radius { circle, .. } | C::Diameter { circle, .. } => {
            d.circles.iter().find(|c| c.id == *circle).map(|circle| {
                radius(d, circle)
                    * if matches!(c, C::Diameter { .. }) {
                        2.
                    } else {
                        1.
                    }
            })
        }
        C::LineDistance { lines, .. } => {
            let a = d
                .lines
                .iter()
                .find(|l| l.id == lines[0])
                .unwrap()
                .ends
                .map(|id| point(d, id));
            let b = d
                .lines
                .iter()
                .find(|l| l.id == lines[1])
                .unwrap()
                .ends
                .map(|id| point(d, id));
            let v = [a[1][0] - a[0][0], a[1][1] - a[0][1]];
            Some(
                (v[0] * (b[0][1] - a[0][1]) - v[1] * (b[0][0] - a[0][0])).abs()
                    / v[0].hypot(v[1]).max(1e-12),
            )
        }
        C::Angle { lines, .. } => {
            let dir = |id| {
                let l = d.lines.iter().find(|l| l.id == id).unwrap();
                let a = point(d, l.ends[0]);
                let b = point(d, l.ends[1]);
                [b[0] - a[0], b[1] - a[1]]
            };
            let a = dir(lines[0]);
            let b = dir(lines[1]);
            Some((a[0] * b[1] - a[1] * b[0]).atan2(a[0] * b[0] + a[1] * b[1]))
        }
        _ => None,
    }
}
pub fn measure(d: &Design, selected: &[Uuid]) -> String {
    if selected.len() == 1 {
        let id = selected[0];
        if let Some(c) = d.circles.iter().find(|c| c.id == id) {
            let r = radius(d, c);
            let a = point(d, c.center);
            let rim = point(d, c.rim);
            let start = (rim[1] - a[1]).atan2(rim[0] - a[0]);
            let sweep = c.end.map_or(std::f64::consts::TAU, |id| {
                let end = point(d, id);
                ((end[1] - a[1]).atan2(end[0] - a[0]) - start).rem_euclid(std::f64::consts::TAU)
            });
            return format!(
                "Radius: {:.3} mm\nDiameter: {:.3} mm\nLength: {:.3} mm\nAngle: {:.3}°\nCenter X: {:.3} mm\nCenter Y: {:.3} mm",
                r * 1000.,
                r * 2000.,
                r * sweep * 1000.,
                sweep.to_degrees(),
                a[0] * 1000.,
                a[1] * 1000.
            );
        }
        if d.splines.iter().any(|c| c.id == id) || d.ellipses.iter().any(|c| c.id == id) {
            let samples = crate::sketch::entities::samples(d, id);
            let length = samples
                .windows(2)
                .map(|p| (p[1][0] - p[0][0]).hypot(p[1][1] - p[0][1]))
                .sum::<f64>();
            return format!("Curve length (approx.): {:.3} mm", length * 1000.);
        }
    }
    if let Ok(c) = kind(d, selected, Uuid::nil())
        && let Some(v) = value(d, &c)
    {
        return if matches!(c, C::Angle { .. }) {
            format!("Angle: {:.3}°", v.to_degrees().abs())
        } else {
            format!("{}: {:.3} mm", c.label(), v.abs() * 1000.)
        };
    }
    if selected.len() == 1
        && let Some(p) = d.points.iter().find(|p| p.id == selected[0])
    {
        return format!(
            "X: {:.3} mm · Y: {:.3} mm",
            p.xy[0] * 1000.,
            p.xy[1] * 1000.
        );
    }
    if selected.len() == 2 {
        let p = selected
            .iter()
            .find_map(|id| d.points.iter().find(|p| p.id == *id));
        if let Some(p) = p {
            let curve = *selected.iter().find(|id| **id != p.id).unwrap();
            if let Some(c) = d.circles.iter().find(|c| c.id == curve) {
                let a = point(d, c.center);
                let distance = ((p.xy[0] - a[0]).hypot(p.xy[1] - a[1]) - radius(d, c)).abs();
                return format!("Minimum distance: {:.3} mm", distance * 1000.);
            }
            let samples = crate::sketch::entities::samples(d, curve);
            if !samples.is_empty() {
                let distance = samples
                    .windows(2)
                    .map(|ab| crate::sketch::entities::segment_distance(p.xy, ab[0], ab[1]))
                    .fold(f64::INFINITY, f64::min);
                return format!("Minimum distance: {:.3} mm", distance * 1000.);
            }
        }
    }
    "Select geometry to measure".into()
}

/// Dimension placement chooses horizontal, vertical or aligned distance for a point pair.
pub fn kind_at(
    d: &Design,
    selected: &[Uuid],
    parameter: Uuid,
    at: Option<[f64; 2]>,
) -> Result<C, String> {
    let kind = kind(d, selected, parameter)?;
    if let (C::Distance { points, parameter }, Some(at)) = (&kind, at) {
        let a = point(d, points[0]);
        let b = point(d, points[1]);
        if at[0] < a[0].min(b[0]) || at[0] > a[0].max(b[0]) {
            return Ok(C::DistanceY {
                points: *points,
                parameter: *parameter,
            });
        }
        if at[1] < a[1].min(b[1]) || at[1] > a[1].max(b[1]) {
            return Ok(C::DistanceX {
                points: *points,
                parameter: *parameter,
            });
        }
    }
    Ok(kind)
}
