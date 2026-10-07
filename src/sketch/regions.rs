//! Closed exact line/arc loops and persistent boundary-based region selection.
use crate::{document::schema::Design, sketch::entities};
use uuid::Uuid;
#[derive(Clone, Debug)]
pub struct Edge {
    pub id: Uuid,
    pub start: [f64; 2],
    pub end: [f64; 2],
    pub center: [f64; 2],
    /// Zero for a line, signed radians for a circular arc (TAU for a circle).
    pub sweep: f64,
}
impl Edge {
    fn reverse(&mut self) {
        std::mem::swap(&mut self.start, &mut self.end);
        self.sweep = -self.sweep;
    }
    pub fn samples(&self) -> Vec<[f64; 2]> {
        if self.sweep == 0. {
            return vec![self.start, self.end];
        }
        let r = distance(self.start, self.center);
        let a = (self.start[1] - self.center[1]).atan2(self.start[0] - self.center[0]);
        let n = (self.sweep.abs() / std::f64::consts::TAU * 192.).ceil() as usize;
        (0..=n)
            .map(|i| {
                let t = a + self.sweep * i as f64 / n as f64;
                [self.center[0] + r * t.cos(), self.center[1] + r * t.sin()]
            })
            .collect()
    }
}
#[derive(Clone, Debug)]
pub struct Region {
    pub boundary: Vec<Uuid>,
    pub wires: Vec<Vec<Edge>>,
}
fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
fn on_arc(e: &Edge, p: [f64; 2]) -> bool {
    if e.sweep.abs() >= std::f64::consts::TAU - 1e-8 {
        return true;
    }
    let angle = |p: [f64; 2]| (p[1] - e.center[1]).atan2(p[0] - e.center[0]);
    let delta = ((angle(p) - angle(e.start)) * e.sweep.signum()).rem_euclid(std::f64::consts::TAU);
    delta <= e.sweep.abs() + 1e-8 || distance(p, e.start) < 1e-7
}
/// Exact circular intersection candidates, including tangencies between sample vertices.
fn circular_intersections(a: &Edge, b: &Edge) -> Vec<[f64; 2]> {
    if a.sweep == 0. {
        return circular_intersections(b, a);
    }
    let radius = distance(a.start, a.center);
    let mut points = vec![];
    if b.sweep == 0. {
        let v = [b.end[0] - b.start[0], b.end[1] - b.start[1]];
        let f = [b.start[0] - a.center[0], b.start[1] - a.center[1]];
        let aa = v[0] * v[0] + v[1] * v[1];
        let bb = 2. * (f[0] * v[0] + f[1] * v[1]);
        let cc = f[0] * f[0] + f[1] * f[1] - radius * radius;
        let disc = bb * bb - 4. * aa * cc;
        if disc >= -1e-20 {
            for t in [
                (-bb - disc.max(0.).sqrt()) / (2. * aa),
                (-bb + disc.max(0.).sqrt()) / (2. * aa),
            ] {
                if (-1e-8..=1. + 1e-8).contains(&t) {
                    points.push([b.start[0] + t * v[0], b.start[1] + t * v[1]]);
                }
            }
        }
    } else {
        let r = distance(b.start, b.center);
        let d = distance(a.center, b.center);
        if d < 1e-10 && (r - radius).abs() < 1e-10 {
            // Coincident arcs: interior samples expose overlap; shared endpoints remain valid.
            points.extend(a.samples().into_iter().skip(1).filter(|p| on_arc(b, *p)));
            points.extend(b.samples().into_iter().skip(1).filter(|p| on_arc(a, *p)));
        } else if d > 1e-10 && d <= radius + r + 1e-10 && d >= (radius - r).abs() - 1e-10 {
            let x = (radius * radius - r * r + d * d) / (2. * d);
            let h = (radius * radius - x * x).max(0.).sqrt();
            let v = [
                (b.center[0] - a.center[0]) / d,
                (b.center[1] - a.center[1]) / d,
            ];
            for sign in [-1., 1.] {
                points.push([
                    a.center[0] + x * v[0] - sign * h * v[1],
                    a.center[1] + x * v[1] + sign * h * v[0],
                ]);
            }
        }
    }
    points
        .into_iter()
        .filter(|p| on_arc(a, *p) && (b.sweep == 0. || on_arc(b, *p)))
        .collect()
}
fn contains(poly: &[[f64; 2]], p: [f64; 2]) -> bool {
    let mut inside = false;
    for ab in poly.windows(2) {
        let [a, b] = [ab[0], ab[1]];
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
    }
    inside
}
/// Pick the bounded region under a sketch-plane point, excluding its holes.
pub fn at(d: &Design, xy: &[[f64; 2]], point: [f64; 2]) -> Result<Option<Region>, String> {
    Ok(regions(d, xy)?.into_iter().find(|region| {
        let polygons: Vec<Vec<_>> = region
            .wires
            .iter()
            .map(|wire| {
                let mut polygon: Vec<_> = wire.iter().flat_map(Edge::samples).collect();
                if let Some(first) = polygon.first().copied() {
                    polygon.push(first);
                }
                polygon
            })
            .collect();
        polygons.first().is_some_and(|outer| contains(outer, point))
            && !polygons.iter().skip(1).any(|hole| contains(hole, point))
    }))
}

