//! Planar sketch arrangements: split exact curves at intersections and walk bounded faces.
use crate::{document::schema::Design, sketch::entities};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum Curve {
    Line([f64; 2], [f64; 2]),
    Circle {
        center: [f64; 2],
        radius: f64,
        start: f64,
        sweep: f64,
    },
    Ellipse {
        center: [f64; 2],
        major: [f64; 2],
        minor: [f64; 2],
    },
    Bezier(Vec<[f64; 2]>),
    BSpline(Vec<[f64; 2]>),
}
impl Curve {
    pub fn point(&self, t: f64) -> [f64; 2] {
        match self {
            Self::Line(a, b) => std::array::from_fn(|i| a[i] + t * (b[i] - a[i])),
            Self::Circle {
                center,
                radius,
                start,
                sweep,
            } => {
                let a = start + sweep * t;
                [center[0] + radius * a.cos(), center[1] + radius * a.sin()]
            }
            Self::Ellipse {
                center,
                major,
                minor,
            } => {
                let a = std::f64::consts::TAU * t;
                std::array::from_fn(|i| center[i] + major[i] * a.cos() + minor[i] * a.sin())
            }
            Self::Bezier(poles) => {
                let mut p = poles.clone();
                for n in (1..p.len()).rev() {
                    for i in 0..n {
                        p[i] = std::array::from_fn(|j| (1. - t) * p[i][j] + t * p[i + 1][j]);
                    }
                }
                p[0]
            }
            Self::BSpline(poles) => {
                let degree = (poles.len() - 1).min(3);
                let count = poles.len();
                let knot = |i: usize| {
                    if i <= degree {
                        0.
                    } else if i >= count {
                        1.
                    } else {
                        (i - degree) as f64 / (count - degree) as f64
                    }
                };
                let span = if t >= 1. {
                    count - 1
                } else {
                    (degree..count)
                        .find(|k| t >= knot(*k) && t < knot(*k + 1))
                        .unwrap_or(degree)
                };
                let mut p: Vec<_> = (span - degree..=span).map(|k| poles[k]).collect();
                for r in 1..=degree {
                    for j in (r..=degree).rev() {
                        let k = span - degree + j;
                        let a = (t - knot(k)) / (knot(k + degree + 1 - r) - knot(k));
                        p[j] = std::array::from_fn(|i| (1. - a) * p[j - 1][i] + a * p[j][i]);
                    }
                }
                p[degree]
            }
        }
    }
    fn tangent(&self, t: f64) -> [f64; 2] {
        let a = self.point((t - 1e-6).max(0.));
        let b = self.point((t + 1e-6).min(1.));
        [b[0] - a[0], b[1] - a[1]]
    }
    fn subdivisions(&self) -> usize {
        if matches!(self, Self::Line(..)) {
            1
        } else {
            192
        }
    }
}
#[derive(Clone, Debug)]
pub struct Edge {
    /// Source sketch curve, separate from the persistent split-boundary token.
    pub id: Uuid,
    pub start: [f64; 2],
    pub end: [f64; 2],
    pub center: [f64; 2],
    pub sweep: f64,
    pub curve: Curve,
    pub from: f64,
    pub to: f64,
    token: Uuid,
}
impl Edge {
    fn reverse(&mut self) {
        std::mem::swap(&mut self.start, &mut self.end);
        std::mem::swap(&mut self.from, &mut self.to);
        self.sweep = -self.sweep;
    }
    pub fn samples(&self) -> Vec<[f64; 2]> {
        let n = ((self.to - self.from).abs() * self.curve.subdivisions() as f64)
            .ceil()
            .max(1.) as usize;
        (0..=n)
            .map(|i| {
                self.curve
                    .point(self.from + (self.to - self.from) * i as f64 / n as f64)
            })
            .collect()
    }
    #[cfg(feature = "kernel")]
    pub fn native(&self, wire: u32) -> crate::kernel::bridge::ffi::ProfileEdge {
        use crate::kernel::bridge::ffi::{ProfileEdge, ProfilePoint};
        let mut edge = ProfileEdge {
            wire,
            sx: self.start[0],
            sy: self.start[1],
            ex: self.end[0],
            ey: self.end[1],
            cx: self.center[0],
            cy: self.center[1],
            sweep: self.sweep,
            kind: 0,
            poles: vec![],
            identity: self.token.to_string(),
            from: self.from,
            to: self.to,
        };
        let data = match &self.curve {
            Curve::Line(..) | Curve::Circle { .. } => return edge,
            Curve::Ellipse {
                center,
                major,
                minor,
            } => {
                edge.kind = 1;
                vec![*center, *major, *minor]
            }
            Curve::Bezier(p) => {
                edge.kind = 2;
                p.clone()
            }
            Curve::BSpline(p) => {
                edge.kind = 3;
                p.clone()
            }
        };
        edge.poles = data
            .into_iter()
            .map(|p| ProfilePoint { x: p[0], y: p[1] })
            .collect();
        edge
    }
}
#[derive(Clone, Debug)]
pub struct Region {
    pub nested: bool,
    pub boundary: Vec<Uuid>,
    pub curves: Vec<Uuid>,
    pub wires: Vec<Vec<Edge>>,
}
fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
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
fn polygon(wire: &[Edge]) -> Vec<[f64; 2]> {
    let mut p = vec![];
    for e in wire {
        let s = e.samples();
        p.extend_from_slice(&s[..s.len() - 1]);
    }
    if let Some(first) = p.first().copied() {
        p.push(first);
    }
    p
}
fn area(wire: &[Edge]) -> f64 {
    polygon(wire)
        .windows(2)
        .map(|p| cross(p[0], p[1]))
        .sum::<f64>()
        * 0.5
}
pub fn at(d: &Design, xy: &[[f64; 2]], point: [f64; 2]) -> Result<Option<Region>, String> {
    Ok(regions(d, xy)?.into_iter().find(|r| {
        contains(&polygon(&r.wires[0]), point)
            && !r.wires.iter().skip(1).any(|w| contains(&polygon(w), point))
    }))
}
#[derive(Clone)]
struct Source {
    id: Uuid,
    piece: usize,
    curve: Curve,
}
fn sources(d: &Design) -> Result<Vec<Source>, String> {
    let mut result = vec![];
    for id in entities::curve_ids(d)
        .into_iter()
        .filter(|id| !d.construction_geometry.contains(id))
    {
        let mut push = |piece, curve| result.push(Source { id, piece, curve });
        if let Some(l) = d.lines.iter().find(|l| l.id == id) {
            push(
                0,
                Curve::Line(entities::point(d, l.ends[0]), entities::point(d, l.ends[1])),
            );
        }
        if let Some(c) = d.circles.iter().find(|c| c.id == id) {
            let center = entities::point(d, c.center);
            let rim = entities::point(d, c.rim);
            let start = (rim[1] - center[1]).atan2(rim[0] - center[0]);
            let sweep = c.end.map_or(std::f64::consts::TAU, |id| {
                let p = entities::point(d, id);
                ((p[1] - center[1]).atan2(p[0] - center[0]) - start)
                    .rem_euclid(std::f64::consts::TAU)
            });
            if distance(center, rim) < 1e-7 || sweep < 1e-8 {
                return Err("Degenerate circular profile".into());
            }
            push(
                0,
                Curve::Circle {
                    center,
                    radius: distance(center, rim),
                    start,
                    sweep,
                },
            );
        }
        if let Some(e) = d.ellipses.iter().find(|e| e.id == id) {
            let center = entities::point(d, e.center);
            let a = entities::point(d, e.major);
            let b = entities::point(d, e.minor);
            push(
                0,
                Curve::Ellipse {
                    center,
                    major: [a[0] - center[0], a[1] - center[1]],
                    minor: [b[0] - center[0], b[1] - center[1]],
                },
            );
        }
        if let Some(s) = d.splines.iter().find(|s| s.id == id) {
            let p: Vec<_> = s.points.iter().map(|id| entities::point(d, *id)).collect();
            if s.fit {
                for i in 0..p.len() - 1 {
                    let a = p[i.saturating_sub(1)];
                    let b = p[i];
                    let c = p[i + 1];
                    let e = p[(i + 2).min(p.len() - 1)];
                    push(
                        i,
                        Curve::Bezier(vec![
                            b,
                            std::array::from_fn(|j| b[j] + (c[j] - a[j]) / 6.),
                            std::array::from_fn(|j| c[j] - (e[j] - b[j]) / 6.),
                            c,
                        ]),
                    );
                }
            } else {
                push(0, Curve::BSpline(p));
            }
        }
    }
    Ok(result)
}
// Sampling supplies intersection seeds only. Newton refinement splits the original
// analytic curves; extrusion never substitutes a polygon for a curved boundary.
fn approximate_intersections(a: &Curve, b: &Curve, same: bool) -> Vec<(f64, f64)> {
    if matches!(a, Curve::Circle { .. }) && matches!(b, Curve::Circle { .. } | Curve::Line(..)) {
        let Curve::Circle {
            center,
            radius,
            start,
            sweep,
        } = a
        else {
            unreachable!()
        };
        if same {
            return vec![];
        }
        let mut points = vec![];
        match b {
            Curve::Line(p, q) => {
                let v = [q[0] - p[0], q[1] - p[1]];
                let f = [p[0] - center[0], p[1] - center[1]];
                let aa = v[0] * v[0] + v[1] * v[1];
                let bb = 2. * (f[0] * v[0] + f[1] * v[1]);
                let cc = f[0] * f[0] + f[1] * f[1] - radius * radius;
                let disc = bb * bb - 4. * aa * cc;
                if aa > 1e-20 && disc >= -1e-20 {
                    for u in [
                        (-bb - disc.max(0.).sqrt()) / (2. * aa),
                        (-bb + disc.max(0.).sqrt()) / (2. * aa),
                    ] {
                        if (-1e-8..=1. + 1e-8).contains(&u) {
                            points.push(([p[0] + u * v[0], p[1] + u * v[1]], u.clamp(0., 1.)));
                        }
                    }
                }
            }
            Curve::Circle {
                center: c,
                radius: r,
                start: s,
                sweep: w,
            } => {
                let d = distance(*center, *c);
                if d < 1e-10 && (radius - r).abs() < 1e-10 {
                    for t in [0., 1.] {
                        let p = a.point(t);
                        let u = ((p[1] - c[1]).atan2(p[0] - c[0]) - s)
                            .rem_euclid(std::f64::consts::TAU)
                            / w;
                        if u <= 1. + 1e-8 {
                            points.push((p, u.min(1.)));
                        }
                    }
                    for u in [0., 1.] {
                        points.push((b.point(u), u));
                    }
                }
                if d > 1e-10 && d <= radius + r + 1e-10 && d >= (radius - r).abs() - 1e-10 {
                    let x = (radius * radius - r * r + d * d) / (2. * d);
                    let h = (radius * radius - x * x).max(0.).sqrt();
                    let v = [(c[0] - center[0]) / d, (c[1] - center[1]) / d];
                    for sign in [-1., 1.] {
                        let p = [
                            center[0] + x * v[0] - sign * h * v[1],
                            center[1] + x * v[1] + sign * h * v[0],
                        ];
                        let u = ((p[1] - c[1]).atan2(p[0] - c[0]) - s)
                            .rem_euclid(std::f64::consts::TAU)
                            / w;
                        if u <= 1. + 1e-8 {
                            points.push((p, u.min(1.)));
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        return points
            .into_iter()
            .filter_map(|(p, u)| {
                let t = ((p[1] - center[1]).atan2(p[0] - center[0]) - start)
                    .rem_euclid(std::f64::consts::TAU)
                    / sweep;
                (t <= 1. + 1e-8).then_some((t.min(1.), u))
            })
            .collect();
    }
    if matches!(b, Curve::Circle { .. }) && matches!(a, Curve::Line(..)) {
        return approximate_intersections(b, a, false)
            .into_iter()
            .map(|(t, u)| (u, t))
            .collect();
    }
    let na = a.subdivisions();
    let nb = b.subdivisions();
    let pa: Vec<_> = (0..=na).map(|i| a.point(i as f64 / na as f64)).collect();
    let pb: Vec<_> = (0..=nb).map(|i| b.point(i as f64 / nb as f64)).collect();
    let mut hits = vec![];
    for i in 0..na {
        for j in 0..nb {
            if same && (j <= i + 1 || (i == 0 && j == nb - 1)) {
                continue;
            }
            if (0..2).any(|axis| {
                pa[i][axis].min(pa[i + 1][axis]) > pb[j][axis].max(pb[j + 1][axis]) + 1e-9
                    || pb[j][axis].min(pb[j + 1][axis]) > pa[i][axis].max(pa[i + 1][axis]) + 1e-9
            }) {
                continue;
            }
            let p = pa[i];
            let q = pb[j];
            let v = [pa[i + 1][0] - p[0], pa[i + 1][1] - p[1]];
            let w = [pb[j + 1][0] - q[0], pb[j + 1][1] - q[1]];
            let delta = [q[0] - p[0], q[1] - p[1]];
            let det = cross(v, w);
            if det.abs() < 1e-20 {
                // Coincident line segments: split at overlap endpoints.
                if !same
                    && matches!(a, Curve::Line(..))
                    && matches!(b, Curve::Line(..))
                    && cross(delta, v).abs() < 1e-14
                {
                    let dot = |x: [f64; 2], y: [f64; 2]| x[0] * y[0] + x[1] * y[1];
                    for (t, point) in [(0., p), (1., pa[i + 1])] {
                        let u = dot([point[0] - q[0], point[1] - q[1]], w) / dot(w, w);
                        if (-1e-9..=1. + 1e-9).contains(&u) {
                            hits.push((t, u.clamp(0., 1.)));
                        }
                    }
                    for (u, point) in [(0., q), (1., pb[j + 1])] {
                        let t = dot([point[0] - p[0], point[1] - p[1]], v) / dot(v, v);
                        if (-1e-9..=1. + 1e-9).contains(&t) {
                            hits.push((t.clamp(0., 1.), u));
                        }
                    }
                }
                continue;
            }
            let s = cross(delta, w) / det;
            let r = cross(delta, v) / det;
            if !(-0.02..=1.02).contains(&s) || !(-0.02..=1.02).contains(&r) {
                continue;
            }
            let mut t = (i as f64 + s) / na as f64;
            let mut u = (j as f64 + r) / nb as f64;
            for _ in 0..20 {
                let p = a.point(t.clamp(0., 1.));
                let q = b.point(u.clamp(0., 1.));
                let da = a.tangent(t.clamp(0., 1.));
                let db = b.tangent(u.clamp(0., 1.));
                let h = 1e-6;
                let sa = ((t + h).min(1.) - (t - h).max(0.)).max(1e-12);
                let sb = ((u + h).min(1.) - (u - h).max(0.)).max(1e-12);
                let va = [da[0] / sa, da[1] / sa];
                let vb = [db[0] / sb, db[1] / sb];
                let det = cross(va, vb);
                if det.abs() < 1e-18 {
                    break;
                }
                let delta = [q[0] - p[0], q[1] - p[1]];
                t += cross(delta, vb) / det;
                u += cross(delta, va) / det;
                if distance(p, q) < 1e-11 {
                    break;
                }
            }
            if (-1e-7..=1. + 1e-7).contains(&t)
                && (-1e-7..=1. + 1e-7).contains(&u)
                && distance(a.point(t.clamp(0., 1.)), b.point(u.clamp(0., 1.))) < 1e-8
            {
                let hit = (t.clamp(0., 1.), u.clamp(0., 1.));
                if !hits
                    .iter()
                    .any(|&(x, y)| (x - hit.0).abs() < 1e-6 && (y - hit.1).abs() < 1e-6)
                {
                    hits.push(hit);
                }
            }
        }
    }
    hits
}
fn intersections(a: &Curve, b: &Curve, same: bool) -> Result<Vec<(f64, f64)>, String> {
    #[cfg(feature = "kernel")]
    if !matches!(
        (a, b),
        (
            Curve::Line(..) | Curve::Circle { .. },
            Curve::Line(..) | Curve::Circle { .. }
        )
    ) {
        let native = |curve: &Curve| {
            let start = curve.point(0.);
            let end = curve.point(1.);
            let (center, sweep) = match curve {
                Curve::Circle { center, sweep, .. } => (*center, *sweep),
                _ => ([0.; 2], 0.),
            };
            Edge {
                id: Uuid::nil(),
                start,
                end,
                center,
                sweep,
                curve: curve.clone(),
                from: 0.,
                to: 1.,
                token: Uuid::nil(),
            }
            .native(0)
        };
        return crate::kernel::bridge::ffi::profile_intersections(&native(a), &native(b), same)
            .map(|hits| hits.into_iter().map(|p| (p.first, p.second)).collect())
            .map_err(|e| format!("Sketch intersection failed: {e}"));
    }
    Ok(approximate_intersections(a, b, same))
}
fn token(
    source: &Source,
    part: usize,
    pieces: usize,
    crossings: &[(f64, Uuid, usize)],
    interval: &[f64],
) -> Uuid {
    if pieces == 1 && source.piece == 0 {
        return source.id;
    }
    let mut h = blake3::Hasher::new();
    h.update(b"sketch-region-segment-v1");
    h.update(source.id.as_bytes());
    h.update(&(source.piece as u64).to_le_bytes());
    h.update(&(part as u64).to_le_bytes());
    h.update(&(pieces as u64).to_le_bytes());
    for endpoint in interval {
        let mut partners: Vec<_> = crossings
            .iter()
            .filter(|(t, _, _)| (*t - *endpoint).abs() < 1e-7)
            .map(|(_, id, piece)| (*id, *piece))
            .collect();
        partners.sort();
        partners.dedup();
        h.update(&(partners.len() as u64).to_le_bytes());
        for (id, piece) in partners {
            h.update(id.as_bytes());
            h.update(&(piece as u64).to_le_bytes());
        }
    }
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&h.finalize().as_bytes()[..16]);
    bytes[6] = (bytes[6] & 15) | 0x80;
    bytes[8] = (bytes[8] & 63) | 0x80;
    Uuid::from_bytes(bytes)
}
/// Extract all bounded faces. Open tails are ignored; crossings and shared edges
/// subdivide regions instead of invalidating unrelated closed geometry.
pub fn regions(d: &Design, xy: &[[f64; 2]]) -> Result<Vec<Region>, String> {
    d.validate()?;
    if xy.len() != d.points.len() || xy.iter().flatten().any(|v| !v.is_finite()) {
        return Err("Invalid solved profile points".into());
    }
    let mut d = d.clone();
    for (p, xy) in d.points.iter_mut().zip(xy) {
        p.xy = *xy;
    }
    let sources = sources(&d)?;
    let mut cuts = vec![vec![0., 1.]; sources.len()];
    let mut crossings: Vec<Vec<(f64, Uuid, usize)>> = vec![vec![]; sources.len()];
    for i in 0..sources.len() {
        for j in i..sources.len() {
            for (t, u) in intersections(&sources[i].curve, &sources[j].curve, i == j)? {
                cuts[i].push(t);
                cuts[j].push(u);
                crossings[i].push((t, sources[j].id, sources[j].piece));
                crossings[j].push((u, sources[i].id, sources[i].piece));
            }
        }
    }
    let mut vertices: Vec<[f64; 2]> = vec![];
    let mut edges: Vec<(usize, usize, Edge)> = vec![];
    for (source_index, (s, cut)) in sources.iter().zip(&mut cuts).enumerate() {
        cut.sort_by(f64::total_cmp);
        cut.dedup_by(|a, b| (*a - *b).abs() < 1e-7);
        let pieces = cut.len() - 1;
        for (part, t) in cut.windows(2).enumerate() {
            let start = s.curve.point(t[0]);
            let end = s.curve.point(t[1]);
            let vertex = |p, vertices: &mut Vec<[f64; 2]>| {
                if let Some(i) = vertices.iter().position(|q| distance(p, *q) < 1e-8) {
                    i
                } else {
                    vertices.push(p);
                    vertices.len() - 1
                }
            };
            let a = vertex(start, &mut vertices);
            let b = vertex(end, &mut vertices);
            if distance(start, end) < 1e-10 && t[1] - t[0] < 0.5 {
                continue;
            }
            let (center, sweep) = match &s.curve {
                Curve::Circle { center, sweep, .. } => (*center, sweep * (t[1] - t[0])),
                _ => ([0.; 2], 0.),
            };
            // Shared line boundaries occur once in the planar graph.
            if matches!(s.curve, Curve::Line(..))
                && edges.iter().any(|(x, y, e)| {
                    matches!(e.curve, Curve::Line(..))
                        && ((*x == a && *y == b) || (*x == b && *y == a))
                })
            {
                continue;
            }
            if edges.iter().any(|(x, y, e)| {
                let same = *x == a && *y == b;
                let reversed = *x == b && *y == a;
                (same || reversed)
                    && [0.25, 0.5, 0.75].iter().all(|fraction| {
                        distance(
                            s.curve.point(t[0] + (t[1] - t[0]) * fraction),
                            e.curve.point(
                                e.from
                                    + (e.to - e.from)
                                        * if same { *fraction } else { 1. - fraction },
                            ),
                        ) < 1e-9
                    })
            }) {
                continue;
            }
            edges.push((
                a,
                b,
                Edge {
                    id: s.id,
                    start,
                    end,
                    center,
                    sweep,
                    curve: s.curve.clone(),
                    from: t[0],
                    to: t[1],
                    token: token(s, part, pieces, &crossings[source_index], t),
                },
            ));
        }
    }
    let mut outgoing = vec![vec![]; vertices.len()];
    for (i, (a, b, _)) in edges.iter().enumerate() {
        outgoing[*a].push(2 * i);
        outgoing[*b].push(2 * i + 1);
    }
    let angle = |h: usize| {
        let e = &edges[h / 2].2;
        let (t, sign) = if h % 2 == 0 {
            (e.from, 1.)
        } else {
            (e.to, -1.)
        };
        let p = e.curve.point(t);
        let q = e
            .curve
            .point((t + sign * 1e-5).clamp(e.from.min(e.to), e.from.max(e.to)));
        let v = [q[0] - p[0], q[1] - p[1]];
        v[1].atan2(v[0])
    };
    for list in &mut outgoing {
        list.sort_by(|a, b| angle(*a).total_cmp(&angle(*b)));
    }
    let mut used = vec![false; edges.len() * 2];
    let mut loops: Vec<Vec<Edge>> = vec![];
    let mut exteriors: Vec<Vec<Edge>> = vec![];
    for first in 0..used.len() {
        if used[first] {
            continue;
        }
        let mut h = first;
        let mut walk = vec![];
        loop {
            if used[h] {
                break;
            }
            used[h] = true;
            walk.push(h);
            let (a, b, _) = &edges[h / 2];
            let end = if h % 2 == 0 { *b } else { *a };
            let list = &outgoing[end];
            let index = list.iter().position(|x| *x == (h ^ 1)).unwrap();
            h = list[(index + list.len() - 1) % list.len()];
            if h == first {
                break;
            }
        }
        // Dangling edges are walked twice and cannot bound a face.
        let mut wire = vec![];
        for &half in &walk {
            if walk.contains(&(half ^ 1)) {
                continue;
            }
            let mut e = edges[half / 2].2.clone();
            if half % 2 == 1 {
                e.reverse();
            }
            wire.push(e);
        }
        // Removing a bridge can leave disconnected cycles in the same face walk
        // (for example a line joining an outer contour to a hole).
        let mut cycles: Vec<Vec<Edge>> = vec![];
        for edge in wire {
            if cycles
                .last()
                .and_then(|cycle| cycle.last())
                .is_none_or(|last| distance(last.end, edge.start) > 1e-8)
            {
                cycles.push(vec![]);
            }
            cycles.last_mut().unwrap().push(edge);
        }
        if cycles.len() > 1
            && distance(
                cycles.last().unwrap().last().unwrap().end,
                cycles[0][0].start,
            ) < 1e-8
        {
            let mut last = cycles.pop().unwrap();
            last.append(&mut cycles[0]);
            cycles[0] = last;
        }
        for wire in cycles {
            if distance(wire[0].start, wire.last().unwrap().end) > 1e-8 {
                continue;
            }
            let area = area(&wire);
            if area > 1e-12 {
                loops.push(wire);
            } else if area < -1e-12 {
                exteriors.push(wire);
            }
        }
    }
    if loops.is_empty() {
        return Err("No closed sketch region; connect the boundary endpoints".into());
    }
    let polygons: Vec<_> = loops.iter().map(|w| polygon(w)).collect();
    let areas: Vec<_> = loops.iter().map(|w| area(w)).collect();
    let outside_polygons: Vec<_> = exteriors.iter().map(|wire| polygon(wire)).collect();
    let outside_areas: Vec<_> = exteriors.iter().map(|wire| -area(wire)).collect();
    let mut result = vec![];
    for i in 0..loops.len() {
        let mut wires = vec![loops[i].clone()];
        let children: Vec<_> = (0..exteriors.len())
            .filter(|&j| {
                outside_areas[j] < areas[i] - 1e-12
                    && outside_polygons[j]
                        .iter()
                        .all(|p| contains(&polygons[i], *p))
            })
            .collect();
        for &j in &children {
            if !children.iter().any(|&other| {
                other != j
                    && outside_areas[other] > outside_areas[j] + 1e-12
                    && outside_polygons[j]
                        .iter()
                        .all(|p| contains(&outside_polygons[other], *p))
            }) {
                wires.push(exteriors[j].clone());
            }
        }
        let mut boundary: Vec<_> = loops[i].iter().map(|e| e.token).collect();
        boundary.sort();
        boundary.dedup();
        let mut curves: Vec<_> = loops[i].iter().map(|e| e.id).collect();
        curves.sort();
        curves.dedup();
        result.push(Region {
            nested: (0..loops.len()).any(|j| {
                j != i
                    && areas[j] > areas[i] + 1e-12
                    && polygons[i].iter().all(|p| contains(&polygons[j], *p))
            }),
            boundary,
            curves,
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
    let outer: Vec<_> = all.iter().filter(|r| !r.nested).collect();
    if outer.len() != 1 {
        return Err("Multiple regions: click inside the region to extrude".into());
    }
    Ok(outer[0].clone())
}
#[cfg(feature = "kernel")]
pub fn extrude(region: &Region, depth: f64) -> Result<crate::kernel::bridge::ffi::Mesh, String> {
    let edges: Vec<_> = region
        .wires
        .iter()
        .enumerate()
        .flat_map(|(wire, edges)| edges.iter().map(move |e| e.native(wire as u32)))
        .collect();
    crate::kernel::bridge::ffi::extrude_region(&edges, depth).map_err(|e| e.to_string())
}

pub fn is_boundary_token(id: &Uuid) -> bool {
    id.as_bytes()[6] >> 4 == 8 && id.as_bytes()[8] >> 6 == 2
}
