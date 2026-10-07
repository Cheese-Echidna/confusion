use confusion::{
    document::{
        model::{CapRole, ExtrudeFeature, ExtrudeOperation, SketchPlane},
        schema::{Design, Extrusion},
    },
    evaluation::{cache::EvaluationCache, solid},
};
fn design(pockets: usize) -> Design {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.08, 0.05]);
    let depth = d.parameter("thickness", "10 mm".into());
    let base = uuid::Uuid::new_v4();
    d.extrusion = Some(Extrusion {
        id: base,
        depth,
        boundary: vec![],
    });
    d.sync_construction();
    let mut target = base;
    for n in 0..pockets {
        let sketch = d
            .create_sketch(SketchPlane::Face {
                support: target,
                producer: base,
                role: CapRole::End,
            })
            .unwrap();
        let x = 0.004 + (n % 8) as f64 * 0.009;
        let y = 0.004 + (n / 8) as f64 * 0.012;
        d.rectangle([x, y], [x + 0.005, y + 0.007]);
        let boundary = confusion::sketch::regions::select(
            &d,
            &d.points.iter().map(|p| p.xy).collect::<Vec<_>>(),
            &[],
        )
        .unwrap()
        .boundary;
        let depth = d.parameter(&format!("pocket{n}"), "3 mm".into());
        let id = uuid::Uuid::new_v4();
        d.features.push(ExtrudeFeature {
            id,
            name: format!("Pocket {n}"),
            sketch,
            boundary,
            depth,
            operation: ExtrudeOperation::Cut,
            target: Some(target),
        });
        d.sync_construction();
        target = id;
    }
    d
}
fn main() -> Result<(), String> {
    let mut d = design(24);
    let mut cache = EvaluationCache::default();
    solid::evaluate_cached(&d, || false, &mut cache)?;
    for (name, parameter) in [("last pocket", "pocket23"), ("base plate", "thickness")] {
        let mut full = Vec::new();
        let mut incremental = Vec::new();
        for n in 0..5 {
            d.parameters
                .iter_mut()
                .find(|p| p.name == parameter)
                .unwrap()
                .expression = if parameter == "thickness" {
                format!("{} mm", 11 + n % 2)
            } else {
                format!("{} mm", 4 + n % 2)
            };
            let start = std::time::Instant::now();
            let cached = solid::evaluate_cached(&d, || false, &mut cache)?;
            incremental.push(start.elapsed().as_secs_f64() * 1000.);
            let start = std::time::Instant::now();
            let fresh = solid::evaluate(&d, || false)?;
            full.push(start.elapsed().as_secs_f64() * 1000.);
            assert!((cached.mesh.unwrap().volume - fresh.mesh.unwrap().volume).abs() < 1e-12);
        }
        full.sort_by(f64::total_cmp);
        incremental.sort_by(f64::total_cmp);
        println!(
            "{name}: full={:.1}ms cached={:.1}ms reused_shapes={} reused_sketches={}",
            full[2],
            incremental[2],
            cache.reused_features(),
            cache.reused_sketches
        );
    }
    let mut repeated = Vec::new();
    for _ in 0..5 {
        let start = std::time::Instant::now();
        solid::evaluate_cached(&d, || false, &mut cache)?;
        repeated.push(start.elapsed().as_secs_f64() * 1000.);
    }
    repeated.sort_by(f64::total_cmp);
    println!(
        "unchanged model: cached={:.2}ms reused_shapes={} reused_sketches={}",
        repeated[2],
        cache.reused_features(),
        cache.reused_sketches
    );
    Ok(())
}
