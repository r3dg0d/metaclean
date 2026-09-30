use super::{Inspection, MetaField};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

const CORE_PROPS: &str = "docProps/core.xml";
const APP_PROPS: &str = "docProps/app.xml";

pub fn inspect_office(path: &Path, kind: &str) -> Result<Inspection> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).context("open as zip (OOXML)")?;
    let mut inspection = Inspection {
        path: path.display().to_string(),
        kind: kind.into(),
        tool: "zip+xml".into(),
        fields: vec![],
        notes: vec![],
    };

    for name in [CORE_PROPS, APP_PROPS] {
        if let Ok(mut f) = archive.by_name(name) {
            let mut buf = String::new();
            f.read_to_string(&mut buf)?;
            extract_xml_fields(&buf, name, &mut inspection.fields);
        }
    }
    if inspection.fields.is_empty() {
        inspection
            .notes
            .push("no docProps core/app metadata found".into());
    }
    Ok(inspection)
}

fn extract_xml_fields(xml: &str, source: &str, fields: &mut Vec<MetaField>) {
    // Extremely small tag scraper for common Dublin Core / app props
    let tags = [
        ("dc:creator", "author"),
        ("cp:lastModifiedBy", "author"),
        ("dc:title", "document"),
        ("dc:subject", "document"),
        ("dc:description", "document"),
        ("cp:keywords", "document"),
        ("cp:revision", "document"),
        ("dcterms:created", "timestamp"),
        ("dcterms:modified", "timestamp"),
        ("Application", "software"),
        ("AppVersion", "software"),
        ("Company", "author"),
    ];
    for (tag, cat) in tags {
        if let Some(val) = simple_tag_value(xml, tag) {
            if !val.trim().is_empty() {
                fields.push(MetaField {
                    key: format!("{source}:{tag}"),
                    value: val,
                    category: cat.into(),
                });
            }
        }
    }
}

fn simple_tag_value(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let start = xml.find(&open)?;
    let after_lt = &xml[start..];
    let gt = after_lt.find('>')?;
    let content_start = start + gt + 1;
    let end = xml[content_start..].find(&close)? + content_start;
    Some(xml[content_start..end].trim().to_string())
}

pub fn scrub_office_copy(src: &Path, dst: &Path) -> Result<Vec<String>> {
    let file = File::open(src)?;
    let mut archive = ZipArchive::new(file)?;
    let mut out = File::create(dst)?;
    let mut writer = ZipWriter::new(&mut out);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut removed = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        if name == CORE_PROPS || name == APP_PROPS {
            // Write emptied/minimal stubs
            writer.start_file(&name, opts)?;
            if name == CORE_PROPS {
                writer.write_all(
                    br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"></cp:coreProperties>"#,
                )?;
            } else {
                writer.write_all(
                    br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"></Properties>"#,
                )?;
            }
            removed.push(format!("cleared {name}"));
        } else {
            writer.start_file(&name, opts)?;
            std::io::copy(&mut file, &mut writer)?;
        }
    }
    writer.finish()?;
    if removed.is_empty() {
        removed.push("no docProps entries to clear".into());
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrape_creator() {
        let xml = r#"<cp:coreProperties><dc:creator>Alice</dc:creator><dcterms:created>2020-01-01</dcterms:created></cp:coreProperties>"#;
        let mut fields = vec![];
        extract_xml_fields(xml, "docProps/core.xml", &mut fields);
        assert!(fields.iter().any(|f| f.value == "Alice"));
    }
}