/// Reject ambiguous/open/intersecting contours rather than silently changing the region.
pub fn regions(d: &Design, xy: &[[f64; 2]]) -> Result<Vec<Region>, String> {
    d.validate()?;
    if xy.len() != d.points.len() || xy.iter().flatten().any(|v| !v.is_finite()) {
        return Err("Invalid solved profile points".into());
    }
    let mut solved = d.clone();
    for (p, xy) in solved.points.iter_mut().zip(xy) {
        p.xy = *xy;
    }
    let d = &solved;
    let active = |id: &Uuid| !d.construction_geometry.contains(id);
    if d.ellipses.iter().any(|e| active(&e.id)) || d.splines.iter().any(|e| active(&e.id)) {
        return Err(
            "Extrusion supports lines, circles and arcs; mark other curves as construction".into(),
        );
    }
    let mut edges = Vec::new();
    for l in d.lines.iter().filter(|l| active(&l.id)) {
        edges.push(Edge {
            id: l.id,
            start: entities::point(d, l.ends[0]),
            end: entities::point(d, l.ends[1]),
            center: [0.; 2],
            sweep: 0.,
        });
    }
    for c in d.circles.iter().filter(|c| active(&c.id)) {
        let start = entities::point(d, c.rim);
        let center = entities::point(d, c.center);
        let end = c.end.map_or(start, |id| entities::point(d, id));
        let sweep = c.end.map_or(std::f64::consts::TAU, |_| {
            ((end[1] - center[1]).atan2(end[0] - center[0])
                - (start[1] - center[1]).atan2(start[0] - center[0]))
            .rem_euclid(std::f64::consts::TAU)
        });
        if distance(start, center) < 1e-7
            || sweep < 1e-8
            || (distance(end, center) - distance(start, center)).abs() > 1e-7
        {
            return Err("Degenerate circular profile".into());
        }
        edges.push(Edge {
            id: c.id,
            start,
            end,
            center,
            sweep,
        });
    }
    if edges.is_empty() {
        return Err("Draw a closed profile first".into());
    }
    for e in &edges {
        if e.sweep == 0. && distance(e.start, e.end) < 1e-7 {
            return Err("Profile has a zero-length edge".into());
        }
    }
    let mut loops: Vec<Vec<Edge>> = Vec::new();
    while !edges.is_empty() {
        let first = edges.remove(0);
        let start = first.start;
        let mut wire = vec![first];
        while distance(wire.last().unwrap().end, start) > 1e-7 {
            let end = wire.last().unwrap().end;
            let candidates: Vec<_> = edges
                .iter()
                .enumerate()
                .filter(|(_, e)| distance(e.start, end) < 1e-7 || distance(e.end, end) < 1e-7)
                .map(|(i, _)| i)
                .collect();
            if candidates.len() != 1 {
                return Err("Profile is open or branches; connect every endpoint".into());
            }
            let mut e = edges.remove(candidates[0]);
            if distance(e.start, end) > 1e-7 {
                e.reverse();
            }
            wire.push(e);
        }
        loops.push(wire);
    }
    for (i, wire) in loops.iter().enumerate() {
        for (a, e) in wire.iter().enumerate() {
            for (j, other) in loops.iter().enumerate().skip(i) {
                for (b, f) in other.iter().enumerate() {
                    if i == j && b <= a {
                        continue;
                    }
                    if e.sweep == 0. && f.sweep == 0. {
                        continue;
                    }
                    let adjacent = i == j && (b == a + 1 || (a == 0 && b == wire.len() - 1));
                    for p in circular_intersections(e, f) {
                        let shared = adjacent
                            && [e.start, e.end].iter().any(|q| distance(*q, p) < 1e-7)
                            && [f.start, f.end].iter().any(|q| distance(*q, p) < 1e-7);
                        if !shared {
                            return Err("Profiles intersect or touch".into());
                        }
                    }
                }
            }
        }
    }
    let polygons: Vec<Vec<_>> = loops
        .iter()
        .map(|w| {
            let mut p = vec![];
            for e in w {
                let s = e.samples();
                p.extend_from_slice(&s[..s.len() - 1]);
            }
            p.push(p[0]);
            p
        })
        .collect();
    // Sampling is used only for classification and rejection. Native edges remain exact.
    for (i, p) in polygons.iter().enumerate() {
        for (a, ab) in p.windows(2).enumerate() {
            for (j, q) in polygons.iter().enumerate().skip(i) {
                for (b, cd) in q.windows(2).enumerate() {
                    if i == j && (b <= a + 1 || (a == 0 && b == p.len() - 2)) {
                        continue;
                    }
                    let cross = |u: [f64; 2], v: [f64; 2], w: [f64; 2]| {
                        (v[0] - u[0]) * (w[1] - u[1]) - (v[1] - u[1]) * (w[0] - u[0])
                    };
                    if cross(ab[0], ab[1], cd[0]) * cross(ab[0], ab[1], cd[1]) <= 0.
                        && cross(cd[0], cd[1], ab[0]) * cross(cd[0], cd[1], ab[1]) <= 0.
                        && ab[0][0].min(ab[1][0]) <= cd[0][0].max(cd[1][0]) + 1e-10
                        && cd[0][0].min(cd[1][0]) <= ab[0][0].max(ab[1][0]) + 1e-10
                        && ab[0][1].min(ab[1][1]) <= cd[0][1].max(cd[1][1]) + 1e-10
                        && cd[0][1].min(cd[1][1]) <= ab[0][1].max(ab[1][1]) + 1e-10
                    {
                        return Err("Profiles intersect or touch".into());
                    }
                }
            }
        }
    }
    let parents: Vec<Vec<usize>> = polygons
        .iter()
        .enumerate()
        .map(|(i, p)| {
            polygons
                .iter()
                .enumerate()
                .filter(|(j, q)| *j != i && contains(q, p[0]))
                .map(|(j, _)| j)
                .collect()
        })
        .collect();
    let mut result = vec![];
    for i in 0..loops.len() {
        let mut wires = vec![loops[i].clone()];
        for j in 0..loops.len() {
            if parents[j].contains(&i) && parents[j].len() == parents[i].len() + 1 {
                wires.push(loops[j].clone());
            }
        }
        // Each closed boundary defines a selectable region, with immediate children as holes.
        for (k, w) in wires.iter_mut().enumerate() {
            let area: f64 = w
                .iter()
                .flat_map(Edge::samples)
                .collect::<Vec<_>>()
                .windows(2)
                .map(|p| p[0][0] * p[1][1] - p[1][0] * p[0][1])
                .sum();
            if (area > 0.) != (k == 0) {
                w.reverse();
                for e in w {
                    e.reverse();
                }
            }
        }
        result.push(Region {
            boundary: loops[i].iter().map(|e| e.id).collect(),
            wires,
        });
    }
    Ok(result)
}
pub fn select(d: &Design, xy: &[[f64; 2]], boundary: &[Uuid]) -> Result<Region, String> {
    let all = regions(d, xy)?;
    if !boundary.is_empty() {
        return all
            .into_iter()
            .find(|r| {
                r.boundary.len() == boundary.len()
                    && boundary.iter().all(|id| r.boundary.contains(id))
            })
            .ok_or("Selected extrusion boundary changed; select the region again".into());
    }
    let outer: Vec<_> = all
        .iter()
        .filter(|r| {
            !all.iter().any(|other| {
                other
                    .wires
                    .iter()
                    .skip(1)
                    .any(|w| w.iter().any(|e| r.boundary.contains(&e.id)))
            })
        })
        .collect();
    if outer.len() != 1 {
        return Err("Multiple regions: select a boundary curve before extruding".into());
    }
    Ok(outer[0].clone())
}
#[cfg(feature = "kernel")]
pub fn extrude(region: &Region, depth: f64) -> Result<crate::kernel::bridge::ffi::Mesh, String> {
    use crate::kernel::bridge::ffi;
    let edges: Vec<_> = region
        .wires
        .iter()
        .enumerate()
        .flat_map(|(wire, edges)| {
            edges.iter().map(move |e| ffi::ProfileEdge {
                wire: wire as u32,
                sx: e.start[0],
                sy: e.start[1],
                ex: e.end[0],
                ey: e.end[1],
                cx: e.center[0],
                cy: e.center[1],
                sweep: e.sweep,
            })
        })
        .collect();
    ffi::extrude_region(&edges, depth).map_err(|e| e.to_string())
}

#[cfg(test)]
mod picking_tests {
    use super::*;
    #[test]
    fn picks_nested_regions_and_rejects_empty_space() {
        let mut d = Design::default();
        let outer = crate::sketch::edit::circle(&mut d, [0., 0.], [0.03, 0.], None).unwrap();
        let inner = crate::sketch::edit::circle(&mut d, [0., 0.], [0.01, 0.], None).unwrap();
        let xy: Vec<_> = d.points.iter().map(|p| p.xy).collect();
        assert!(
            at(&d, &xy, [0.02, 0.])
                .unwrap()
                .unwrap()
                .boundary
                .contains(&outer)
        );
        assert!(
            at(&d, &xy, [0., 0.])
                .unwrap()
                .unwrap()
                .boundary
                .contains(&inner)
        );
        assert!(at(&d, &xy, [0.05, 0.]).unwrap().is_none());
    }
}
