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
        let p = triangle.map(|i| nalgebra::Vector3::from(mesh.vertices[i as usize]));
        let normal = (p[1] - p[0]).cross(&(p[2] - p[0])).normalize();
        for value in normal.iter() {
            writer.write_all(&(*value as f32).to_le_bytes())?;
        }
        for point in p {
            for value in point.iter() {
                writer.write_all(&((*value * 1000.) as f32).to_le_bytes())?;
            }
        }
        writer.write_all(&0u16.to_le_bytes())?;
    }
    Ok(())
}
