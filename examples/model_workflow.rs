//! Rebuild the face-attached pocket acceptance document and verify exact regeneration.
use confusion::{
    document::{
        model::{CapRole, ExtrudeFeature, ExtrudeOperation, SketchPlane},
        schema::{Design, Extrusion},
    },
    evaluation::solid,
    persistence::container,
    sketch::regions,
};
fn main() {
    if std::env::args().nth(1).as_deref() == Some("--verify") {
        let path = std::env::args()
            .nth(2)
            .expect("--verify requires a .con path");
        let design = container::load(std::path::Path::new(&path)).unwrap();
        let evaluated = solid::evaluate(&design, || false).unwrap();
        let mesh = evaluated.mesh.expect("Design has no solid features");
        println!(
            "Validated {} solid features, {} faces, volume {:.3} mm³",
            evaluated.features.len(),
            mesh.faces,
            mesh.volume * 1e9
        );
        return;
    }
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/confusion-pocket.con".into());
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.08, 0.05]);
    let thickness = d.parameter("thickness", "10 mm".into());
    let base = uuid::Uuid::new_v4();
    d.extrusion = Some(Extrusion {
        id: base,
        depth: thickness,
        boundary: vec![],
    });
    d.sync_construction();
    let sketch = d
        .create_sketch(SketchPlane::Face {
            support: base,
            producer: base,
            role: CapRole::End,
        })
        .unwrap();
    d.rectangle([0.01, 0.01], [0.03, 0.02]);
    let boundary = regions::select(&d, &d.points.iter().map(|p| p.xy).collect::<Vec<_>>(), &[])
        .unwrap()
        .boundary;
    let depth = d.parameter("pocketDepth", "3 mm".into());
    d.features.push(ExtrudeFeature {
        id: uuid::Uuid::new_v4(),
        name: "Pocket".into(),
        sketch,
        boundary,
        depth,
        operation: ExtrudeOperation::Cut,
        target: Some(base),
    });
    d.sync_construction();
    container::save(std::path::Path::new(&path), &d).unwrap();
    let mut reopened = container::load(std::path::Path::new(&path)).unwrap();
    for (width, thickness) in [(80., 10.), (100., 15.), (90., 12.)] {
        reopened
            .parameters
            .iter_mut()
            .find(|p| p.name == "width")
            .unwrap()
            .expression = format!("{width} mm");
        reopened
            .parameters
            .iter_mut()
            .find(|p| p.name == "thickness")
            .unwrap()
            .expression = format!("{thickness} mm");
        let mesh = solid::evaluate(&reopened, || false).unwrap().mesh.unwrap();
        assert!((mesh.volume - (width * 50. * thickness - 20. * 10. * 3.) * 1e-9).abs() < 1e-11);
    }
    println!(
        "Saved plate with face-attached pocket to {path}; exact regeneration passes three size configurations."
    );
}
