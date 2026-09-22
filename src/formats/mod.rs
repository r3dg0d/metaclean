pub mod detect;
pub mod office;
pub mod pdf;
pub mod rust_exif;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    Jpeg,
    Png,
    Webp,
    Heic,
    Mp4,
    Mov,
    Mp3,
    Flac,
    Wav,
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaField {
    pub key: String,
    pub value: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Inspection {
    pub path: String,
    pub kind: String,
    pub tool: String,
    pub fields: Vec<MetaField>,
    pub notes: Vec<String>,
}
