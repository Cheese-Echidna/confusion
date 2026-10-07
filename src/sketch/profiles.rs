//! Extract one simple closed line profile from solved sketch coordinates.
//! Exports closed_profile; evaluation uses this before exact extrusion. Shared point
//! IDs provide coincident topology. Open, branching, disconnected and crossing wires fail.
use crate::document::schema::Design;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
pub fn closed_profile(d: &Design, xy: &[[f64; 2]]) -> Result<Vec<[f64; 2]>, String> {
    if d.lines.len() < 3 {
        return Err("Draw a closed profile with at least three lines".into());
    }
    let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for l in &d.lines {
        adjacency.entry(l.ends[0]).or_default().push(l.ends[1]);
        adjacency.entry(l.ends[1]).or_default().push(l.ends[0]);
    }
    if adjacency.values().any(|v| v.len() != 2) {
        return Err("Profile is open or branches; connect every endpoint".into());
    }
    let start = d.lines[0].ends[0];
    let mut current = start;
    let mut previous = None;
    let mut visited = HashSet::new();
    let mut polygon = Vec::new();
    loop {
        if !visited.insert(current) {
            return Err("Profile repeats a vertex".into());
        }
        polygon.push(
            xy[d.points
                .iter()
                .position(|p| p.id == current)
                .ok_or("Missing profile point")?],
        );
        let neighbors = &adjacency[&current];
        let next = if Some(neighbors[0]) == previous {
            neighbors[1]
        } else {
            neighbors[0]
        };
        if next == start {
            break;
        }
        previous = Some(current);
        current = next;
    }
    if visited.len() != adjacency.len() {
        return Err("Multiple profiles and holes are not supported by this extrusion yet".into());
    }
    let cross = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| {
        (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
    };
    let n = polygon.len();
    for i in 0..n {
        let a = polygon[i];
        let b = polygon[(i + 1) % n];
        if (a[0] - b[0]).hypot(a[1] - b[1]) < 1e-7 {
            return Err("Profile has a zero-length edge".into());
        }
        for j in i + 1..n {
            if j == i + 1 || (i == 0 && j == n - 1) {
                continue;
            }
            let c = polygon[j];
            let e = polygon[(j + 1) % n];
            let c1 = cross(a, b, c);
            let c2 = cross(a, b, e);
            let c3 = cross(c, e, a);
            let c4 = cross(c, e, b);
            if c1 * c2 <= 0.
                && c3 * c4 <= 0.
                && a[0].min(b[0]) <= c[0].max(e[0]) + 1e-12
                && c[0].min(e[0]) <= a[0].max(b[0]) + 1e-12
                && a[1].min(b[1]) <= c[1].max(e[1]) + 1e-12
                && c[1].min(e[1]) <= a[1].max(b[1]) + 1e-12
            {
                return Err("Profile intersects itself".into());
            }
        }
    }
    let area: f64 = (0..n)
        .map(|i| polygon[i][0] * polygon[(i + 1) % n][1] - polygon[(i + 1) % n][0] * polygon[i][1])
        .sum();
    if area.abs() < 1e-12 {
        return Err("Profile has no area".into());
    }
    if area < 0. {
        polygon.reverse();
    }
    Ok(polygon)
}
