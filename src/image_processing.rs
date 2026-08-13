//! image-upload capability: decode, bound, resize, strip metadata,
//! re-encode. One function, called once at upload time (design.md §8.2 —
//! the read path stays a plain byte stream, no on-the-fly processing).

use image::{imageops::FilterType, ImageFormat, ImageReader};
use std::io::Cursor;

use crate::error::AppError;

/// Bounding box per design.md's Full HD decision — neither dimension may
/// exceed this after resize; smaller sources are never upscaled.
const MAX_DIMENSION: u32 = 1920;

/// Strict decode-time caps, checked *before* the full pixel buffer is
/// allocated — a defense against a small file whose header claims an
/// enormous resolution (decompression-bomb protection), independent of
/// the crate's own default 512MiB `max_alloc` non-strict limit.
const MAX_DECODE_DIMENSION: u32 = 8000;

const ALLOWED_FORMATS: &[ImageFormat] = &[ImageFormat::Jpeg, ImageFormat::Png, ImageFormat::WebP];

pub struct ProcessedImage {
    pub bytes: Vec<u8>,
    pub mime_type: &'static str,
}

/// Validates format/size, decodes, resizes, and re-encodes as JPEG
/// (chosen uniformly for the stored representation regardless of the
/// source format — one code path, universally viewable, no separate
/// WebP-encoder concerns). Re-encoding through the `image` crate's own
/// pixel buffer inherently drops EXIF (the crate doesn't round-trip
/// metadata it doesn't parse into the output), satisfying the "strip
/// EXIF" requirement as a side effect rather than a separate step.
pub fn process_upload(raw: &[u8]) -> Result<ProcessedImage, AppError> {
    let mut reader = ImageReader::new(Cursor::new(raw))
        .with_guessed_format()
        .map_err(|_| AppError::Validation("Datei konnte nicht gelesen werden.".into()))?;

    let format = reader
        .format()
        .ok_or_else(|| AppError::Validation("Unbekanntes Bildformat.".into()))?;
    if !ALLOWED_FORMATS.contains(&format) {
        return Err(AppError::Validation(
            "Nur JPEG, PNG oder WebP werden unterstützt.".into(),
        ));
    }

    reader.limits({
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(MAX_DECODE_DIMENSION);
        limits.max_image_height = Some(MAX_DECODE_DIMENSION);
        limits
    });

    let img = reader
        .decode()
        .map_err(|_| AppError::Validation("Bild konnte nicht dekodiert werden (zu groß oder beschädigt).".into()))?;

    let (w, h) = (img.width(), img.height());
    let resized = if w > MAX_DIMENSION || h > MAX_DIMENSION {
        img.resize(MAX_DIMENSION, MAX_DIMENSION, FilterType::Lanczos3)
    } else {
        img
    };

    let mut out = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut out), ImageFormat::Jpeg)
        .map_err(|_| AppError::Other(anyhow::anyhow!("re-encode failed")))?;

    Ok(ProcessedImage { bytes: out, mime_type: "image/jpeg" })
}
