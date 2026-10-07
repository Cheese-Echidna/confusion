//! Binary STL writer. Coordinates are emitted in millimetres; STL cannot store units.
use super::export::ExportMesh;
use std::io::{self, Write};
pub(crate) fn write(mut writer: impl Write, mesh: &ExportMesh) -> io::Result<()> {
    let mut header = [0u8; 80];
    let label = b"Confusion binary STL; coordinates in millimetres";
    header[..label.len()].copy_from_slice(label);
    writer.write_all(&header)?;
    let count = u32::try_from(mesh.triangles.len())
        .map_err(|_| io::Error::other("Too many STL triangles"))?;
    writer.write_all(&count.to_le_bytes())?;
    for triangle in &mesh.triangles {
        // Validate the actual serialized geometry, since f32 conversion can collapse
        // distinct vertices or turn a valid f64 triangle into a line.
        let p = triangle.map(|i| mesh.vertices[i as usize].map(|v| (v * 1000.) as f32));
        if p.iter().flatten().any(|v| !v.is_finite()) {
            return Err(io::Error::other("Invalid STL coordinate"));
        }
        let normal = super::export::triangle_normal(p.map(|point| point.map(f64::from)))
            .ok_or_else(|| io::Error::other("Degenerate triangle at STL precision"))?;
        for value in normal {
            writer.write_all(&(value as f32).to_le_bytes())?;
        }
        for point in p {
            for value in point {
                writer.write_all(&value.to_le_bytes())?;
            }
        }
        writer.write_all(&0u16.to_le_bytes())?;
    }
    Ok(())
}
