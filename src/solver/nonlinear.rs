//! Damped least-squares planar solve with parallel residual rows and rank-based DOF.
//! Exports solve/Solution; consumes immutable Design and resolved SI parameters.
//! UI receives owned solutions through runtime worker; no document mutation occurs here.
#[cfg(feature = "solver")]
mod implementation {
    use crate::document::schema::{ConstraintKind, Design};
    use faer::{Mat, linalg::solvers::SolveLstsq};
    use rayon::prelude::*;
    use std::collections::HashMap;
    use uuid::Uuid;
    #[derive(Clone, Debug)]
    pub struct Solution {
        pub points: Vec<[f64; 2]>,
        pub dof: usize,
        pub conflicts: Vec<Uuid>,
        pub residual: f64,
        pub point_dof: Vec<usize>,
        pub redundant: Vec<Uuid>,
    }
    struct Row {
        id: Uuid,
        value: f64,
        jac: Vec<f64>,
    }
    fn rows(d: &Design, x: &[f64], p: &HashMap<Uuid, f64>) -> Vec<Row> {
        let point = |id: Uuid| d.points.iter().position(|p| p.id == id).unwrap() * 2;
        let ends = |id: Uuid| d.lines.iter().find(|l| l.id == id).unwrap().ends.map(point);
        let mut result: Vec<Row> = d
            .constraints
            .par_iter()
            .flat_map_iter(|c| {
                let mut result = Vec::new();
                let mut row = |value: f64, entries: &[(usize, f64)]| {
                    let mut jac = vec![0.; x.len()];
                    for &(i, v) in entries {
                        jac[i] += v;
                    }
                    result.push(Row {
                        id: c.id,
                        value,
                        jac,
                    });
                };
                match &c.kind {
                    ConstraintKind::Horizontal { line } => {
                        let [a, b] = ends(*line);
                        row(x[b + 1] - x[a + 1], &[(b + 1, 1.), (a + 1, -1.)]);
                    }
                    ConstraintKind::Vertical { line } => {
                        let [a, b] = ends(*line);
                        row(x[b] - x[a], &[(b, 1.), (a, -1.)]);
                    }
                    ConstraintKind::Fixed { point: id, xy } => {
                        let a = point(*id);
                        for k in 0..2 {
                            row(x[a + k] - xy[k] * 1000., &[(a + k, 1.)]);
                        }
                    }
                    ConstraintKind::DistanceX { points, parameter }
                    | ConstraintKind::DistanceY { points, parameter } => {
                        let k = usize::from(matches!(&c.kind, ConstraintKind::DistanceY { .. }));
                        let [a, b] = points.map(point);
                        row(
                            x[b + k] - x[a + k] - p[parameter] * 1000.,
                            &[(b + k, 1.), (a + k, -1.)],
                        );
                    }
                    ConstraintKind::Length { line, parameter } => {
                        let [a, b] = ends(*line);
                        let dx = x[b] - x[a];
                        let dy = x[b + 1] - x[a + 1];
                        let len = dx.hypot(dy);
                        let denom = len.max(1e-12);
                        row(
                            len - p[parameter] * 1000.,
                            &[
                                (a, -dx / denom),
                                (a + 1, -dy / denom),
                                (b, dx / denom),
                                (b + 1, dy / denom),
                            ],
                        );
                    }
                    _ => {
                        let values = crate::solver::problem::residual(d, &c.kind, x, p);
                        let mut jac = vec![vec![0.; x.len()]; values.len()];
                        for i in 0..x.len() {
                            let h = 1e-5;
                            let mut next = x.to_vec();
                            next[i] += h;
                            let plus = crate::solver::problem::residual(d, &c.kind, &next, p);
                            next[i] -= 2. * h;
                            let minus = crate::solver::problem::residual(d, &c.kind, &next, p);
                            for k in 0..values.len() {
                                jac[k][i] = (plus[k] - minus[k]) / (2. * h);
                            }
                        }
                        for (value, jac) in values.into_iter().zip(jac) {
                            result.push(Row {
                                id: c.id,
                                value,
                                jac,
                            });
                        }
                    }
                }
                result
            })
            .collect();
        for c in &d.circles {
            if let Some(end) = c.end {
                let kind = ConstraintKind::PointOnCircle {
                    point: end,
                    circle: c.id,
                };
                let value = crate::solver::problem::residual(d, &kind, x, p)[0];
                let mut jac = vec![0.; x.len()];
                for i in 0..x.len() {
                    let mut next = x.to_vec();
                    next[i] += 1e-5;
                    let plus = crate::solver::problem::residual(d, &kind, &next, p)[0];
                    next[i] -= 2e-5;
                    let minus = crate::solver::problem::residual(d, &kind, &next, p)[0];
                    jac[i] = (plus - minus) / 2e-5;
                }
                result.push(Row {
                    id: c.id,
                    value,
                    jac,
                });
            } else {
                let a = point(c.center);
                let b = point(c.rim);
                let seed = d.points[b / 2].xy;
                let center = d.points[a / 2].xy;
                let v = [seed[0] - center[0], seed[1] - center[1]];
                let len = v[0].hypot(v[1]).max(1e-12);
                let mut jac = vec![0.; x.len()];
                jac[b] = v[1] / len;
                jac[b + 1] = -v[0] / len;
                jac[a] = -v[1] / len;
                jac[a + 1] = v[0] / len;
                result.push(Row {
                    id: c.id,
                    value: (x[b] - x[a]) * v[1] / len - (x[b + 1] - x[a + 1]) * v[0] / len,
                    jac,
                });
            }
        }
        for e in &d.ellipses {
            let a = point(e.center);
            let b = point(e.major);
            let c = point(e.minor);
            let v = [x[b] - x[a], x[b + 1] - x[a + 1]];
            let w = [x[c] - x[a], x[c + 1] - x[a + 1]];
            let len = v[0].hypot(v[1]).max(1e-8);
            let mut jac = vec![0.; x.len()];
            // Axis perpendicularity is intrinsic to the ellipse representation.
            let value = (v[0] * w[0] + v[1] * w[1]) / len;
            for k in 0..2 {
                jac[b + k] = w[k] / len - value * v[k] / (len * len);
                jac[c + k] = v[k] / len;
                jac[a + k] = -jac[b + k] - jac[c + k];
            }
            result.push(Row {
                id: e.id,
                value,
                jac,
            });
        }
        result
    }
    pub fn solve(d: &Design, p: &HashMap<Uuid, f64>) -> Result<Solution, String> {
        solve_cancellable(d, p, || false)
    }
    pub fn solve_cancellable(
        d: &Design,
        p: &HashMap<Uuid, f64>,
        cancelled: impl Fn() -> bool,
    ) -> Result<Solution, String> {
        solve_internal(d, p, cancelled, &[])
    }
    /// Temporary soft targets influence only the preview solution, never the design constraints.
    pub fn solve_drag(
        d: &Design,
        p: &HashMap<Uuid, f64>,
        targets: &[(Uuid, [f64; 2])],
    ) -> Result<Solution, String> {
        if targets.iter().any(|(id, xy)| {
            !d.points.iter().any(|p| p.id == *id) || xy.iter().any(|v| !v.is_finite())
        }) {
            return Err("Invalid drag target".into());
        }
        solve_internal(d, p, || false, targets)
    }
    fn solve_internal(
        d: &Design,
        p: &HashMap<Uuid, f64>,
        cancelled: impl Fn() -> bool,
        targets: &[(Uuid, [f64; 2])],
    ) -> Result<Solution, String> {
        d.validate()?;
        if d.parameters
            .iter()
            .any(|parameter| p.get(&parameter.id).is_none_or(|v| !v.is_finite()))
        {
            return Err("Missing or invalid resolved parameter".into());
        }
        for c in &d.constraints {
            if matches!(
                c.kind,
                ConstraintKind::Length { .. }
                    | ConstraintKind::Radius { .. }
                    | ConstraintKind::Diameter { .. }
            ) && let Some(id) = c.kind.parameter()
                && p[&id] <= 1e-7
            {
                return Err("Length, radius and diameter dimensions must be positive".into());
            }
        }
        let mut x: Vec<f64> = d
            .points
            .iter()
            .flat_map(|p| p.xy.map(|v| v * 1000.))
            .collect();
        let n = x.len();
        if n == 0 {
            return Ok(Solution {
                points: vec![],
                dof: 0,
                conflicts: vec![],
                residual: 0.,
                point_dof: vec![],
                redundant: vec![],
            });
        }
        let evaluate = |x: &[f64]| {
            let mut result = rows(d, x, p);
            for (id, target) in targets {
                let i = d.points.iter().position(|p| p.id == *id).unwrap() * 2;
                for k in 0..2 {
                    let mut jac = vec![0.; x.len()];
                    jac[i + k] = 1e-4;
                    result.push(Row {
                        id: Uuid::nil(),
                        value: (x[i + k] - target[k] * 1000.) * 1e-4,
                        jac,
                    });
                }
            }
            result
        };
        let mut damping: f64 = 1e-4;
        for _ in 0..80 {
            if cancelled() {
                return Err("Evaluation superseded".into());
            }
            let r = evaluate(&x);
            if r.is_empty() {
                break;
            }
            let error: f64 = r.iter().map(|r| r.value * r.value).sum();
            if error < 1e-14 {
                break;
            }
            // Augmented QR avoids squaring the condition number via normal equations.
            let a = Mat::from_fn(r.len() + n, n, |i, j| {
                if i < r.len() {
                    r[i].jac[j]
                } else if i - r.len() == j {
                    damping.sqrt()
                } else {
                    0.
                }
            });
            let b = Mat::from_fn(
                r.len() + n,
                1,
                |i, _| if i < r.len() { -r[i].value } else { 0. },
            );
            let step = a.col_piv_qr().solve_lstsq(&b);
            let next: Vec<_> = x
                .iter()
                .enumerate()
                .map(|(i, v)| v + step[(i, 0)])
                .collect();
            let next_error: f64 = evaluate(&next).iter().map(|r| r.value * r.value).sum();
            if next_error < error {
                x = next;
                damping = (damping * 0.25).max(1e-12);
            } else {
                damping = (damping * 10.).min(1e12);
            }
        }
        if x.iter().any(|v| !v.is_finite() || v.abs() > 1e6) {
            return Err("Solver diverged outside supported coordinate range".into());
        }
        let r = rows(d, &x, p);
        let mut freedom = vec![true; n];
        let rank = if r.is_empty() {
            0
        } else {
            let j = Mat::from_fn(r.len(), n, |i, k| r[i].jac[k]);
            let svd = j.svd().map_err(|_| "Could not determine sketch rank")?;
            let values = j
                .singular_values()
                .map_err(|_| "Could not determine sketch rank")?;
            let threshold = values.first().copied().unwrap_or(1.).max(1.) * 1e-8;
            let rank = values.iter().filter(|v| **v > threshold).count();
            for (i, free) in freedom.iter_mut().enumerate() {
                *free = (rank..n).map(|k| svd.V()[(i, k)].powi(2)).sum::<f64>() > 1e-8;
            }
            rank
        };
        let point_dof = freedom
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| usize::from(p[0]) + usize::from(p[1]))
            .collect();
        let mut redundant = Vec::new();
        let mut basis: Vec<Vec<f64>> = Vec::new();
        // Reorthogonalized Jacobian rows identify locally dependent constraint groups.
        // These diagnostics concern local rank, not a mathematically minimal conflict set.
        let mut ordered: Vec<_> = r
            .iter()
            .filter(|row| !d.constraints.iter().any(|c| c.id == row.id))
            .collect();
        ordered.extend(
            r.iter()
                .filter(|row| d.constraints.iter().any(|c| c.id == row.id)),
        );
        let mut independent = std::collections::HashSet::new();
        for row in ordered {
            let mut v = row.jac.clone();
            let norm = v.iter().map(|v| v * v).sum::<f64>().sqrt();
            for _ in 0..2 {
                for q in &basis {
                    let projection = v.iter().zip(q).map(|(a, b)| a * b).sum::<f64>();
                    for (v, q) in v.iter_mut().zip(q) {
                        *v -= projection * q;
                    }
                }
            }
            let remainder = v.iter().map(|v| v * v).sum::<f64>().sqrt();
            if remainder > norm.max(1.) * 1e-8 {
                for v in &mut v {
                    *v /= remainder
                }
                basis.push(v);
                independent.insert(row.id);
            }
        }
        for c in &d.constraints {
            if !independent.contains(&c.id) {
                redundant.push(c.id)
            }
        }
        let mut conflicts: Vec<_> = r
            .iter()
            .filter(|r| r.value.abs() > 1e-5)
            .map(|r| r.id)
            .collect();
        conflicts.sort();
        conflicts.dedup();
        Ok(Solution {
            points: x
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| [p[0] * 0.001, p[1] * 0.001])
                .collect(),
            dof: n - rank,
            point_dof,
            redundant,
            conflicts,
            residual: r.iter().map(|r| r.value.abs() * 0.001).fold(0., f64::max),
        })
    }
}
#[cfg(feature = "solver")]
pub use implementation::*;
