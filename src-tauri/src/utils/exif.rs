// ============================================================
// Sift - EXIF Reading Utility
// ============================================================

use crate::models::photo::{Dimensions, ExifData};
use std::fs::File;
use std::io::BufReader;

pub fn read_exif_data(jpg_path: &str) -> Result<ExifData, String> {
    let file = File::open(jpg_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut reader = BufReader::new(file);

    let exif_reader = exif::Reader::new();
    let exif = exif_reader
        .read_from_container(&mut reader)
        .map_err(|e| format!("Failed to read EXIF: {}", e))?;

    let camera = get_field_string(&exif, exif::Tag::Model);
    let lens = get_field_string(&exif, exif::Tag::LensModel);
    let iso = get_field_u32(&exif, exif::Tag::PhotographicSensitivity);
    let aperture = get_field_rational_string(&exif, exif::Tag::FNumber, "f/");
    let shutter_speed = get_shutter_speed(&exif);
    let focal_length = get_field_rational_string(&exif, exif::Tag::FocalLength, "");
    let date_taken = get_field_string(&exif, exif::Tag::DateTimeOriginal);

    let width = get_field_u32(&exif, exif::Tag::PixelXDimension);
    let height = get_field_u32(&exif, exif::Tag::PixelYDimension);

    Ok(ExifData {
        camera,
        lens,
        iso,
        aperture,
        shutter_speed,
        focal_length: if focal_length.is_empty() {
            String::new()
        } else {
            format!("{}mm", focal_length)
        },
        date_taken,
        dimensions: Dimensions { width, height },
    })
}

fn get_field_string(exif: &exif::Exif, tag: exif::Tag) -> String {
    exif.get_field(tag, exif::In::PRIMARY)
        .map(|f| f.display_value().to_string().trim_matches('"').to_string())
        .unwrap_or_default()
}

fn get_field_u32(exif: &exif::Exif, tag: exif::Tag) -> u32 {
    exif.get_field(tag, exif::In::PRIMARY)
        .and_then(|f| match &f.value {
            exif::Value::Short(v) => v.first().map(|&x| x as u32),
            exif::Value::Long(v) => v.first().copied(),
            _ => f.display_value().to_string().parse().ok(),
        })
        .unwrap_or(0)
}

fn get_field_rational_string(exif: &exif::Exif, tag: exif::Tag, prefix: &str) -> String {
    exif.get_field(tag, exif::In::PRIMARY)
        .map(|f| {
            let val = f.display_value().to_string();
            if val.is_empty() {
                String::new()
            } else {
                format!("{}{}", prefix, val)
            }
        })
        .unwrap_or_default()
}

fn get_shutter_speed(exif: &exif::Exif) -> String {
    exif.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY)
        .map(|f| {
            let val = f.display_value().to_string();
            if val.is_empty() {
                String::new()
            } else {
                format!("{}s", val)
            }
        })
        .unwrap_or_default()
}
