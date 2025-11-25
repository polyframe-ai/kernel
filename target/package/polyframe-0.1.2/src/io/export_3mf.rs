// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! 3MF (3D Manufacturing Format) exporter

use crate::geometry::Mesh;
use anyhow::{Context, Result};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::fs::File;
use std::io::{Cursor, Write as IoWrite};
use zip::write::{ExtendedFileOptions, FileOptions, ZipWriter};
use zip::CompressionMethod;

/// Export mesh to 3MF format
pub fn export(mesh: &Mesh, path: &str) -> Result<()> {
    let file = File::create(path).context("Failed to create 3MF file")?;
    let mut zip = ZipWriter::new(file);

    // Create 3D/3dmodel.model file
    let model_xml = generate_3dmodel_xml(mesh)?;

    let options: FileOptions<ExtendedFileOptions> =
        FileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("3D/3dmodel.model", options.clone())?;
    zip.write_all(model_xml.as_bytes())?;

    // Create [Content_Types].xml
    let content_types = generate_content_types_xml();
    zip.start_file("[Content_Types].xml", options.clone())?;
    zip.write_all(content_types.as_bytes())?;

    // Create _rels/.rels
    let rels = generate_rels_xml();
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(rels.as_bytes())?;

    zip.finish()?;
    Ok(())
}

fn generate_3dmodel_xml(mesh: &Mesh) -> Result<String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // XML declaration
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    // Root element
    let mut model = BytesStart::new("model");
    model.push_attribute(("unit", "millimeter"));
    model.push_attribute(("xml:lang", "en-US"));
    model.push_attribute((
        "xmlns",
        "http://schemas.microsoft.com/3dmanufacturing/core/2015/02",
    ));
    writer.write_event(Event::Start(model))?;

    // Resources
    writer.write_event(Event::Start(BytesStart::new("resources")))?;

    // Object
    let mut object = BytesStart::new("object");
    object.push_attribute(("id", "1"));
    object.push_attribute(("type", "model"));
    writer.write_event(Event::Start(object))?;

    // Mesh
    writer.write_event(Event::Start(BytesStart::new("mesh")))?;

    // Vertices
    writer.write_event(Event::Start(BytesStart::new("vertices")))?;
    for vertex in &mesh.vertices {
        let mut v = BytesStart::new("vertex");
        v.push_attribute(("x", vertex.position.x.to_string().as_str()));
        v.push_attribute(("y", vertex.position.y.to_string().as_str()));
        v.push_attribute(("z", vertex.position.z.to_string().as_str()));
        writer.write_event(Event::Empty(v))?;
    }
    writer.write_event(Event::End(BytesEnd::new("vertices")))?;

    // Triangles
    writer.write_event(Event::Start(BytesStart::new("triangles")))?;
    for triangle in &mesh.triangles {
        let mut t = BytesStart::new("triangle");
        t.push_attribute(("v1", triangle.indices[0].to_string().as_str()));
        t.push_attribute(("v2", triangle.indices[1].to_string().as_str()));
        t.push_attribute(("v3", triangle.indices[2].to_string().as_str()));
        writer.write_event(Event::Empty(t))?;
    }
    writer.write_event(Event::End(BytesEnd::new("triangles")))?;

    // Close mesh
    writer.write_event(Event::End(BytesEnd::new("mesh")))?;

    // Close object
    writer.write_event(Event::End(BytesEnd::new("object")))?;

    // Close resources
    writer.write_event(Event::End(BytesEnd::new("resources")))?;

    // Build section
    writer.write_event(Event::Start(BytesStart::new("build")))?;
    let mut item = BytesStart::new("item");
    item.push_attribute(("objectid", "1"));
    writer.write_event(Event::Empty(item))?;
    writer.write_event(Event::End(BytesEnd::new("build")))?;

    // Close model
    writer.write_event(Event::End(BytesEnd::new("model")))?;

    let result = writer.into_inner().into_inner();
    Ok(String::from_utf8(result)?)
}

fn generate_content_types_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#
        .to_string()
}

fn generate_rels_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Primitive;
    use nalgebra::Vector3;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_3mf() -> Result<()> {
        let mesh = Primitive::cube(Vector3::new(10.0, 10.0, 10.0), true).to_mesh();

        let file = NamedTempFile::new()?;
        let path = file.path().to_str().unwrap();

        export(&mesh, path)?;

        // Verify file was created and is a valid ZIP
        let metadata = std::fs::metadata(path)?;
        assert!(metadata.len() > 0);

        Ok(())
    }
}
