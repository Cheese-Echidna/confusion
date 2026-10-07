use confusion::{
    document::schema::{Design, Parameter},
    parameters::expression,
};
use uuid::Uuid;

fn evaluate(source: &str, scalar: bool, angular: bool) -> Result<f64, String> {
    let mut design = Design::default();
    let id = Uuid::new_v4();
    design.parameters.push(Parameter {
        id,
        name: "result".into(),
        expression: source.into(),
        scalar,
        angular,
    });
    expression::evaluate(&design).map(|values| values[&id])
}

#[test]
fn invalid_intermediates_cannot_be_masked_by_functions_or_powers() {
    for source in [
        "min(0 / 0, 1)",
        "max(0 / 0, 1)",
        "(1 / 0) ^ 0",
        "min(asin(2), 1 rad) / 1 rad",
        "max(acos(-2), 1 rad) / 1 rad",
        "sqrt(-1)",
        "0 ^ -1",
        "1 / -0",
    ] {
        assert!(evaluate(source, true, false).is_err(), "accepted {source}");
    }
    let huge = "9".repeat(400);
    for source in [format!("min({huge}, 1)"), format!("({huge} mm) ^ 0")] {
        assert!(evaluate(&source, true, false).is_err(), "accepted {source}");
    }
    // Finite literals can still overflow at every arithmetic operation.
    let large = "9".repeat(308);
    for source in [
        format!("min({large} + {large}, 1)"),
        format!("min({large} * 2, 1)"),
        format!("({large} ^ 2) ^ 0"),
    ] {
        assert!(evaluate(&source, true, false).is_err(), "accepted {source}");
    }
}

#[test]
fn dimension_overflow_returns_an_error_without_panicking() {
    // Numerically this is always one; only the length exponent grows.
    let mut source = "1 m".to_string();
    for _ in 0..8 {
        source = format!("({source}) ^ 16");
    }
    assert!(
        evaluate(&format!("({source}) ^ 0"), true, false)
            .unwrap_err()
            .contains("dimension overflow")
    );
    let mut near_limit = "1 m".to_string();
    for _ in 0..7 {
        near_limit = format!("({near_limit}) ^ 16");
    }
    near_limit = format!("({near_limit}) ^ 7");
    for operation in ["*", "/"] {
        let rhs = if operation == "/" {
            format!("({near_limit}) ^ -1")
        } else {
            near_limit.clone()
        };
        let expression = format!("(({near_limit}) {operation} ({rhs})) ^ 0");
        assert!(
            evaluate(&expression, true, false)
                .unwrap_err()
                .contains("dimension overflow")
        );
    }
}

#[test]
fn valid_dimensional_and_scalar_expressions_still_work() {
    for (source, scalar, angular, expected) in [
        ("min(2 + 3, 7) * 2 / 4", true, false, 2.5),
        ("sqrt((3 mm) ^ 2) + 2 cm", false, false, 0.023),
        ("sin(90 deg) + cos(0 rad)", true, false, 2.0),
        ("asin(1)", false, true, std::f64::consts::FRAC_PI_2),
        ("2 ^ -3", true, false, 0.125),
        ("abs(-2 in)", false, false, 0.0508),
    ] {
        let value = evaluate(source, scalar, angular).unwrap();
        assert!(
            (value - expected).abs() < 1e-12,
            "incorrect {source}: {value}"
        );
    }
}
