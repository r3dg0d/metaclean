use crate::formats::detect::detect_kind;
use crate::formats::office::inspect_office;
use crate::formats::pdf::inspect_pdf;
use crate::formats::rust_exif::{inspect_jpeg_exif, inspect_mp3_id3};
use crate::formats::{FileKind, Inspection, MetaField};
use crate::scrub::external::{exiftool_available, exiftool_inspect, mat2_available, which};
use anyhow::Result;
use std::path::Path;

pub fn inspect_file(path: &Path, prefer_exiftool: bool, prefer_mat2: bool) -> Result<Inspection> {
    let kind = detect_kind(path);

    if prefer_exiftool && exiftool_available() {
        if let Ok(mut insp) = exiftool_inspect(path) {
            insp.kind = kind.as_str().into();
            return Ok(insp);
        }
    }
    // mat2 is primarily a scrubber; still note availability
    let _ = prefer_mat2 && mat2_available();

    match kind {
        FileKind::Jpeg => inspect_jpeg_exif(path),
        FileKind::Mp3 => inspect_mp3_id3(path),
        FileKind::Pdf => inspect_pdf(path),
        FileKind::Docx => inspect_office(path, "docx"),
        FileKind::Xlsx => inspect_office(path, "xlsx"),
        FileKind::Pptx => inspect_office(path, "pptx"),
        FileKind::Png | FileKind::Webp => {
            let mut insp = Inspection {
                path: path.display().to_string(),
                kind: kind.as_str().into(),
                tool: "builtin-limited".into(),
                fields: vec![],
                notes: vec![
                    "limited pure-Rust inspection; install exiftool for full metadata".into(),
                ],
            };
            if which("exiftool").is_none() {
                insp.notes
                    .push("exiftool not found — showing file kind only".into());
            }
            // File size as a basic field
            if let Ok(meta) = std::fs::metadata(path) {
                insp.fields.push(MetaField {
                    key: "filesize".into(),
                    value: meta.len().to_string(),
                    category: "file".into(),
                });
            }
            Ok(insp)
        }
        FileKind::Heic
        | FileKind::Mp4
        | FileKind::Mov
        | FileKind::Flac
        | FileKind::Wav
        | FileKind::Unknown => {
            if exiftool_available() {
                let mut insp = exiftool_inspect(path)?;
                insp.kind = kind.as_str().into();
                Ok(insp)
            } else {
                Ok(Inspection {
                    path: path.display().to_string(),
                    kind: kind.as_str().into(),
                    tool: "none".into(),
                    fields: vec![],
                    notes: vec![format!(
                        "no pure-Rust inspector for {}; install exiftool for full support",
                        kind.as_str()
                    )],
                })
            }
        }
    }
}
