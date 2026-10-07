//! Minimal 3MF Core package with explicit millimetre units and welded indexed geometry.
use super::export::ExportMesh;
use std::io::{self, Seek, Write};
pub(crate) fn write(writer: impl Write + Seek, mesh: &ExportMesh) -> io::Result<()> {
    let mut zip = zip::ZipWriter::new(writer);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/></Types>"#)?;
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#)?;
    zip.start_file("3D/3dmodel.model", options)?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?><model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" type="model"><mesh><vertices>"#)?;
    for p in &mesh.vertices {
        writeln!(
            zip,
            "<vertex x=\"{}\" y=\"{}\" z=\"{}\"/>",
            p[0] * 1000.,
            p[1] * 1000.,
            p[2] * 1000.
        )?;
    }
    zip.write_all(b"</vertices><triangles>")?;
    for t in &mesh.triangles {
        writeln!(
            zip,
            "<triangle v1=\"{}\" v2=\"{}\" v3=\"{}\"/>",
            t[0], t[1], t[2]
        )?;
    }
    zip.write_all(
        b"</triangles></mesh></object></resources><build><item objectid=\"1\"/></build></model>",
    )?;
    zip.finish()?;
    Ok(())
}
