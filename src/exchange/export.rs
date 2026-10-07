//! Geometry-only printing exports. Input coordinates are SI; output coordinates are millimetres.
use std::{collections::HashMap, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    Stl,
    ThreeMf,
}
impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Stl => "stl",
            Self::ThreeMf => "3mf",
        }
    }
}

/// Indexed, welded export geometry, independent of display vertices and GPU precision.
pub struct ExportMesh {
    pub vertices: Vec<[f64; 3]>,
    pub triangles: Vec<[u32; 3]>,
}
impl ExportMesh {
    pub fn new(vertices: Vec<[f64; 3]>, indices: &[u32]) -> Result<Self, String> {
        if indices.is_empty() || !indices.len().is_multiple_of(3) {
            return Err("No complete solid triangles to export".into());
        }
        let mut welded = Vec::new();
        let mut lookup = HashMap::new();
        let mut remap = Vec::new();
        for p in vertices {
            if p.iter()
                .any(|v| !v.is_finite() || (v * 1000.).abs() > f32::MAX as f64)
            {
                return Err("Invalid export coordinate".into());
            }
            // OCCT tessellates faces independently; weld seams at one nanometre.
            let key = p.map(|v| {
                let value = (v * 1e9).round();
                if value == 0. { 0 } else { value.to_bits() }
            });
            let index = *lookup.entry(key).or_insert_with(|| {
                let id = welded.len() as u32;
                welded.push(p);
                id
            });
            remap.push(index);
        }
        let mut triangles = Vec::new();
        for t in indices.as_chunks::<3>().0 {
            let mut result = [0; 3];
            for i in 0..3 {
                result[i] = *remap
                    .get(t[i] as usize)
                    .ok_or("Export mesh index out of range")?;
            }
            if result[0] == result[1] || result[1] == result[2] || result[2] == result[0] {
                return Err("Degenerate export triangle".into());
            }
            let a = nalgebra::Vector3::from(welded[result[0] as usize]);
            let b = nalgebra::Vector3::from(welded[result[1] as usize]);
            let c = nalgebra::Vector3::from(welded[result[2] as usize]);
            if (b - a).cross(&(c - a)).norm_squared() == 0. {
                return Err("Degenerate export triangle".into());
            }
            triangles.push(result);
        }
        Ok(Self {
            vertices: welded,
            triangles,
        })
    }
}

pub fn save(path: &Path, mesh: &ExportMesh, format: ExportFormat) -> Result<(), String> {
    // Validate even callers that construct ExportMesh directly, before replacing a file.
    let indices: Vec<_> = mesh.triangles.iter().flatten().copied().collect();
    let mesh = ExportMesh::new(mesh.vertices.clone(), &indices)?;
    let mut edges: HashMap<(u32, u32), (u32, i32)> = HashMap::new();
    for t in &mesh.triangles {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            let edge = edges.entry((a.min(b), a.max(b))).or_default();
            edge.0 += 1;
            edge.1 += if a < b { 1 } else { -1 };
        }
    }
    if edges
        .values()
        .any(|&(count, orientation)| count != 2 || orientation != 0)
    {
        return Err("Export requires a closed mesh with consistently oriented triangles".into());
    }
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    match format {
        ExportFormat::Stl => {
            crate::exchange::stl::write(&mut temporary, &mesh).map_err(|e| e.to_string())?
        }
        ExportFormat::ThreeMf => crate::exchange::three_mf::write(temporary.as_file_mut(), &mesh)
            .map_err(|e| e.to_string())?,
    }
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(all(feature = "solver", feature = "kernel"))]
pub fn export_design(
    path: &Path,
    design: &crate::document::schema::Design,
    format: ExportFormat,
) -> Result<(), String> {
    let evaluated = crate::evaluation::solid::evaluate(design, || false)?;
    if !evaluated.solution.conflicts.is_empty() {
        return Err("Resolve sketch conflicts before exporting".into());
    }
    let mesh = evaluated.mesh.ok_or("No solid bodies to export")?;
    if !mesh.inspection.valid {
        return Err("The evaluated solid is invalid".into());
    }
    let mesh = ExportMesh::new(
        mesh.vertices.iter().map(|v| [v.x, v.y, v.z]).collect(),
        &mesh.indices,
    )?;
    save(path, &mesh, format)
}
