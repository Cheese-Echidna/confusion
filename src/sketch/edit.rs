//! Reference-safe sketch editing. All operations work on a candidate; callers commit once.
use crate::document::schema::{Circle, ConstraintKind as C, Design};
use crate::sketch::entities::{curve_points, point};
use uuid::Uuid;
pub fn circle(
    d: &mut Design,
    center: [f64; 2],
    rim: [f64; 2],
    end: Option<[f64; 2]>,
) -> Result<Uuid, String> {
    if (center[0] - rim[0]).hypot(center[1] - rim[1]) < 1e-7 {
        return Err("Circle radius is too small".into());
    }
    d.ensure_sketch();
    let id = Uuid::new_v4();
    let center = d.point(center);
    let rim = d.point(rim);
    let end = end.map(|p| d.point(p));
    d.circles.push(Circle {
        id,
        center,
        rim,
        end,
    });
    d.validate()?;
    Ok(id)
}
pub fn delete(d: &mut Design, selection: &[Uuid]) {
    let mut removed = selection.to_vec();
    for l in &d.lines {
        if l.ends.iter().any(|p| selection.contains(p)) {
            removed.push(l.id)
        }
    }
    for c in &d.circles {
        if [Some(c.center), Some(c.rim), c.end]
            .iter()
            .flatten()
            .any(|p| selection.contains(p))
        {
            removed.push(c.id)
        }
    }
    for e in &d.ellipses {
        if [e.center, e.major, e.minor]
            .iter()
            .any(|id| selection.contains(id))
        {
            removed.push(e.id)
        }
    }
    for s in &d.splines {
        if s.points.iter().any(|id| selection.contains(id)) {
            removed.push(s.id)
        }
    }
    d.ellipses.retain(|e| !removed.contains(&e.id));
    d.splines.retain(|s| !removed.contains(&s.id));
    d.lines.retain(|l| !removed.contains(&l.id));
    d.circles.retain(|c| !removed.contains(&c.id));
    d.points.retain(|p| !selection.contains(&p.id));
    d.constraints
        .retain(|c| !c.kind.references().iter().any(|id| removed.contains(id)));
    d.driven_dimensions
        .retain(|c| !c.kind.references().iter().any(|id| removed.contains(id)));
    d.construction_geometry.retain(|id| !removed.contains(id));
    let used: Vec<_> = d
        .lines
        .iter()
        .flat_map(|l| l.ends)
        .chain(
            d.circles
                .iter()
                .flat_map(|c| [Some(c.center), Some(c.rim), c.end].into_iter().flatten()),
        )
        .chain(d.ellipses.iter().flat_map(|e| [e.center, e.major, e.minor]))
        .chain(d.splines.iter().flat_map(|s| s.points.iter().copied()))
        .chain(d.constraints.iter().flat_map(|c| c.kind.references()))
        .collect();
    d.points.retain(|p| used.contains(&p.id));
    let constraints: Vec<_> = d
        .constraints
        .iter()
        .chain(&d.driven_dimensions)
        .map(|c| c.id)
        .collect();
    d.dimension_positions
        .retain(|id, _| constraints.contains(id));
}
pub fn toggle_construction(d: &mut Design, ids: &[Uuid]) {
    for id in ids {
        if let Some(i) = d.construction_geometry.iter().position(|p| p == id) {
            d.construction_geometry.remove(i);
        } else if !curve_points(d, *id).is_empty() {
            d.construction_geometry.push(*id)
        }
    }
}
pub fn fixed(d: &mut Design, ids: &[Uuid]) {
    let mut points = Vec::new();
    for id in ids {
        let ps = curve_points(d, *id);
        if ps.is_empty() {
            points.push(*id)
        } else {
            points.extend(ps)
        }
    }
    points.sort();
    points.dedup();
    let all = points.iter().all(|p| {
        d.constraints
            .iter()
            .any(|c| matches!(c.kind,C::Fixed{point,..} if point==*p))
    });
    if all {
        d.constraints
            .retain(|c| !matches!(c.kind,C::Fixed{point,..} if points.contains(&point)))
    } else {
        for p in points {
            if !d
                .constraints
                .iter()
                .any(|c| matches!(c.kind,C::Fixed{point,..} if point==p))
            {
                d.constrain(C::Fixed {
                    point: p,
                    xy: point(d, p),
                })
            }
        }
    }
}
/// Split a straight edge at a projected point, preserving H/V relationships on both pieces.
pub fn split(d: &mut Design, id: Uuid, p: [f64; 2]) -> Result<(), String> {
    if d.circles.iter().any(|c| c.id == id) {
        return break_circle(d, id, p);
    }
    let l = d
        .lines
        .iter()
        .find(|l| l.id == id)
        .ok_or("Select a line to break")?
        .clone();
    let [a, b] = l.ends.map(|id| point(d, id));
    let v = [b[0] - a[0], b[1] - a[1]];
    let t = ((p[0] - a[0]) * v[0] + (p[1] - a[1]) * v[1]) / (v[0] * v[0] + v[1] * v[1]);
    if !(1e-5..1. - 1e-5).contains(&t) {
        return Err("Break point must be inside the line".into());
    }
    let at = d.point([a[0] + t * v[0], a[1] + t * v[1]]);
    let next = Uuid::new_v4();
    let mut constraints = Vec::new();
    for c in &d.constraints {
        match c.kind {
            C::Horizontal { line } if line == id => constraints.push(C::Horizontal { line: next }),
            C::Vertical { line } if line == id => constraints.push(C::Vertical { line: next }),
            _ => {}
        }
    }
    // A line-length dimension must remain attached to the original endpoints, not a shortened piece.
    for c in &mut d.constraints {
        if let C::Length { line, parameter } = c.kind
            && line == id
        {
            c.kind = C::Distance {
                points: l.ends,
                parameter,
            }
        }
    }
    d.lines.iter_mut().find(|l| l.id == id).unwrap().ends[1] = at;
    d.lines.push(crate::document::schema::Line {
        id: next,
        ends: [at, l.ends[1]],
    });
    for c in constraints {
        d.constrain(c)
    }
    if d.construction_geometry.contains(&id) {
        d.construction_geometry.push(next)
    }
    Ok(())
}
/// Intersections with other straight edges, sorted along the selected edge.
fn intersections(d: &Design, id: Uuid) -> Result<Vec<f64>, String> {
    let l = d
        .lines
        .iter()
        .find(|l| l.id == id)
        .ok_or("Select a straight line")?;
    let [a, b] = l.ends.map(|id| point(d, id));
    let v = [b[0] - a[0], b[1] - a[1]];
    let cross = |a: [f64; 2], b: [f64; 2]| a[0] * b[1] - a[1] * b[0];
    let mut ts = vec![];
    for other in d.lines.iter().filter(|l| l.id != id) {
        let [c, e] = other.ends.map(|id| point(d, id));
        let w = [e[0] - c[0], e[1] - c[1]];
        let denom = cross(v, w);
        if denom.abs() < 1e-14 {
            continue;
        }
        let q = [c[0] - a[0], c[1] - a[1]];
        let t = cross(q, w) / denom;
        let u = cross(q, v) / denom;
        if (-1e-8..=1. + 1e-8).contains(&u) {
            ts.push(t)
        }
    }
    for circle in &d.circles {
        let center = point(d, circle.center);
        let radius = crate::sketch::entities::radius(d, circle);
        for t in line_circle(a, b, center, radius) {
            let at = [a[0] + v[0] * t, a[1] + v[1] * t];
            if on_arc(d, circle, at) {
                ts.push(t)
            }
        }
    }
    ts.sort_by(f64::total_cmp);
    ts.dedup_by(|a, b| (*a - *b).abs() < 1e-8);
    Ok(ts)
}
pub fn trim(d: &mut Design, id: Uuid, p: [f64; 2]) -> Result<(), String> {
    if d.circles.iter().any(|c| c.id == id) {
        return trim_circle(d, id, p);
    }
    let l = d
        .lines
        .iter()
        .find(|l| l.id == id)
        .ok_or("Trim currently requires a straight line")?
        .clone();
    let [a, b] = l.ends.map(|id| point(d, id));
    let v = [b[0] - a[0], b[1] - a[1]];
    let t = ((p[0] - a[0]) * v[0] + (p[1] - a[1]) * v[1]) / (v[0] * v[0] + v[1] * v[1]);
    let ts = intersections(d, id)?;
    let lo = ts
        .iter()
        .copied()
        .rfind(|v| *v > 1e-8 && *v < t)
        .unwrap_or(0.);
    let hi = ts
        .iter()
        .copied()
        .find(|v| *v > t && *v < 1. - 1e-8)
        .unwrap_or(1.);
    let at = |t: f64| [a[0] + t * v[0], a[1] + t * v[1]];
    let construction = d.construction_geometry.contains(&id);
    // Changed edges invalidate their constraints explicitly; unaffected edges retain their IDs.
    delete(d, &[id]);
    for (start, end) in [(0., lo), (hi, 1.)] {
        if end - start > 1e-8 {
            let next = d.line(at(start), at(end));
            if construction {
                d.construction_geometry.push(next)
            }
        }
    }
    Ok(())
}
pub fn extend(d: &mut Design, id: Uuid, p: [f64; 2]) -> Result<(), String> {
    if d.circles.iter().any(|c| c.id == id) {
        return extend_arc(d, id, p);
    }
    let l = d
        .lines
        .iter()
        .find(|l| l.id == id)
        .ok_or("Extend currently requires a straight line")?
        .clone();
    let [a, b] = l.ends.map(|id| point(d, id));
    let v = [b[0] - a[0], b[1] - a[1]];
    let start = (p[0] - a[0]).hypot(p[1] - a[1]) < (p[0] - b[0]).hypot(p[1] - b[1]);
    let ts = intersections(d, id)?;
    let t = if start {
        ts.into_iter().rfind(|t| *t < -1e-8)
    } else {
        ts.into_iter().find(|t| *t > 1. + 1e-8)
    }
    .ok_or("No intersection to extend to")?;
    let new = d.point([a[0] + t * v[0], a[1] + t * v[1]]);
    d.lines.iter_mut().find(|l| l.id == id).unwrap().ends[usize::from(!start)] = new;
    d.constraints.retain(|c| {
        !c.kind.references().contains(&id)
            || matches!(c.kind, C::Horizontal { .. } | C::Vertical { .. })
    });
    Ok(())
}
/// Offset independent lines and circles. New geometry retains editable H/V relationships.
pub fn offset(d: &mut Design, ids: &[Uuid], distance: f64) -> Result<(), String> {
    if ids.is_empty() {
        return Err("Select curves to offset".into());
    }
    if !distance.is_finite() || distance.abs() < 1e-7 {
        return Err("Offset must be nonzero".into());
    }
    let parameter = d.parameter(
        &format!("offset{}", d.parameters.len()),
        format!("{} mm", distance * 1000.),
    );
    let line_ids: Vec<_> = ids
        .iter()
        .copied()
        .filter(|id| d.lines.iter().any(|l| l.id == *id))
        .collect();
    if !line_ids.is_empty() {
        offset_lines(d, &line_ids, distance, parameter)?;
    }
    for id in ids.iter().filter(|id| !line_ids.contains(id)) {
        let c = d
            .circles
            .iter()
            .find(|c| c.id == *id)
            .cloned()
            .ok_or("Select lines, circles or arcs to offset")?;
        let a = point(d, c.center);
        let b = point(d, c.rim);
        let old = (a[0] - b[0]).hypot(a[1] - b[1]);
        let radius = old + distance;
        if radius < 1e-7 {
            return Err("Offset collapses the circle".into());
        }
        let factor = radius / old;
        let rim = [a[0] + (b[0] - a[0]) * factor, a[1] + (b[1] - a[1]) * factor];
        let end = c.end.map(|id| {
            let p = point(d, id);
            [a[0] + (p[0] - a[0]) * factor, a[1] + (p[1] - a[1]) * factor]
        });
        let next = circle(d, a, rim, end)?;
        d.constrain(C::OffsetRadius {
            circles: [*id, next],
            parameter,
        });
    }
    Ok(())
}
fn offset_lines(
    d: &mut Design,
    ids: &[Uuid],
    distance: f64,
    parameter: Uuid,
) -> Result<(), String> {
    use std::collections::{HashMap, HashSet};
    let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for id in ids {
        let line = d.lines.iter().find(|l| l.id == *id).unwrap();
        for p in line.ends {
            adjacency.entry(p).or_default().push(*id);
        }
    }
    if adjacency.values().any(|edges| edges.len() > 2) {
        return Err("Offset selection must be a chain without branches".into());
    }
    let closed = adjacency.values().all(|edges| edges.len() == 2);
    let mut current = adjacency
        .iter()
        .find(|(_, edges)| edges.len() == 1)
        .map(|(p, _)| *p)
        .unwrap_or_else(|| d.lines.iter().find(|l| l.id == ids[0]).unwrap().ends[0]);
    let mut vertices = vec![current];
    let mut ordered = Vec::new();
    let mut visited = HashSet::new();
    while let Some(id) = adjacency[&current]
        .iter()
        .find(|id| !visited.contains(*id))
        .copied()
    {
        visited.insert(id);
        ordered.push(id);
        let line = d.lines.iter().find(|l| l.id == id).unwrap();
        current = if line.ends[0] == current {
            line.ends[1]
        } else {
            line.ends[0]
        };
        vertices.push(current);
    }
    if visited.len() != ids.len() {
        return Err("Offset one connected chain at a time".into());
    }
    let coords: Vec<_> = vertices.iter().map(|p| point(d, *p)).collect();
    let mut shifted = Vec::new();
    for pair in coords.windows(2) {
        let a = pair[0];
        let b = pair[1];
        let length = (b[0] - a[0]).hypot(b[1] - a[1]);
        if length < 1e-7 {
            return Err("Cannot offset a degenerate edge".into());
        }
        let n = [
            -(b[1] - a[1]) / length * distance,
            (b[0] - a[0]) / length * distance,
        ];
        shifted.push([[a[0] + n[0], a[1] + n[1]], [b[0] + n[0], b[1] + n[1]]]);
    }
    let intersection = |first: [[f64; 2]; 2], second: [[f64; 2]; 2]| {
        let a = first[0];
        let b = second[0];
        let v = [first[1][0] - a[0], first[1][1] - a[1]];
        let w = [second[1][0] - b[0], second[1][1] - b[1]];
        let cross = |v: [f64; 2], w: [f64; 2]| v[0] * w[1] - v[1] * w[0];
        let denominator = cross(v, w);
        if denominator.abs() < 1e-14 {
            return first[1];
        }
        let t = cross([b[0] - a[0], b[1] - a[1]], w) / denominator;
        [a[0] + v[0] * t, a[1] + v[1] * t]
    };
    let mut offsets = Vec::new();
    for i in 0..if closed {
        ordered.len()
    } else {
        ordered.len() + 1
    } {
        let at = if i == 0 && !closed {
            shifted[0][0]
        } else if i == ordered.len() {
            shifted.last().unwrap()[1]
        } else {
            intersection(shifted[(i + ordered.len() - 1) % ordered.len()], shifted[i])
        };
        offsets.push(at);
    }
    let mut new_ids = Vec::new();
    for i in 0..ordered.len() {
        let a = offsets[i];
        let b = offsets[(i + 1) % offsets.len()];
        let source = d.lines.iter().find(|l| l.id == ordered[i]).unwrap().clone();
        let [sa, sb] = source.ends.map(|id| point(d, id));
        let sign = if source.ends[0] == vertices[i] {
            1.
        } else {
            -1.
        };
        if ((b[0] - a[0]) * (sb[0] - sa[0]) + (b[1] - a[1]) * (sb[1] - sa[1])) * sign <= 0. {
            return Err("Offset collapses an edge".into());
        }
        let next = d.line(a, b);
        new_ids.push(next);
        d.constrain(C::LineOffset {
            lines: [ordered[i], next],
            parameter,
            side: sign,
        });
    }
    if closed {
        let mut profile = d.clone();
        profile.lines.retain(|l| new_ids.contains(&l.id));
        profile.circles.clear();
        profile.ellipses.clear();
        profile.splines.clear();
        profile.construction_geometry.clear();
        let xy = profile.points.iter().map(|p| p.xy).collect::<Vec<_>>();
        crate::sketch::profiles::closed_profile(&profile, &xy)?;
    } else {
        for (original, shifted, line) in [
            (vertices[0], offsets[0], ordered[0]),
            (
                *vertices.last().unwrap(),
                *offsets.last().unwrap(),
                *ordered.last().unwrap(),
            ),
        ] {
            let connector = d.line(point(d, original), shifted);
            d.construction_geometry.push(connector);
            d.constrain(C::Perpendicular {
                lines: [line, connector],
            });
        }
    }
    Ok(())
}

