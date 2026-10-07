//! Shared exact sketch queries for picking, annotation, editing and display.
use crate::document::schema::{Circle, Design};
use uuid::Uuid;
pub fn point(d: &Design, id: Uuid) -> [f64; 2] {
    d.points.iter().find(|p| p.id == id).unwrap().xy
}
pub fn curve_points(d: &Design, id: Uuid) -> Vec<Uuid> {
    if let Some(l) = d.lines.iter().find(|l| l.id == id) {
        return l.ends.to_vec();
    }
    if let Some(c) = d.circles.iter().find(|c| c.id == id) {
        let mut p = vec![c.center, c.rim];
        if let Some(e) = c.end {
            p.push(e)
        }
        return p;
    }
    if let Some(e) = d.ellipses.iter().find(|e| e.id == id) {
        return vec![e.center, e.major, e.minor];
    }
    if let Some(s) = d.splines.iter().find(|s| s.id == id) {
        return s.points.clone();
    }
    vec![]
}
pub fn radius(d: &Design, c: &Circle) -> f64 {
    let a = point(d, c.center);
    let b = point(d, c.rim);
    (a[0] - b[0]).hypot(a[1] - b[1])
}
pub fn samples(d: &Design, id: Uuid) -> Vec<[f64; 2]> {
    if let Some(l) = d.lines.iter().find(|l| l.id == id) {
        return l.ends.map(|id| point(d, id)).to_vec();
    }
    if let Some(e) = d.ellipses.iter().find(|e| e.id == id) {
        let a = point(d, e.center);
        let b = point(d, e.major);
        let c = point(d, e.minor);
        let major = [b[0] - a[0], b[1] - a[1]];
        let minor = [c[0] - a[0], c[1] - a[1]];
        return (0..=128)
            .map(|i| {
                let t = std::f64::consts::TAU * i as f64 / 128.;
                [
                    a[0] + major[0] * t.cos() + minor[0] * t.sin(),
                    a[1] + major[1] * t.cos() + minor[1] * t.sin(),
                ]
            })
            .collect();
    }
    if let Some(s) = d.splines.iter().find(|s| s.id == id) {
        let controls: Vec<_> = s.points.iter().map(|id| point(d, *id)).collect();
        let mut result = vec![];
        if s.fit {
            for i in 0..controls.len() - 1 {
                let p0 = controls[i.saturating_sub(1)];
                let p1 = controls[i];
                let p2 = controls[i + 1];
                let p3 = controls[(i + 2).min(controls.len() - 1)];
                for k in 0..32 {
                    let t = k as f64 / 32.;
                    result.push(std::array::from_fn(|j| {
                        0.5 * ((2. * p1[j])
                            + (-p0[j] + p2[j]) * t
                            + (2. * p0[j] - 5. * p1[j] + 4. * p2[j] - p3[j]) * t * t
                            + (-p0[j] + 3. * p1[j] - 3. * p2[j] + p3[j]) * t * t * t)
                    }));
                }
            }
            result.push(*controls.last().unwrap());
        } else {
            // Uniform clamped B-spline, degree up to three, evaluated by de Boor.
            let degree = (controls.len() - 1).min(3);
            let count = controls.len();
            let knots: Vec<_> = (0..count + degree + 1)
                .map(|i| {
                    if i <= degree {
                        0.
                    } else if i >= count {
                        1.
                    } else {
                        (i - degree) as f64 / (count - degree) as f64
                    }
                })
                .collect();
            for i in 0..=128 {
                let t = i as f64 / 128.;
                let span = if i == 128 {
                    count - 1
                } else {
                    (degree..count)
                        .find(|k| t >= knots[*k] && t < knots[*k + 1])
                        .unwrap()
                };
                let mut values: Vec<_> = (span - degree..=span).map(|k| controls[k]).collect();
                for r in 1..=degree {
                    for j in (r..=degree).rev() {
                        let k = span - degree + j;
                        let alpha = (t - knots[k]) / (knots[k + degree + 1 - r] - knots[k]);
                        values[j] = std::array::from_fn(|axis| {
                            (1. - alpha) * values[j - 1][axis] + alpha * values[j][axis]
                        });
                    }
                }
                result.push(values[degree]);
            }
        }
        return result;
    }
    let Some(c) = d.circles.iter().find(|c| c.id == id) else {
        return vec![];
    };
    let a = point(d, c.center);
    let b = point(d, c.rim);
    let r = radius(d, c);
    let start = (b[1] - a[1]).atan2(b[0] - a[0]);
    let sweep = c.end.map_or(std::f64::consts::TAU, |id| {
        let e = point(d, id);
        ((e[1] - a[1]).atan2(e[0] - a[0]) - start).rem_euclid(std::f64::consts::TAU)
    });
    (0..=96)
        .map(|i| {
            let t = start + sweep * i as f64 / 96.;
            [a[0] + r * t.cos(), a[1] + r * t.sin()]
        })
        .collect()
}
pub fn closest(d: &Design, p: [f64; 2], radius: f64) -> Option<Uuid> {
    d.lines
        .iter()
        .map(|l| l.id)
        .chain(d.circles.iter().map(|c| c.id))
        .chain(d.ellipses.iter().map(|c| c.id))
        .chain(d.splines.iter().map(|c| c.id))
        .filter_map(|id| {
            let sample = samples(d, id);
            let distance = sample
                .windows(2)
                .map(|ab| segment_distance(p, ab[0], ab[1]))
                .fold(f64::INFINITY, f64::min);
            (distance < radius).then_some((id, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|v| v.0)
}
pub fn segment_distance(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let v = [b[0] - a[0], b[1] - a[1]];
    let t = (((p[0] - a[0]) * v[0] + (p[1] - a[1]) * v[1])
        / (v[0] * v[0] + v[1] * v[1]).max(1e-24))
    .clamp(0., 1.);
    (p[0] - a[0] - t * v[0]).hypot(p[1] - a[1] - t * v[1])
}
pub fn position(d: &Design, id: Uuid) -> [f64; 2] {
    if let Some(l) = d.lines.iter().find(|l| l.id == id) {
        let [a, b] = l.ends.map(|id| point(d, id));
        return [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    }
    if d.points.iter().any(|p| p.id == id) {
        return point(d, id);
    }
    let ps = samples(d, id);
    if ps.is_empty() {
        return [0., 0.];
    };
    ps[ps.len() / 2]
}

pub fn curve_ids(d: &Design) -> Vec<Uuid> {
    d.lines
        .iter()
        .map(|c| c.id)
        .chain(d.circles.iter().map(|c| c.id))
        .chain(d.ellipses.iter().map(|c| c.id))
        .chain(d.splines.iter().map(|c| c.id))
        .collect()
}

pub fn segment_intersects_box(a: [f64; 2], b: [f64; 2], min: [f64; 2], max: [f64; 2]) -> bool {
    let mut lo: f64 = 0.;
    let mut hi: f64 = 1.;
    for k in 0..2 {
        let delta = b[k] - a[k];
        if delta.abs() < 1e-14 {
            if a[k] < min[k] || a[k] > max[k] {
                return false;
            }
        } else {
            let first = (min[k] - a[k]) / delta;
            let last = (max[k] - a[k]) / delta;
            lo = lo.max(first.min(last));
            hi = hi.min(first.max(last));
        }
    }
    lo <= hi
}
