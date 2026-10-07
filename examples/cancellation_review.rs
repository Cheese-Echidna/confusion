//! Measure native cancellation and latest-edit delivery on the 24-pocket fixture.
use confusion::{
    document::{
        model::{CapRole, ExtrudeFeature, ExtrudeOperation, SketchPlane},
        schema::{Design, Extrusion},
    },
    evaluation::{cache::EvaluationCache, solid},
    kernel::cancellation::EvaluationCancellation,
    runtime::worker::Worker,
};
use std::time::{Duration, Instant};
fn median(mut samples: Vec<f64>) -> f64 {
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}
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
    let d = design(24);
    let start = Instant::now();
    let full = solid::evaluate(&d, || false)?;
    println!(
        "uncached full rebuild={:.1}ms",
        start.elapsed().as_secs_f64() * 1000.
    );
    let expected = full.mesh.unwrap().volume;
    let mut cancellation = Vec::new();
    let mut delivery = Vec::new();
    for _ in 0..5 {
        let mut cache = EvaluationCache::default();
        let token = EvaluationCancellation::default();
        let other = token.clone();
        let cancel = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            let when = Instant::now();
            other.cancel();
            when
        });
        let result = solid::evaluate_cancellable(&d, &token, &mut cache);
        let completed = Instant::now();
        let requested = cancel.join().unwrap();
        let error = result
            .err()
            .ok_or("Fixture finished before cancellation was requested")?;
        if !error.contains("Evaluation superseded") {
            return Err(error);
        }
        cancellation.push(completed.saturating_duration_since(requested).as_secs_f64() * 1000.);
        let rebuilt =
            solid::evaluate_cancellable(&d, &EvaluationCancellation::default(), &mut cache)?;
        assert!((rebuilt.mesh.unwrap().volume - expected).abs() < 1e-12);
        let worker = Worker::new();
        worker.submit(1, d.clone());
        std::thread::sleep(Duration::from_millis(50));
        let start = Instant::now();
        worker.submit(2, design(0));
        loop {
            if let Some(result) = worker.poll() {
                assert_eq!(result.revision, 2);
                assert_eq!(result.features.len(), 1);
                let mesh = result.mesh?.ok_or("Missing replacement mesh")?;
                assert!((mesh.volume - 0.08 * 0.05 * 0.01).abs() < 1e-12);
                break;
            }
            if start.elapsed() > Duration::from_secs(5) {
                return Err("Worker timed out".into());
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        delivery.push(start.elapsed().as_secs_f64() * 1000.);
    }
    println!(
        "native cancellation after 50ms: median response={:.2}ms",
        median(cancellation)
    );
    println!(
        "new plate edit replacing active 24-pocket rebuild: median delivery={:.2}ms",
        median(delivery)
    );
    Ok(())
}
