use super::{Inspection, MetaField};
use anyhow::Result;
use id3::TagLike;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn inspect_jpeg_exif(path: &Path) -> Result<Inspection> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut inspection = Inspection {
        path: path.display().to_string(),
        kind: "jpeg".into(),
        tool: "kamadak-exif".into(),
        fields: vec![],
        notes: vec![],
    };

    let exif = match exif::Reader::new().read_from_container(&mut reader) {
        Ok(e) => e,
        Err(e) => {
            inspection.notes.push(format!("no readable EXIF: {e}"));
            return Ok(inspection);
        }
    };

    for f in exif.fields() {
        let tag = format!("{}", f.tag);
        let value = f.display_value().with_unit(&exif).to_string();
        let category = categorize_exif_tag(&tag);
        inspection.fields.push(MetaField {
            key: tag,
            value,
            category,
        });
    }

    if inspection.fields.is_empty() {
        inspection
            .notes
            .push("EXIF present but empty field set".into());
    }
    Ok(inspection)
}

fn categorize_exif_tag(tag: &str) -> String {
    let t = tag.to_lowercase();
    if t.contains("gps") {
        "gps".into()
    } else if t.contains("date") || t.contains("time") {
        "timestamp".into()
    } else if t.contains("make")
        || t.contains("model")
        || t.contains("lens")
        || t.contains("camera")
    {
        "camera".into()
    } else if t.contains("artist") || t.contains("author") || t.contains("copyright") {
        "author".into()
    } else if t.contains("software") || t.contains("processing") {
        "software".into()
    } else if t.contains("thumbnail") || t.contains("jpeginterchange") {
        "thumbnail".into()
    } else if t.contains("xmp") {
        "xmp".into()
    } else {
        "exif".into()
    }
}

pub fn scrub_jpeg_copy(src: &Path, dst: &Path) -> Result<Vec<String>> {
    // Re-encode via `image` crate to drop EXIF/APP segments
    let img = image::open(src)?;
    let rgb = img.to_rgb8();
    rgb.save_with_format(dst, image::ImageFormat::Jpeg)?;
    Ok(vec![
        "re-encoded JPEG via image crate (EXIF/XMP/thumbnails dropped)".into(),
    ])
}

pub fn scrub_png_copy(src: &Path, dst: &Path) -> Result<Vec<String>> {
    let img = image::open(src)?;
    img.save_with_format(dst, image::ImageFormat::Png)?;
    Ok(vec![
        "re-encoded PNG via image crate (ancillary metadata dropped where possible)".into(),
    ])
}

pub fn inspect_mp3_id3(path: &Path) -> Result<Inspection> {
    let mut inspection = Inspection {
        path: path.display().to_string(),
        kind: "mp3".into(),
        tool: "id3".into(),
        fields: vec![],
        notes: vec![],
    };
    match id3::Tag::read_from_path(path) {
        Ok(tag) => {
            if let Some(t) = tag.title() {
                inspection.fields.push(field("title", t, "id3"));
            }
            if let Some(t) = tag.artist() {
                inspection.fields.push(field("artist", t, "id3"));
            }
            if let Some(t) = tag.album() {
                inspection.fields.push(field("album", t, "id3"));
            }
            if let Some(t) = tag.year() {
                inspection
                    .fields
                    .push(field("year", &t.to_string(), "timestamp"));
            }
            if let Some(t) = tag.genre() {
                inspection.fields.push(field("genre", t, "id3"));
            }
            for picture in tag.pictures() {
                inspection.fields.push(field(
                    "picture",
                    &format!("{:?} {} bytes", picture.picture_type, picture.data.len()),
                    "thumbnail",
                ));
            }
            if inspection.fields.is_empty() {
                inspection
                    .notes
                    .push("ID3 tag present but no common fields".into());
            }
        }
        Err(e) => inspection.notes.push(format!("no ID3 tag: {e}")),
    }
    Ok(inspection)
}

pub fn scrub_mp3_copy(src: &Path, dst: &Path) -> Result<Vec<String>> {
    std::fs::copy(src, dst)?;
    // Remove tags from copy
    match id3::Tag::read_from_path(dst) {
        Ok(_) => {
            id3::Tag::remove_from_path(dst).ok();
            Ok(vec!["removed ID3 tags from copy".into()])
        }
        Err(_) => Ok(vec!["no ID3 tags to remove".into()]),
    }
}

fn field(key: &str, value: &str, category: &str) -> MetaField {
    MetaField {
        key: key.into(),
        value: value.into(),
        category: category.into(),
    }
}