pub fn ellipse(
    d: &mut Design,
    center: [f64; 2],
    major: [f64; 2],
    minor: [f64; 2],
) -> Result<Uuid, String> {
    let v = [major[0] - center[0], major[1] - center[1]];
    let length = v[0].hypot(v[1]);
    if length < 1e-7 {
        return Err("Ellipse axis is too small".into());
    }
    let n = [-v[1] / length, v[0] / length];
    let width = (minor[0] - center[0]) * n[0] + (minor[1] - center[1]) * n[1];
    if width.abs() < 1e-7 {
        return Err("Ellipse minor axis is too small".into());
    }
    let minor = [center[0] + n[0] * width, center[1] + n[1] * width];
    d.ensure_sketch();
    let id = Uuid::new_v4();
    let center = d.point(center);
    let major = d.point(major);
    let minor = d.point(minor);
    d.ellipses.push(crate::document::schema::Ellipse {
        id,
        center,
        major,
        minor,
    });
    Ok(id)
}
pub fn spline(d: &mut Design, points: &[[f64; 2]], fit: bool) -> Result<Uuid, String> {
    if points.len() < 2 || points.len() > 32 {
        return Err("Spline needs between 2 and 32 points".into());
    }
    d.ensure_sketch();
    let id = Uuid::new_v4();
    let points = points.iter().map(|p| d.point(*p)).collect();
    d.splines
        .push(crate::document::schema::Spline { id, points, fit });
    Ok(id)
}
pub fn circumcenter(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Result<[f64; 2], String> {
    let b = [b[0] - a[0], b[1] - a[1]];
    let c = [c[0] - a[0], c[1] - a[1]];
    let denom = 2. * (b[0] * c[1] - b[1] * c[0]);
    if denom.abs() < 1e-12 {
        return Err("Three points must not be collinear".into());
    }
    let b2 = b[0] * b[0] + b[1] * b[1];
    let c2 = c[0] * c[0] + c[1] * c[1];
    Ok([
        a[0] + (b2 * c[1] - c2 * b[1]) / denom,
        a[1] + (b[0] * c2 - c[0] * b2) / denom,
    ])
}
pub fn polygon(
    d: &mut Design,
    center: [f64; 2],
    corner: [f64; 2],
    sides: usize,
) -> Result<(), String> {
    if !(3..=32).contains(&sides) {
        return Err("Polygon needs 3 to 32 sides".into());
    }
    let radius = (corner[0] - center[0]).hypot(corner[1] - center[1]);
    let start = (corner[1] - center[1]).atan2(corner[0] - center[0]);
    let points: Vec<_> = (0..sides)
        .map(|i| {
            let t = start + std::f64::consts::TAU * i as f64 / sides as f64;
            [center[0] + radius * t.cos(), center[1] + radius * t.sin()]
        })
        .collect();
    let ids: Vec<_> = (0..sides)
        .map(|i| d.line(points[i], points[(i + 1) % sides]))
        .collect();
    for id in &ids[1..] {
        d.constrain(C::Equal {
            curves: [ids[0], *id],
        })
    }
    let parameter = d.parameter(
        &format!("polygonAngle{}", d.parameters.len()),
        format!("{} deg", 360. / sides as f64),
    );
    d.parameters
        .iter_mut()
        .find(|p| p.id == parameter)
        .unwrap()
        .angular = true;
    for i in 0..sides {
        d.constrain(C::Angle {
            lines: [ids[i], ids[(i + 1) % sides]],
            parameter,
        });
    }
    Ok(())
}
pub fn slot(d: &mut Design, a: [f64; 2], b: [f64; 2], width_at: [f64; 2]) -> Result<(), String> {
    let v = [b[0] - a[0], b[1] - a[1]];
    let len = v[0].hypot(v[1]);
    if len < 1e-7 {
        return Err("Slot centerline is too short".into());
    }
    let n = [-v[1] / len, v[0] / len];
    let radius = ((width_at[0] - b[0]) * n[0] + (width_at[1] - b[1]) * n[1]).abs();
    if radius < 1e-7 {
        return Err("Slot width is too small".into());
    }
    let a1 = [a[0] + n[0] * radius, a[1] + n[1] * radius];
    let a2 = [a[0] - n[0] * radius, a[1] - n[1] * radius];
    let b1 = [b[0] + n[0] * radius, b[1] + n[1] * radius];
    let b2 = [b[0] - n[0] * radius, b[1] - n[1] * radius];
    d.line(a1, b1);
    d.line(a2, b2);
    let c1 = circle(d, a, a1, Some(a2))?;
    let c2 = circle(d, b, b2, Some(b1))?;
    let centerline = d.line(a, b);
    d.construction_geometry.push(centerline);
    for (center, first, second) in [(a, a1, a2), (b, b1, b2)] {
        let diameter = d.line(first, second);
        d.construction_geometry.push(diameter);
        let center = d.point(center);
        d.constrain(C::Midpoint {
            point: center,
            line: diameter,
        });
        d.constrain(C::Perpendicular {
            lines: [centerline, diameter],
        });
    }
    d.constrain(C::Equal { curves: [c1, c2] });
    Ok(())
}

/// Copy selected geometry with fresh identities and internal constraints, preserving parameter
/// references. Relations to unselected geometry stay on the original geometry only.
pub fn transform(
    d: &mut Design,
    selected: &[Uuid],
    map: impl Fn([f64; 2]) -> [f64; 2],
    copy: bool,
) -> Result<Vec<Uuid>, String> {
    if selected.is_empty() {
        return Err("Select geometry first".into());
    }
    let mut points = Vec::new();
    for id in selected {
        let ids = curve_points(d, *id);
        if ids.is_empty() && d.points.iter().any(|p| p.id == *id) {
            points.push(*id)
        } else {
            points.extend(ids)
        }
    }
    points.sort();
    points.dedup();
    let original = d.clone();
    let o = map([0., 0.]);
    let mx = map([1., 0.]);
    let my = map([0., 1.]);
    let reflected = (mx[0] - o[0]) * (my[1] - o[1]) - (mx[1] - o[1]) * (my[0] - o[0]) < 0.;
    if !copy && reflected {
        for c in &mut d.circles {
            if selected.contains(&c.id)
                && let Some(end) = c.end
            {
                let start = c.rim;
                c.rim = end;
                c.end = Some(start);
            }
        }
    }
    let mut ids = std::collections::HashMap::new();
    for id in &points {
        let next = if copy { Uuid::new_v4() } else { *id };
        ids.insert(*id, next);
        if copy {
            d.points.push(crate::document::schema::Point {
                id: next,
                xy: map(point(&original, *id)),
            })
        } else {
            d.points.iter_mut().find(|p| p.id == *id).unwrap().xy = map(point(&original, *id));
        }
    }
    for id in selected {
        if !ids.contains_key(id) {
            ids.insert(*id, if copy { Uuid::new_v4() } else { *id });
        }
    }
    let mut result = Vec::new();
    for id in selected {
        let next = ids[id];
        result.push(next);
        if !copy {
            continue;
        }
        if let Some(l) = original.lines.iter().find(|l| l.id == *id) {
            d.lines.push(crate::document::schema::Line {
                id: next,
                ends: l.ends.map(|id| ids[&id]),
            })
        }
        if let Some(c) = original.circles.iter().find(|c| c.id == *id) {
            d.circles.push(Circle {
                id: next,
                center: ids[&c.center],
                rim: if reflected {
                    c.end.map_or(ids[&c.rim], |id| ids[&id])
                } else {
                    ids[&c.rim]
                },
                end: if reflected {
                    c.end.map(|_| ids[&c.rim])
                } else {
                    c.end.map(|id| ids[&id])
                },
            })
        }
        if let Some(e) = original.ellipses.iter().find(|e| e.id == *id) {
            d.ellipses.push(crate::document::schema::Ellipse {
                id: next,
                center: ids[&e.center],
                major: ids[&e.major],
                minor: ids[&e.minor],
            })
        }
        if let Some(s) = original.splines.iter().find(|s| s.id == *id) {
            d.splines.push(crate::document::schema::Spline {
                id: next,
                points: s.points.iter().map(|id| ids[id]).collect(),
                fit: s.fit,
            })
        }
        if original.construction_geometry.contains(id) {
            d.construction_geometry.push(next)
        }
    }
    fn remap(v: &mut serde_json::Value, ids: &std::collections::HashMap<Uuid, Uuid>) {
        match v {
            serde_json::Value::String(s) => {
                if let Ok(id) = Uuid::parse_str(s)
                    && let Some(next) = ids.get(&id)
                {
                    *s = next.to_string()
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    remap(v, ids)
                }
            }
            serde_json::Value::Object(o) => {
                for v in o.values_mut() {
                    remap(v, ids)
                }
            }
            _ => {}
        }
    }
    for c in &original.constraints {
        if c.kind.references().iter().all(|id| ids.contains_key(id)) {
            let mut value = serde_json::to_value(&c.kind).map_err(|e| e.to_string())?;
            remap(&mut value, &ids);
            let mut kind: C = serde_json::from_value(value).map_err(|e| e.to_string())?;
            let origin = map([0., 0.]);
            let basis = |direction: [f64; 2]| {
                let p = map(direction);
                let v = [p[0] - origin[0], p[1] - origin[1]];
                let length = v[0].hypot(v[1]).max(1e-12);
                [v[0] / length, v[1] / length]
            };
            let scale = {
                let p = map([1., 0.]);
                (p[0] - origin[0]).hypot(p[1] - origin[1])
            };
            let bx = basis([1., 0.]);
            let by = basis([0., 1.]);
            let reflected = bx[0] * by[1] - bx[1] * by[0] < 0.;
            if let C::Fixed { xy, .. } = &mut kind {
                *xy = map(*xy)
            }
            if let C::HorizontalPoints { points } = kind {
                kind = C::ProjectedAlignment {
                    points,
                    direction: by,
                };
            }
            if let C::VerticalPoints { points } = kind {
                kind = C::ProjectedAlignment {
                    points,
                    direction: bx,
                };
            }
            if matches!(c.kind, C::ProjectedAlignment { .. })
                && let C::ProjectedAlignment { direction, .. } = &mut kind
            {
                *direction = basis(*direction);
            }
            if let C::Horizontal { line } | C::Vertical { line } = kind {
                let old = original
                    .lines
                    .iter()
                    .find(|l| ids.get(&l.id) == Some(&line))
                    .unwrap();
                let [a, b] = old.ends.map(|id| map(point(&original, id)));
                kind = C::Direction {
                    line,
                    angle: (b[1] - a[1]).atan2(b[0] - a[0]),
                };
            }
            if let C::DistanceX { points, parameter } = kind {
                kind = C::ProjectedDistance {
                    points,
                    parameter,
                    direction: bx,
                }
            }
            if let C::DistanceY { points, parameter } = kind {
                kind = C::ProjectedDistance {
                    points,
                    parameter,
                    direction: by,
                }
            }
            if matches!(c.kind, C::ProjectedDistance { .. })
                && let C::ProjectedDistance { direction, .. } = &mut kind
            {
                *direction = basis(*direction)
            }
            if let C::Direction { angle, .. } = &mut kind
                && !matches!(c.kind, C::Horizontal { .. } | C::Vertical { .. })
            {
                let v = basis([angle.cos(), angle.sin()]);
                *angle = v[1].atan2(v[0]);
            }
            if reflected && let C::LineOffset { side, .. } = &mut kind {
                *side = -*side;
            }
            if let Some(parameter) = kind.parameter() {
                let angular = matches!(kind, C::Angle { .. });
                let factor = if angular {
                    if reflected { -1. } else { 1. }
                } else {
                    scale
                };
                if (factor - 1.).abs() > 1e-8 {
                    let old = original
                        .parameters
                        .iter()
                        .find(|p| p.id == parameter)
                        .unwrap();
                    let next = d.parameter(
                        &format!("copy{}", d.parameters.len()),
                        format!("{} * {factor}", old.name),
                    );
                    d.parameters
                        .iter_mut()
                        .find(|p| p.id == next)
                        .unwrap()
                        .angular = angular;
                    let mut value = serde_json::to_value(&kind).map_err(|e| e.to_string())?;
                    value["parameter"] = serde_json::json!(next);
                    kind = serde_json::from_value(value).map_err(|e| e.to_string())?;
                }
            }
            if copy {
                d.constrain(kind)
            } else {
                d.constraints
                    .iter_mut()
                    .find(|constraint| constraint.id == c.id)
                    .unwrap()
                    .kind = kind;
            }
        }
    }
    Ok(result)
}
/// Tangent fillet between two connected straight edges. Radius persists as a driving parameter.
pub fn fillet(d: &mut Design, selected: &[Uuid], radius: f64) -> Result<(), String> {
    if selected.len() != 2 || radius <= 1e-7 {
        return Err("Select two lines and enter a positive radius".into());
    }
    let lines: Vec<_> = selected
        .iter()
        .map(|id| {
            d.lines
                .iter()
                .find(|l| l.id == *id)
                .cloned()
                .ok_or("Select two straight lines")
        })
        .collect::<Result<_, _>>()?;
    let common = *lines[0]
        .ends
        .iter()
        .find(|p| lines[1].ends.contains(p))
        .ok_or("Fillet lines must share a corner")?;
    let vertex = point(d, common);
    let far = lines
        .iter()
        .map(|l| *l.ends.iter().find(|p| **p != common).unwrap())
        .collect::<Vec<_>>();
    let coords = far.iter().map(|p| point(d, *p)).collect::<Vec<_>>();
    let v: Vec<_> = coords
        .iter()
        .map(|p| {
            let len = (p[0] - vertex[0]).hypot(p[1] - vertex[1]);
            [(p[0] - vertex[0]) / len, (p[1] - vertex[1]) / len]
        })
        .collect();
    let angle = (v[0][0] * v[1][0] + v[0][1] * v[1][1])
        .clamp(-1., 1.)
        .acos();
    if angle < 1e-5 || (std::f64::consts::PI - angle).abs() < 1e-5 {
        return Err("Fillet requires a non-collinear corner".into());
    }
    let distance = radius / (angle * 0.5).tan();
    if coords
        .iter()
        .any(|p| (p[0] - vertex[0]).hypot(p[1] - vertex[1]) <= distance)
    {
        return Err("Fillet radius is too large for these edges".into());
    }
    let tangent: Vec<_> = v
        .iter()
        .map(|v| [vertex[0] + v[0] * distance, vertex[1] + v[1] * distance])
        .collect();
    let bisector = [v[0][0] + v[1][0], v[0][1] + v[1][1]];
    let len = bisector[0].hypot(bisector[1]);
    let center = [
        vertex[0] + bisector[0] / len * radius / (angle * 0.5).sin(),
        vertex[1] + bisector[1] / len * radius / (angle * 0.5).sin(),
    ];
    for (i, l) in lines.iter().enumerate() {
        let next = d.point(tangent[i]);
        d.lines
            .iter_mut()
            .find(|line| line.id == l.id)
            .unwrap()
            .ends[usize::from(l.ends[1] == common)] = next;
        for c in &mut d.constraints {
            if let C::Length { line, parameter } = c.kind
                && line == l.id
            {
                c.kind = C::Distance {
                    points: l.ends,
                    parameter,
                }
            }
        }
    }
    let cross = (tangent[0][0] - center[0]) * (tangent[1][1] - center[1])
        - (tangent[0][1] - center[1]) * (tangent[1][0] - center[0]);
    let (start, end) = if cross > 0. {
        (tangent[0], tangent[1])
    } else {
        (tangent[1], tangent[0])
    };
    let arc = circle(d, center, start, Some(end))?;
    let parameter = d.parameter(
        &format!("fillet{}", d.parameters.len()),
        format!("{} mm", radius * 1000.),
    );
    d.constrain(C::Radius {
        circle: arc,
        parameter,
    });
    for l in &lines {
        d.constrain(C::Tangent {
            curves: [l.id, arc],
        })
    }
    Ok(())
}

fn line_circle(a: [f64; 2], b: [f64; 2], center: [f64; 2], radius: f64) -> Vec<f64> {
    let v = [b[0] - a[0], b[1] - a[1]];
    let q = [a[0] - center[0], a[1] - center[1]];
    let aa = v[0] * v[0] + v[1] * v[1];
    let bb = 2. * (v[0] * q[0] + v[1] * q[1]);
    let cc = q[0] * q[0] + q[1] * q[1] - radius * radius;
    let discriminant = bb * bb - 4. * aa * cc;
    if discriminant < -1e-16 || aa < 1e-20 {
        return vec![];
    }
    let h = discriminant.max(0.).sqrt();
    vec![(-bb - h) / (2. * aa), (-bb + h) / (2. * aa)]
}
fn on_arc(d: &Design, c: &Circle, p: [f64; 2]) -> bool {
    let Some(end) = c.end else { return true };
    let a = point(d, c.center);
    let start = point(d, c.rim);
    let end = point(d, end);
    let start = (start[1] - a[1]).atan2(start[0] - a[0]);
    let sweep = ((end[1] - a[1]).atan2(end[0] - a[0]) - start).rem_euclid(std::f64::consts::TAU);
    ((p[1] - a[1]).atan2(p[0] - a[0]) - start).rem_euclid(std::f64::consts::TAU) <= sweep + 1e-8
}
fn circle_intersections(d: &Design, c: &Circle) -> Vec<[f64; 2]> {
    let a = point(d, c.center);
    let r = crate::sketch::entities::radius(d, c);
    let mut result = Vec::new();
    for l in &d.lines {
        let [start, end] = l.ends.map(|id| point(d, id));
        for t in line_circle(start, end, a, r) {
            if (-1e-8..=1. + 1e-8).contains(&t) {
                let p = [
                    start[0] + t * (end[0] - start[0]),
                    start[1] + t * (end[1] - start[1]),
                ];
                if on_arc(d, c, p) {
                    result.push(p)
                }
            }
        }
    }
    for other in d.circles.iter().filter(|other| other.id != c.id) {
        let b = point(d, other.center);
        let s = crate::sketch::entities::radius(d, other);
        let distance = (b[0] - a[0]).hypot(b[1] - a[1]);
        if distance < 1e-10 || distance > r + s + 1e-10 || distance < (r - s).abs() - 1e-10 {
            continue;
        }
        let along = (r * r - s * s + distance * distance) / (2. * distance);
        let height = (r * r - along * along).max(0.).sqrt();
        let v = [(b[0] - a[0]) / distance, (b[1] - a[1]) / distance];
        for sign in [-1., 1.] {
            let p = [
                a[0] + v[0] * along - v[1] * height * sign,
                a[1] + v[1] * along + v[0] * height * sign,
            ];
            if on_arc(d, c, p) && on_arc(d, other, p) {
                result.push(p)
            }
        }
    }
    result
}
fn trim_circle(d: &mut Design, id: Uuid, p: [f64; 2]) -> Result<(), String> {
    let c = d.circles.iter().find(|c| c.id == id).unwrap().clone();
    let center = point(d, c.center);
    let rim = point(d, c.rim);
    let radius = crate::sketch::entities::radius(d, &c);
    let start = (rim[1] - center[1]).atan2(rim[0] - center[0]);
    let tau = std::f64::consts::TAU;
    let sweep = c.end.map_or(tau, |id| {
        let p = point(d, id);
        ((p[1] - center[1]).atan2(p[0] - center[0]) - start).rem_euclid(tau)
    });
    let angle = ((p[1] - center[1]).atan2(p[0] - center[0]) - start).rem_euclid(tau);
    let mut cuts = circle_intersections(d, &c)
        .iter()
        .map(|p| ((p[1] - center[1]).atan2(p[0] - center[0]) - start).rem_euclid(tau))
        .filter(|t| (c.end.is_none() || *t > 1e-8) && *t < sweep - 1e-8)
        .collect::<Vec<_>>();
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|a, b| (*a - *b).abs() < 1e-8);
    if cuts.is_empty() {
        delete(d, &[id]);
        return Ok(());
    }
    let at = |angle: f64| {
        [
            center[0] + radius * (start + angle).cos(),
            center[1] + radius * (start + angle).sin(),
        ]
    };
    let mut pieces = Vec::new();
    if c.end.is_none() {
        let lower = cuts
            .iter()
            .copied()
            .rfind(|t| *t < angle)
            .unwrap_or(*cuts.last().unwrap() - tau);
        let upper = cuts
            .iter()
            .copied()
            .find(|t| *t > angle)
            .unwrap_or(cuts[0] + tau);
        if upper - lower > tau - 1e-8 {
            return Err("Circle needs two distinct intersections to trim a segment".into());
        }
        pieces.push((upper, lower + tau));
    } else {
        let lower = cuts.iter().copied().rfind(|t| *t < angle).unwrap_or(0.);
        let upper = cuts.iter().copied().find(|t| *t > angle).unwrap_or(sweep);
        if lower > 1e-8 {
            pieces.push((0., lower))
        }
        if upper < sweep - 1e-8 {
            pieces.push((upper, sweep))
        }
    }
    let construction = d.construction_geometry.contains(&id);
    let dimensions=d.constraints.iter().filter(|constraint|matches!(constraint.kind,C::Radius{circle,..}|C::Diameter{circle,..} if circle==id)).cloned().collect::<Vec<_>>();
    delete(d, &[id]);
    for (start, end) in pieces {
        let next = circle(d, center, at(start), Some(at(end)))?;
        if construction {
            d.construction_geometry.push(next)
        }
        for constraint in &dimensions {
            let kind = match constraint.kind {
                C::Radius { parameter, .. } => C::Radius {
                    circle: next,
                    parameter,
                },
                C::Diameter { parameter, .. } => C::Diameter {
                    circle: next,
                    parameter,
                },
                _ => unreachable!(),
            };
            d.constrain(kind)
        }
    }
    Ok(())
}
fn break_circle(d: &mut Design, id: Uuid, p: [f64; 2]) -> Result<(), String> {
    let original = d.circles.iter().find(|c| c.id == id).unwrap().clone();
    let center = point(d, original.center);
    let radius = crate::sketch::entities::radius(d, &original);
    let angle = (p[1] - center[1]).atan2(p[0] - center[0]);
    let at = [
        center[0] + radius * angle.cos(),
        center[1] + radius * angle.sin(),
    ];
    let Some(end) = original.end else {
        return Err("Break requires an arc; trim a circle at two intersections first".into());
    };
    if !on_arc(d, &original, at) {
        return Err("Break point must lie inside the arc".into());
    }
    let endpoint = point(d, end);
    if (at[0] - endpoint[0]).hypot(at[1] - endpoint[1]) < 1e-7
        || (at[0] - point(d, original.rim)[0]).hypot(at[1] - point(d, original.rim)[1]) < 1e-7
    {
        return Err("Break point must be inside the arc".into());
    }
    let split = d.point(at);
    d.circles.iter_mut().find(|c| c.id == id).unwrap().end = Some(split);
    let next = circle(d, center, at, Some(endpoint))?;
    d.constrain(C::Equal { curves: [id, next] });
    if d.construction_geometry.contains(&id) {
        d.construction_geometry.push(next)
    }
    Ok(())
}

