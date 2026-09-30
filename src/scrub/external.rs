use crate::formats::{Inspection, MetaField};
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn which(bin: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let full = dir.join(bin);
            if full.is_file() {
                Some(full)
            } else {
                None
            }
        })
    })
}

pub fn exiftool_available() -> bool {
    which("exiftool").is_some()
}

pub fn mat2_available() -> bool {
    which("mat2").is_some()
}

pub fn exiftool_inspect(path: &Path) -> Result<Inspection> {
    let bin = which("exiftool").context("exiftool not found")?;
    let output = Command::new(bin)
        .args(["-json", "-a", "-G", "-s", &path.display().to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("run exiftool")?;
    if !output.status.success() {
        bail!(
            "exiftool failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let v: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let mut fields = Vec::new();
    if let Some(arr) = v.as_array() {
        if let Some(obj) = arr.first().and_then(|x| x.as_object()) {
            for (k, val) in obj {
                if k == "SourceFile" {
                    continue;
                }
                let category = categorize_key(k);
                fields.push(MetaField {
                    key: k.clone(),
                    value: value_to_string(val),
                    category,
                });
            }
        }
    }
    Ok(Inspection {
        path: path.display().to_string(),
        kind: "unknown".into(),
        tool: "exiftool".into(),
        fields,
        notes: vec![],
    })
}

fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn categorize_key(k: &str) -> String {
    let l = k.to_lowercase();
    if l.contains("gps") {
        "gps".into()
    } else if l.contains("date") || l.contains("time") {
        "timestamp".into()
    } else if l.contains("make")
        || l.contains("model")
        || l.contains("camera")
        || l.contains("lens")
    {
        "camera".into()
    } else if l.contains("author")
        || l.contains("artist")
        || l.contains("creator")
        || l.contains("by-line")
    {
        "author".into()
    } else if l.contains("software") || l.contains("producer") || l.contains("tool") {
        "software".into()
    } else if l.contains("xmp") {
        "xmp".into()
    } else if l.contains("thumb") {
        "thumbnail".into()
    } else if l.contains("id3") || l.contains("audio") {
        "id3".into()
    } else {
        "meta".into()
    }
}

/// Scrub using exiftool: copy to dst then strip all tags.
pub fn exiftool_scrub_to(src: &Path, dst: &Path) -> Result<Vec<String>> {
    let bin = which("exiftool").context("exiftool not found")?;
    std::fs::copy(src, dst)?;
    let output = Command::new(bin)
        .args(["-all=", "-overwrite_original", &dst.display().to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("run exiftool scrub")?;
    let msg = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if !output.status.success() {
        bail!("exiftool scrub failed: {}", msg.trim());
    }
    Ok(vec![format!("exiftool scrub: {}", msg.trim())])
}

pub fn mat2_scrub_to(src: &Path, dst: &Path) -> Result<Vec<String>> {
    let bin = which("mat2").context("mat2 not found")?;
    let parent = dst.parent().unwrap_or_else(|| Path::new("."));
    let output = Command::new(&bin)
        .args([src.display().to_string()])
        .current_dir(parent)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("run mat2")?;
    if !output.status.success() {
        bail!(
            "mat2 failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = src.extension().and_then(|s| s.to_str()).unwrap_or("");
    let cleaned_name = if ext.is_empty() {
        format!("{stem}.cleaned")
    } else {
        format!("{stem}.cleaned.{ext}")
    };
    // mat2 writes next to source by default
    let cleaned_near_src = src.with_file_name(&cleaned_name);
    let cleaned_near_cwd = parent.join(&cleaned_name);
    let cleaned = if cleaned_near_src.exists() {
        cleaned_near_src
    } else {
        cleaned_near_cwd
    };
    if cleaned.exists() {
        if cleaned != dst {
            std::fs::rename(&cleaned, dst).or_else(|_| {
                std::fs::copy(&cleaned, dst)?;
                std::fs::remove_file(&cleaned)?;
                Ok::<(), std::io::Error>(())
            })?;
        }
        Ok(vec!["mat2 produced cleaned copy".into()])
    } else {
        std::fs::copy(src, dst)?;
        Ok(vec![
            "mat2 ran but cleaned file not found — copied original".into(),
        ])
    }
}
