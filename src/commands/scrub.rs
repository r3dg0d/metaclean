use crate::scrub::engine::{scrub_one, ScrubOpts};
use crate::util::config::AppConfig;
use crate::util::output::OutputOpts;
use crate::util::xdg::XdgPaths;
use anyhow::{bail, Result};
use std::path::Path;
use walkdir::WalkDir;

pub fn run(
    out: &OutputOpts,
    cfg: &AppConfig,
    paths: &XdgPaths,
    target: &Path,
    recursive: bool,
    in_place: bool,
    output: Option<&Path>,
    dry_run: bool,
) -> Result<()> {
    if recursive {
        if !target.is_dir() {
            bail!("--recursive requires a directory");
        }
        if output.is_some() {
            bail!("--output cannot be combined with --recursive");
        }
        let mut results = Vec::new();
        for entry in WalkDir::new(target).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let p = entry.path();
            // skip our own outputs
            if p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.contains(".cleaned."))
                .unwrap_or(false)
            {
                continue;
            }
            match scrub_one(
                p,
                paths,
                &ScrubOpts {
                    dry_run,
                    in_place,
                    output: None,
                    suffix: &cfg.default_suffix,
                    prefer_exiftool: cfg.prefer_exiftool,
                    prefer_mat2: cfg.prefer_mat2,
                },
            ) {
                Ok(r) => {
                    out.print_verbose(&format!("scrubbed {}", p.display()));
                    results.push(r);
                }
                Err(e) => out.warn(&format!("{}: {e:#}", p.display())),
            }
        }
        out.emit_or_human(&results, || {
            let mut s = format!("scrubbed {} file(s)\n", results.len());
            for r in &results {
                s.push_str(&format!(
                    "- {} -> {}\n",
                    r.source,
                    r.output.as_deref().unwrap_or("(dry-run)")
                ));
            }
            s
        })?;
        return Ok(());
    }

    let result = scrub_one(
        target,
        paths,
        &ScrubOpts {
            dry_run,
            in_place,
            output,
            suffix: &cfg.default_suffix,
            prefer_exiftool: cfg.prefer_exiftool,
            prefer_mat2: cfg.prefer_mat2,
        },
    )?;

    out.emit_or_human(&result, || {
        let mut s = String::new();
        s.push_str(&format!("source: {}\n", result.source));
        if let Some(o) = &result.output {
            s.push_str(&format!("output: {o}\n"));
        }
        if let Some(b) = &result.backup {
            s.push_str(&format!("backup: {b}\n"));
        }
        s.push_str("actions:\n");
        for a in &result.actions {
            s.push_str(&format!("  - {a}\n"));
        }
        if result.dry_run || out.verbose {
            s.push_str("metadata that would be / was targeted:\n");
            for w in &result.would_remove {
                s.push_str(&format!("  * {w}\n"));
            }
        }
        s
    })?;
    Ok(())
}
