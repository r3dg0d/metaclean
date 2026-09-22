# metaclean

Metadata **inspector** and **scrubber** for privacy-sensitive files.

Supports: **JPEG, PNG, WebP, HEIC, MP4, MOV, MP3, FLAC, WAV, PDF**, and common Office formats (**docx / xlsx / pptx** as zip+XML).

Owner: **r3dg0d** · License: **MIT**

## Safety defaults

- **Never** destructively overwrites originals by default
- Writes a scrubbed copy with suffix `.cleaned` (configurable), or to `--output`
- `--in-place` requires intent and **backs up** to `~/.cache/metaclean/backups/` (XDG cache)
- `--dry-run` shows what would be removed without writing

## Tool preference

1. **`mat2`** (if installed) — excellent multi-format scrubber  
2. **`exiftool`** (if installed) — broad inspect + tag removal  
3. **Built-in Rust fallbacks** — JPEG (re-encode / `kamadak-exif`), PNG/WebP re-encode, MP3 ID3 strip, PDF Info/Metadata removal (`lopdf`), OOXML `docProps` clearing

## Commands

```bash
metaclean inspect photo.jpg
metaclean scrub photo.jpg
metaclean scrub --output /tmp/clean.jpg photo.jpg
metaclean scrub --in-place photo.jpg
metaclean scrub --recursive ~/Pictures/export
metaclean verify photo.cleaned.jpg
metaclean --dry-run scrub photo.jpg
metaclean --json inspect photo.jpg
metaclean completions bash
```

Global flags: `--json` `--verbose` `--quiet` `--config` `--dry-run` `--help` `--version`

## What it surfaces

EXIF **GPS**, timestamps, camera make/model, author/artist, software, XMP, thumbnails, ID3 tags, video container metadata (via exiftool), PDF Info, Office core/app properties.

## Config (XDG)

`~/.config/metaclean/config.toml`:

```toml
prefer_exiftool = true
prefer_mat2 = true
default_suffix = ".cleaned"
```

## Install

```bash
cargo install --path .
# optional backends
# sudo apt install libimage-exiftool-perl mat2
```

## Development

```bash
cargo test
cargo build --release
cargo run -- --help
```

## Disclaimer

Scrubbing reduces metadata footprint; it is not a guarantee against all steganography or residual forensic traces. Prefer `mat2`/`exiftool` for maximum coverage.
