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
    fn samples(&self) -> Vec<[f64; 2]> {
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
/// Reject ambiguous/open/intersecting contours rather than silently changing the region.
pub fn regions(d: &Design, xy: &[[f64; 2]]) -> Result<Vec<Region>, String> {
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
