use crate::scrub::engine::verify_file;
use crate::util::config::AppConfig;
use crate::util::output::OutputOpts;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct VerifyReport {
    path: String,
    remaining_fields: usize,
    sensitive_remaining: Vec<String>,
    clean: bool,
    notes: Vec<String>,
}

pub fn run(out: &OutputOpts, cfg: &AppConfig, path: &Path) -> Result<()> {
    let insp = verify_file(path, cfg.prefer_exiftool, cfg.prefer_mat2)?;
    let sensitive: Vec<String> = insp
        .fields
        .iter()
        .filter(|f| {
            matches!(
                f.category.as_str(),
                "gps" | "author" | "timestamp" | "camera" | "thumbnail" | "xmp" | "id3"
            )
        })
        .map(|f| format!("{} ({})", f.key, f.category))
        .collect();
    let report = VerifyReport {
        path: path.display().to_string(),
        remaining_fields: insp.fields.len(),
        clean: sensitive.is_empty(),
        sensitive_remaining: sensitive.clone(),
        notes: insp.notes.clone(),
    };
    out.emit_or_human(&report, || {
        let mut s = format!(
            "verify: {} — {} field(s), clean={}\n",
            report.path, report.remaining_fields, report.clean
        );
        for x in &report.sensitive_remaining {
            s.push_str(&format!("  remaining sensitive: {x}\n"));
        }
        for n in &report.notes {
            s.push_str(&format!("  note: {n}\n"));
        }
        s
    })?;
    Ok(())
}
