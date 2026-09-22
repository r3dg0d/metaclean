use crate::inspect::engine::inspect_file;
use crate::util::config::AppConfig;
use crate::util::output::OutputOpts;
use anyhow::Result;
use std::path::Path;

pub fn run(out: &OutputOpts, cfg: &AppConfig, path: &Path) -> Result<()> {
    let insp = inspect_file(path, cfg.prefer_exiftool, cfg.prefer_mat2)?;
    out.emit_or_human(&insp, || {
        let mut s = format!(
            "File: {}\nKind: {}\nTool: {}\n\n",
            insp.path, insp.kind, insp.tool
        );
        if insp.fields.is_empty() {
            s.push_str("(no metadata fields found)\n");
        } else {
            for f in &insp.fields {
                s.push_str(&format!("[{}] {}: {}\n", f.category, f.key, f.value));
            }
        }
        for n in &insp.notes {
            s.push_str(&format!("note: {n}\n"));
        }
        s
    })?;
    Ok(())
}
