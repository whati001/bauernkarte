//! Uploads are decoded, bounded, resized and re-encoded once, at upload
//! time; the read path serves stored bytes as-is.

use std::io::Cursor;

use image::{imageops::FilterType, ImageFormat, ImageReader};

use crate::api::error::AppError;

/// Full HD bounding box; smaller sources are never upscaled.
const MAX_DIMENSION: u32 = 1920;

/// Checked before the pixel buffer is allocated — a small file whose
/// header claims an enormous resolution is a decompression bomb.
const MAX_DECODE_DIMENSION: u32 = 8000;

const ALLOWED_FORMATS: &[ImageFormat] = &[ImageFormat::Jpeg, ImageFormat::Png, ImageFormat::WebP];

pub struct ProcessedImage {
    pub bytes: Vec<u8>,
    pub mime_type: &'static str,
}

/// Always re-encoded as JPEG: one code path, universally viewable, and
/// going through the `image` crate's pixel buffer drops EXIF as a side
/// effect (it doesn't round-trip metadata it doesn't parse).
pub fn process_upload(raw: &[u8]) -> Result<ProcessedImage, AppError> {
    let mut reader = ImageReader::new(Cursor::new(raw))
        .with_guessed_format()
        .map_err(|_| AppError::invalid("error-image-decode"))?;
    let format = reader.format().ok_or_else(|| AppError::invalid("error-image-format"))?;
    if !ALLOWED_FORMATS.contains(&format) {
        return Err(AppError::invalid("error-image-format"));
    }
    reader.limits({
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(MAX_DECODE_DIMENSION);
        limits.max_image_height = Some(MAX_DECODE_DIMENSION);
        limits
    });
    let img = reader.decode().map_err(|_| AppError::invalid("error-image-decode"))?;

    let resized = if img.width() > MAX_DIMENSION || img.height() > MAX_DIMENSION {
        img.resize(MAX_DIMENSION, MAX_DIMENSION, FilterType::Lanczos3)
    } else {
        img
    };
    // JPEG has no alpha channel; a transparent PNG would fail to encode.
    let rgb = resized.to_rgb8();
    let mut out = Vec::new();
    rgb.write_to(&mut Cursor::new(&mut out), ImageFormat::Jpeg)
        .map_err(|err| AppError::from(anyhow::anyhow!("re-encode failed: {err}")))?;
    Ok(ProcessedImage { bytes: out, mime_type: "image/jpeg" })
}
