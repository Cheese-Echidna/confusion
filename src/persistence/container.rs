//! Versioned .con ZIP container containing current intent only: manifest.json and design.json.
//! Exports save/load; validates inputs, limits uncompressed size and replaces atomically
//! in the destination directory. Evaluation always regenerates solved geometry after load.
use crate::document::schema::Design;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    version: u32,
    units: String,
}
const LIMIT: u64 = 4 * 1024 * 1024;
pub fn save(path: &Path, design: &Design) -> Result<(), String> {
    design.validate()?;
    let mut design = design.clone();
    design.sync_construction();
    design.validate()?;
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let temporary = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    let mut writer = zip::ZipWriter::new(temporary.reopen().map_err(|e| e.to_string())?);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    writer
        .start_file("manifest.json", options)
        .map_err(|e| e.to_string())?;
    writer
        .write_all(
            &serde_json::to_vec(&Manifest {
                format: "confusion".into(),
                version: 6,
                units: "metres".into(),
            })
            .unwrap(),
        )
        .map_err(|e| e.to_string())?;
    writer
        .start_file("design.json", options)
        .map_err(|e| e.to_string())?;
    writer
        .write_all(&serde_json::to_vec_pretty(&design).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    writer
        .finish()
        .map_err(|e| e.to_string())?
        .sync_all()
        .map_err(|e| e.to_string())?;
    temporary.persist(path).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    File::open(dir)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn load(path: &Path) -> Result<Design, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > LIMIT {
        return Err("File exceeds current document size limit".into());
    }
    let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    if zip.len() != 2 {
        return Err("Unexpected .con entries".into());
    }
    let mut read = |name: &str| -> Result<Vec<u8>, String> {
        let entry = zip.by_name(name).map_err(|e| e.to_string())?;
        if entry.size() > LIMIT {
            return Err("Document entry exceeds size limit".into());
        }
        let mut result = Vec::new();
        entry
            .take(LIMIT + 1)
            .read_to_end(&mut result)
            .map_err(|e| e.to_string())?;
        if result.len() as u64 > LIMIT {
            return Err("Document entry exceeds size limit".into());
        }
        Ok(result)
    };
    let manifest: Manifest =
        serde_json::from_slice(&read("manifest.json")?).map_err(|e| e.to_string())?;
    if manifest.format != "confusion"
        || !matches!(manifest.version, 1..=6)
        || manifest.units != "metres"
    {
        return Err("Unsupported .con format, version or units".into());
    }
    let mut design: Design =
        serde_json::from_slice(&read("design.json")?).map_err(|e| e.to_string())?;
    design.validate()?;
    if manifest.version == 1 {
        design.sync_construction();
    }
    design.validate()?;
    Ok(design)
}
