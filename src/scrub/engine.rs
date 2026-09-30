use crate::formats::detect::detect_kind;
use crate::formats::office::scrub_office_copy;
use crate::formats::pdf::scrub_pdf_copy;
use crate::formats::rust_exif::{scrub_jpeg_copy, scrub_mp3_copy, scrub_png_copy};
use crate::formats::{FileKind, Inspection};
use crate::inspect::engine::inspect_file;
use crate::scrub::external::{
    exiftool_available, exiftool_scrub_to, mat2_available, mat2_scrub_to,
};
use crate::util::xdg::XdgPaths;
use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct ScrubResult {
    pub source: String,
    pub output: Option<String>,
    pub dry_run: bool,
    pub in_place: bool,
    pub backup: Option<String>,
    pub actions: Vec<String>,
    pub would_remove: Vec<String>,
}

pub struct ScrubOpts<'a> {
    pub dry_run: bool,
    pub in_place: bool,
    pub output: Option<&'a Path>,
    pub suffix: &'a str,
    pub prefer_exiftool: bool,
    pub prefer_mat2: bool,
}

pub fn scrub_one(path: &Path, paths: &XdgPaths, opts: &ScrubOpts<'_>) -> Result<ScrubResult> {
    if !path.is_file() {
        bail!("{} is not a file", path.display());
    }
    let inspection = inspect_file(path, opts.prefer_exiftool, opts.prefer_mat2)?;
    let would_remove: Vec<String> = inspection
        .fields
        .iter()
        .map(|f| format!("{} [{}] = {}", f.key, f.category, f.value))
        .collect();

    if opts.dry_run {
        return Ok(ScrubResult {
            source: path.display().to_string(),
            output: None,
            dry_run: true,
            in_place: opts.in_place,
            backup: None,
            actions: vec!["dry-run: no files modified".into()],
            would_remove,
        });
    }

    let dest = resolve_dest(path, opts)?;
    let mut actions = Vec::new();
    let mut backup = None;

    if opts.in_place {
        paths.ensure_all()?;
        let ts = Utc::now().format("%Y%m%dT%H%M%S");
        let bak_name = format!(
            "{}.{ts}.bak",
            path.file_name().and_then(|s| s.to_str()).unwrap_or("file")
        );
        let bak_path = paths.backup_dir.join(bak_name);
        fs::copy(path, &bak_path).with_context(|| format!("backup to {}", bak_path.display()))?;
        backup = Some(bak_path.display().to_string());
        actions.push(format!("backed up original to {}", bak_path.display()));
    }

    // Prefer external tools
    let scrub_actions = if opts.prefer_mat2 && mat2_available() {
        match mat2_scrub_to(path, &dest) {
            Ok(a) => a,
            Err(e) => {
                actions.push(format!("mat2 failed ({e:#}); falling back"));
                builtin_scrub(path, &dest)?
            }
        }
    } else if opts.prefer_exiftool && exiftool_available() {
        match exiftool_scrub_to(path, &dest) {
            Ok(a) => a,
            Err(e) => {
                actions.push(format!("exiftool failed ({e:#}); falling back"));
                builtin_scrub(path, &dest)?
            }
        }
    } else {
        builtin_scrub(path, &dest)?
    };
    actions.extend(scrub_actions);

    if opts.in_place && dest != path {
        fs::rename(&dest, path)
            .or_else(|_| {
                fs::copy(&dest, path)?;
                fs::remove_file(&dest)?;
                Ok::<(), std::io::Error>(())
            })
            .with_context(|| "replace original in-place")?;
        actions.push("replaced original in-place".into());
    }

    Ok(ScrubResult {
        source: path.display().to_string(),
        output: if opts.in_place {
            Some(path.display().to_string())
        } else {
            Some(dest.display().to_string())
        },
        dry_run: false,
        in_place: opts.in_place,
        backup,
        actions,
        would_remove,
    })
}

fn resolve_dest(path: &Path, opts: &ScrubOpts<'_>) -> Result<PathBuf> {
    if let Some(out) = opts.output {
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).ok();
        }
        return Ok(out.to_path_buf());
    }
    if opts.in_place {
        // write to temp sibling then replace
        let tmp = path.with_extension(format!(
            "{}.metaclean.tmp",
            path.extension().and_then(|e| e.to_str()).unwrap_or("bin")
        ));
        return Ok(tmp);
    }
    // default: suffix before extension
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = path.extension().and_then(|s| s.to_str());
    let name = match ext {
        Some(e) => format!("{stem}{}.{e}", opts.suffix),
        None => format!("{stem}{}", opts.suffix),
    };
    Ok(path.with_file_name(name))
}

fn builtin_scrub(src: &Path, dst: &Path) -> Result<Vec<String>> {
    match detect_kind(src) {
        FileKind::Jpeg => scrub_jpeg_copy(src, dst),
        FileKind::Png | FileKind::Webp => scrub_png_copy(src, dst),
        FileKind::Mp3 => scrub_mp3_copy(src, dst),
        FileKind::Pdf => scrub_pdf_copy(src, dst),
        FileKind::Docx | FileKind::Xlsx | FileKind::Pptx => scrub_office_copy(src, dst),
        other => {
            // Last resort: copy and warn
            fs::copy(src, dst)?;
            Ok(vec![format!(
                "no builtin scrubber for {}; copied unchanged — install exiftool/mat2",
                other.as_str()
            )])
        }
    }
}

pub fn verify_file(path: &Path, prefer_exiftool: bool, prefer_mat2: bool) -> Result<Inspection> {
    let insp = inspect_file(path, prefer_exiftool, prefer_mat2)?;
    Ok(insp)
}
