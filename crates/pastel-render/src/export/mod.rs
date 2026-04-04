mod svg;

use skia_safe::{EncodedImageFormat, Surface};
use std::path::Path;

use pastel_lang::ir::IrDocument;

pub use svg::export_svg;

/// Export a rendered surface to PNG file.
pub fn export_png(surface: &mut Surface, path: &Path) -> Result<(), String> {
    let image = surface.image_snapshot();
    let data = image
        .encode(None, EncodedImageFormat::PNG, 100)
        .ok_or("failed to encode PNG")?;
    std::fs::write(path, data.as_bytes())
        .map_err(|e| format!("failed to write {}: {}", path.display(), e))
}

/// Export IR document to SVG file.
pub fn export_svg_file(doc: &IrDocument, path: &Path) -> Result<(), String> {
    let svg = export_svg(doc);
    std::fs::write(path, svg)
        .map_err(|e| format!("failed to write {}: {}", path.display(), e))
}
