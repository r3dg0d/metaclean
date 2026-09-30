use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub prefer_exiftool: bool,
    pub prefer_mat2: bool,
    pub default_suffix: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            prefer_exiftool: true,
            prefer_mat2: true,
            default_suffix: ".cleaned".into(),
        }
    }
}

impl AppConfig {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path else {
            return Ok(Self::default());
        };
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        if raw.trim_start().starts_with('{') {
            return Ok(serde_json::from_str(&raw)?);
        }
        let mut cfg = Self::default();
        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let k = k.trim();
            let v = v.trim().trim_matches('"').trim_matches('\'');
            match k {
                "prefer_exiftool" => cfg.prefer_exiftool = v == "true" || v == "1",
                "prefer_mat2" => cfg.prefer_mat2 = v == "true" || v == "1",
                "default_suffix" => cfg.default_suffix = v.to_string(),
                _ => {}
            }
        }
        Ok(cfg)
    }
}
