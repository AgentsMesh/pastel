use skia_safe::{EncodedImageFormat, Surface};
use std::path::Path;

/// Export a rendered surface to PNG file.
pub fn export_png(surface: &mut Surface, path: &Path) -> Result<(), String> {
    let image = surface.image_snapshot();
    let data = image
        .encode(None, EncodedImageFormat::PNG, 100)
        .ok_or("failed to encode PNG")?;
    std::fs::write(path, data.as_bytes())
        .map_err(|e| format!("failed to write {}: {}", path.display(), e))
}

/// Export a rendered surface to JPEG file.
pub fn export_jpeg(surface: &mut Surface, path: &Path, quality: i32) -> Result<(), String> {
    let image = surface.image_snapshot();
    let data = image
        .encode(None, EncodedImageFormat::JPEG, Some(quality as u32))
        .ok_or("failed to encode JPEG")?;
    std::fs::write(path, data.as_bytes())
        .map_err(|e| format!("failed to write {}: {}", path.display(), e))
}
