use super::FileKind;
use std::path::Path;

pub fn detect_kind(path: &Path) -> FileKind {
    if let Some(kind) = infer::get_from_path(path).ok().flatten() {
        match kind.mime_type() {
            "image/jpeg" => return FileKind::Jpeg,
            "image/png" => return FileKind::Png,
            "image/webp" => return FileKind::Webp,
            "image/heic" | "image/heif" => return FileKind::Heic,
            "video/mp4" => return FileKind::Mp4,
            "video/quicktime" => return FileKind::Mov,
            "audio/mpeg" => return FileKind::Mp3,
            "audio/flac" => return FileKind::Flac,
            "audio/wav" | "audio/x-wav" => return FileKind::Wav,
            "application/pdf" => return FileKind::Pdf,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                return FileKind::Docx
            }
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
                return FileKind::Xlsx
            }
            "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
                return FileKind::Pptx
            }
            _ => {}
        }
    }
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => FileKind::Jpeg,
        "png" => FileKind::Png,
        "webp" => FileKind::Webp,
        "heic" | "heif" => FileKind::Heic,
        "mp4" | "m4v" => FileKind::Mp4,
        "mov" => FileKind::Mov,
        "mp3" => FileKind::Mp3,
        "flac" => FileKind::Flac,
        "wav" => FileKind::Wav,
        "pdf" => FileKind::Pdf,
        "docx" => FileKind::Docx,
        "xlsx" => FileKind::Xlsx,
        "pptx" => FileKind::Pptx,
        _ => FileKind::Unknown,
    }
}

impl FileKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FileKind::Jpeg => "jpeg",
            FileKind::Png => "png",
            FileKind::Webp => "webp",
            FileKind::Heic => "heic",
            FileKind::Mp4 => "mp4",
            FileKind::Mov => "mov",
            FileKind::Mp3 => "mp3",
            FileKind::Flac => "flac",
            FileKind::Wav => "wav",
            FileKind::Pdf => "pdf",
            FileKind::Docx => "docx",
            FileKind::Xlsx => "xlsx",
            FileKind::Pptx => "pptx",
            FileKind::Unknown => "unknown",
        }
    }
}
