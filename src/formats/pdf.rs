use super::{Inspection, MetaField};
use anyhow::Result;
use std::path::Path;

pub fn inspect_pdf(path: &Path) -> Result<Inspection> {
    let mut inspection = Inspection {
        path: path.display().to_string(),
        kind: "pdf".into(),
        tool: "lopdf".into(),
        fields: vec![],
        notes: vec![],
    };

    let doc = match lopdf::Document::load(path) {
        Ok(d) => d,
        Err(e) => {
            inspection.notes.push(format!("failed to parse PDF: {e}"));
            return Ok(inspection);
        }
    };

    if let Ok(info_ref) = doc.trailer.get(b"Info") {
        if let Ok(id) = info_ref.as_reference() {
            if let Ok(info) = doc.get_dictionary(id) {
                for (k, v) in info.iter() {
                    let key = String::from_utf8_lossy(k).to_string();
                    let value = format!("{v:?}");
                    let category = match key.to_lowercase().as_str() {
                        "author" | "creator" => "author",
                        "producer" => "software",
                        "title" | "subject" | "keywords" => "document",
                        "creationdate" | "moddate" => "timestamp",
                        _ => "pdf",
                    };
                    inspection.fields.push(MetaField {
                        key,
                        value,
                        category: category.into(),
                    });
                }
            }
        }
    }
    if inspection.fields.is_empty() {
        inspection.notes.push("no Info dictionary metadata found".into());
    }
    Ok(inspection)
}

pub fn scrub_pdf_copy(src: &Path, dst: &Path) -> Result<Vec<String>> {
    let mut doc = lopdf::Document::load(src)?;
    // Remove Info and metadata streams best-effort
    let mut actions = Vec::new();
    if doc.trailer.remove(b"Info").is_some() {
        actions.push("removed Info dictionary from trailer".into());
    }
    if doc.trailer.remove(b"Metadata").is_some() {
        actions.push("removed Metadata reference from trailer".into());
    }
    // Also clear XMP metadata object if present via catalog
    if let Ok(root) = doc.trailer.get(b"Root").and_then(|r| r.as_reference()) {
        if let Ok(catalog) = doc.get_dictionary_mut(root) {
            if catalog.remove(b"Metadata").is_some() {
                actions.push("removed Metadata from catalog".into());
            }
        }
    }
    doc.save(dst)?;
    if actions.is_empty() {
        actions.push("saved copy (no Info/Metadata keys found to strip)".into());
    }
    Ok(actions)
}
