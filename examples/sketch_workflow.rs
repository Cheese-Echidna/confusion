//! Runnable acceptance sketch: a fully constrained plate, four holes and a center slot.
use confusion::{
    document::schema::{ConstraintKind as C, Design},
    parameters::expression,
    persistence::container,
    sketch::edit,
    solver::nonlinear,
};
use uuid::Uuid;
fn dimension(d: &mut Design, kind: C, position: [f64; 2]) {
    let id = Uuid::new_v4();
    d.constraints
        .push(confusion::document::schema::Constraint { id, kind });
    d.dimension_positions.insert(id, position);
}
pub fn plate() -> Design {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.08, 0.05]);
    let origin = d.points[0].id;
    let diameter = d.parameter("holeDiameter", "6 mm".into());
    let margin = d.parameter("margin", "10 mm".into());
    let right = d.parameter("rightHoleX", "width - margin".into());
    let top = d.parameter("topHoleY", "height - margin".into());
    for (i, (xy, x, y)) in [
        ([0.01, 0.01], margin, margin),
        ([0.07, 0.01], right, margin),
        ([0.07, 0.04], right, top),
        ([0.01, 0.04], margin, top),
    ]
    .into_iter()
    .enumerate()
    {
        let circle = edit::circle(&mut d, xy, [xy[0] + 0.003, xy[1]], None).unwrap();
        let center = d.circles.last().unwrap().center;
        dimension(
            &mut d,
            C::Diameter {
                circle,
                parameter: diameter,
            },
            [xy[0] + 0.004, xy[1] + 0.004],
        );
        dimension(
            &mut d,
            C::DistanceX {
                points: [origin, center],
                parameter: x,
            },
            [xy[0] * 0.5, -0.006 - i as f64 * 0.004],
        );
        dimension(
            &mut d,
            C::DistanceY {
                points: [origin, center],
                parameter: y,
            },
            [-0.006 - i as f64 * 0.004, xy[1] * 0.5],
        );
    }
    let start = d.parameter("slotStart", "30 mm".into());
    let end = d.parameter("slotEnd", "width - slotStart".into());
    let center_y = d.parameter("centerY", "height / 2".into());
    let radius = d.parameter("slotRadius", "2 mm".into());
    let before = d.circles.len();
    edit::slot(&mut d, [0.03, 0.025], [0.05, 0.025], [0.05, 0.027]).unwrap();
    let left = d.circles[before].center;
    let right = d.circles[before + 1].center;
    let arc = d.circles[before].id;
    for (center, x) in [(left, start), (right, end)] {
        d.constrain(C::DistanceX {
            points: [origin, center],
            parameter: x,
        });
        d.constrain(C::DistanceY {
            points: [origin, center],
            parameter: center_y,
        });
    }
    dimension(
        &mut d,
        C::Radius {
            circle: arc,
            parameter: radius,
        },
        [0.035, 0.034],
    );
    for constraint in &d.constraints {
        if matches!(constraint.kind, C::DistanceX { .. })
            && constraint
                .kind
                .parameter()
                .is_some_and(|id| d.parameters.iter().any(|p| p.id == id && p.name == "width"))
        {
            d.dimension_positions.insert(constraint.id, [0.04, -0.003]);
        }
        if matches!(constraint.kind, C::DistanceY { .. })
            && constraint.kind.parameter().is_some_and(|id| {
                d.parameters
                    .iter()
                    .any(|p| p.id == id && p.name == "height")
            })
        {
            d.dimension_positions.insert(constraint.id, [-0.003, 0.025]);
        }
    }
    d.sync_construction();
    d
}
fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/confusion-complex-sketch.con".into());
    let mut d = plate();
    let solution = nonlinear::solve(&d, &expression::evaluate(&d).unwrap()).unwrap();
    assert!(solution.conflicts.is_empty());
    assert_eq!(solution.dof, 0);
    assert!(solution.point_dof.iter().all(|d| *d == 0));
    for (p, xy) in d.points.iter_mut().zip(solution.points) {
        p.xy = xy;
    }
    container::save(std::path::Path::new(&path), &d).unwrap();
    let mut reopened = container::load(std::path::Path::new(&path)).unwrap();
    reopened
        .parameters
        .iter_mut()
        .find(|p| p.name == "width")
        .unwrap()
        .expression = "100 mm".into();
    let solution = nonlinear::solve(&reopened, &expression::evaluate(&reopened).unwrap()).unwrap();
    assert!(solution.conflicts.is_empty());
    assert_eq!(solution.dof, 0);
    assert!(
        solution
            .points
            .iter()
            .any(|p| (p[0] - 0.09).abs() < 1e-8 && (p[1] - 0.04).abs() < 1e-8)
    );
    println!(
        "Saved fully constrained {}-point, {}-constraint sketch to {path}; width edit regenerates after reopen",
        d.points.len(),
        d.constraints.len()
    );
}
