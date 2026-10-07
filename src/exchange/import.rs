//! Validated editable Fusion transfer. Native archives require Autodesk's decoder first.
use crate::document::schema::Design;
use serde::Deserialize;
use std::{io::Read, path::Path};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FusionTransfer {
    format: String,
    version: u32,
    design: Design,
}

pub fn load_fusion_transfer(path: &Path) -> Result<Design, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > 4 * 1024 * 1024 {
        return Err("Fusion transfer exceeds 4 MiB".into());
    }
    let mut bytes = Vec::new();
    file.take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("Fusion transfer exceeds 4 MiB".into());
    }
    let mut transfer: FusionTransfer =
        serde_json::from_slice(&bytes).map_err(|e| format!("Invalid Fusion transfer: {e}"))?;
    if transfer.format != "confusion-fusion-transfer" || transfer.version != 1 {
        return Err("Unsupported Fusion transfer format or version".into());
    }
    transfer.design.validate()?;
    crate::parameters::expression::evaluate(&transfer.design)?;
    transfer.design.sync_construction();
    transfer.design.validate()?;
    Ok(transfer.design)
}

pub fn load_design(path: &Path) -> Result<Design, String> {
    match path.extension().and_then(|s| s.to_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("con") => crate::persistence::container::load(path),
        Some("json") => load_fusion_transfer(path),
        Some("f3d" | "f3z") => Err("Native Fusion archives cannot yet be decoded locally. Open the archive in Fusion and run tools/fusion/ConfusionTransfer to preserve supported sketches, constraints and parameters; then open the resulting .fusion.json here. Full Fusion feature parity is not supported.".into()),
        _ => Err("Choose a .con design or .fusion.json editable transfer".into()),
    }
}