fn extend_arc(d: &mut Design, id: Uuid, p: [f64; 2]) -> Result<(), String> {
    let c = d.circles.iter().find(|c| c.id == id).unwrap().clone();
    let Some(end) = c.end else {
        return Err("Select an arc endpoint to extend".into());
    };
    let center = point(d, c.center);
    let start = point(d, c.rim);
    let endpoint = point(d, end);
    let angle = (start[1] - center[1]).atan2(start[0] - center[0]);
    let tau = std::f64::consts::TAU;
    let sweep = ((endpoint[1] - center[1]).atan2(endpoint[0] - center[0]) - angle).rem_euclid(tau);
    let from_start =
        (p[0] - start[0]).hypot(p[1] - start[1]) < (p[0] - endpoint[0]).hypot(p[1] - endpoint[1]);
    let mut full = c.clone();
    full.end = None;
    let mut candidates = circle_intersections(d, &full)
        .into_iter()
        .filter_map(|p| {
            let t = ((p[1] - center[1]).atan2(p[0] - center[0]) - angle).rem_euclid(tau);
            (t > sweep + 1e-8 && t < tau - 1e-8).then_some((t, p))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
    let at = if from_start {
        candidates.last()
    } else {
        candidates.first()
    }
    .ok_or("No intersection to extend the arc to")?
    .1;
    let point = d.point(at);
    let arc = d.circles.iter_mut().find(|c| c.id == id).unwrap();
    if from_start {
        arc.rim = point
    } else {
        arc.end = Some(point)
    }
    Ok(())
}
