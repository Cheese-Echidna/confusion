//! Exact, flattened AP214 STEP solid export, with millimeter file units.
use crate::document::schema::Design;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct StepExportReport {
    pub solids: u32,
    pub volume_m3: f64,
}

/// Solve and evaluate the complete design from scratch, then atomically replace
/// the destination with exact native B-reps. Failure preserves an existing file.
pub fn export_design(path: impl AsRef<Path>, design: &Design) -> Result<StepExportReport, String> {
    #[cfg(all(feature = "solver", feature = "kernel"))]
    {
        use crate::{evaluation::solid, kernel::bridge::ffi};
        let path = path.as_ref();
        let (model, cache) = solid::evaluate_for_export(design)?;
        let mesh = model.mesh.ok_or("Design contains no solids to export")?;
        if !mesh.inspection.valid || mesh.inspection.solids == 0 {
            return Err("STEP export requires valid solids".into());
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let temporary = tempfile::Builder::new()
            .prefix(".confusion-step-")
            .suffix(".step")
            .tempfile_in(parent)
            .map_err(|e| format!("Create STEP temporary file: {e}"))?;
        let native_path = temporary
            .path()
            .to_str()
            .ok_or("STEP temporary path must be UTF-8")?;
        ffi::write_step(
            cache.native.as_ref().ok_or("Missing native evaluation")?,
            native_path,
        )
        .map_err(|e| e.to_string())?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|e| format!("Sync STEP file: {e}"))?;
        temporary
            .persist(path)
            .map_err(|e| format!("Replace STEP destination: {}", e.error))?;
        Ok(StepExportReport {
            solids: mesh.inspection.solids,
            volume_m3: mesh.volume,
        })
    }
    #[cfg(not(all(feature = "solver", feature = "kernel")))]
    {
        let _ = (path, design);
        Err("STEP export requires the solver and kernel features".into())
    }
}
