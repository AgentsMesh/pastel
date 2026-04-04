mod svg;
mod svg_effects;
mod svg_text;
mod pdf;

use skia_safe::{EncodedImageFormat, Surface};
use std::path::Path;

use pastel_lang::ir::IrDocument;
use pastel_lang::ir::node::IrNode;

pub use svg::export_svg;
pub use svg::export_svg_nodes;
pub use pdf::{export_pdf, export_pdf_file};

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

/// Export specific nodes to SVG file (for page rendering).
pub fn export_svg_nodes_file(
    doc: &IrDocument, nodes: &[IrNode], path: &Path,
) -> Result<(), String> {
    let svg = export_svg_nodes(doc, nodes);
    std::fs::write(path, svg)
        .map_err(|e| format!("failed to write {}: {}", path.display(), e))
}
