//! Damped least-squares planar solve with parallel residual rows and rank-based DOF.
//! Exports solve/Solution; consumes immutable Design and resolved SI parameters.
//! UI receives owned solutions through runtime worker; no document mutation occurs here.
#[cfg(feature = "solver")]
mod implementation {
    use crate::document::schema::{ConstraintKind, Design};
    use faer::{Mat, linalg::solvers::Solve};
    use rayon::prelude::*;
    use std::collections::HashMap;
    use uuid::Uuid;
    #[derive(Clone, Debug)]
    pub struct Solution {
        pub points: Vec<[f64; 2]>,
        pub dof: usize,
        pub conflicts: Vec<Uuid>,
        pub residual: f64,
    }
    struct Row {
        id: Uuid,
        value: f64,
        jac: Vec<f64>,
    }
    fn rows(d: &Design, x: &[f64], p: &HashMap<Uuid, f64>) -> Vec<Row> {
        let point = |id: Uuid| d.points.iter().position(|p| p.id == id).unwrap() * 2;
        let ends = |id: Uuid| d.lines.iter().find(|l| l.id == id).unwrap().ends.map(point);
        d.constraints
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
                }
                result
            })
            .collect()
    }
    pub fn solve(d: &Design, p: &HashMap<Uuid, f64>) -> Result<Solution, String> {
        solve_cancellable(d, p, || false)
    }
    pub fn solve_cancellable(
        d: &Design,
        p: &HashMap<Uuid, f64>,
        cancelled: impl Fn() -> bool,
    ) -> Result<Solution, String> {
        d.validate()?;
        if d.parameters
            .iter()
            .any(|parameter| p.get(&parameter.id).is_none_or(|v| !v.is_finite()))
        {
            return Err("Missing or invalid resolved parameter".into());
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
            });
        }
        let mut damping = 1e-4;
        for _ in 0..80 {
            if cancelled() {
                return Err("Evaluation superseded".into());
            }
            let r = rows(d, &x, p);
            if r.is_empty() {
                break;
            }
            let error: f64 = r.iter().map(|r| r.value * r.value).sum();
            if error < 1e-14 {
                break;
            }
            let a = Mat::from_fn(n, n, |i, j| {
                r.iter().map(|r| r.jac[i] * r.jac[j]).sum::<f64>()
                    + if i == j { damping } else { 0. }
            });
            let b = Mat::from_fn(n, 1, |i, _| {
                -r.iter().map(|r| r.jac[i] * r.value).sum::<f64>()
            });
            let step = a.col_piv_qr().solve(&b);
            let next: Vec<_> = x
                .iter()
                .enumerate()
                .map(|(i, v)| v + step[(i, 0)])
                .collect();
            let next_error: f64 = rows(d, &next, p).iter().map(|r| r.value * r.value).sum();
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
        let rank = if r.is_empty() {
            0
        } else {
            let j = Mat::from_fn(r.len(), n, |i, k| r[i].jac[k]);
            let singular = j
                .singular_values()
                .map_err(|_| "Could not determine sketch rank")?;
            singular.iter().filter(|v| **v > 1e-7).count()
        };
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
            conflicts,
            residual: r.iter().map(|r| r.value.abs() * 0.001).fold(0., f64::max),
        })
    }
}
#[cfg(feature = "solver")]
pub use implementation::*;
