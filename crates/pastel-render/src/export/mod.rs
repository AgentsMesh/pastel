mod pdf;
mod svg;
mod svg_effects;
mod svg_text;

use skia_safe::{EncodedImageFormat, Surface};
use std::path::Path;

use pastel_lang::ir::node::IrNode;
use pastel_lang::ir::IrDocument;

pub use pdf::{export_pdf, export_pdf_file};
pub use svg::export_svg;
pub use svg::export_svg_nodes;

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
    std::fs::write(path, svg).map_err(|e| format!("failed to write {}: {}", path.display(), e))
}

/// Export specific nodes to SVG file (for page rendering).
pub fn export_svg_nodes_file(
    doc: &IrDocument,
    nodes: &[IrNode],
    path: &Path,
) -> Result<(), String> {
    let svg = export_svg_nodes(doc, nodes);
    std::fs::write(path, svg).map_err(|e| format!("failed to write {}: {}", path.display(), e))
}

/// Render nodes at a given scale factor, returning a scaled surface.
pub fn render_nodes_scaled(doc: &IrDocument, nodes: &[IrNode], scale: f32) -> Surface {
    let mut base = crate::render_nodes(doc, nodes);
    let base_image = base.image_snapshot();

    let w = (doc.canvas.width as f32 * scale) as i32;
    let h = (doc.canvas.height as f32 * scale) as i32;

    let mut scaled =
        skia_safe::surfaces::raster_n32_premul((w, h)).expect("failed to create scaled surface");
    scaled
        .canvas()
        .clear(skia_safe::Color4f::new(0.0, 0.0, 0.0, 0.0));
    scaled.canvas().scale((scale, scale));

    let paint = skia_safe::Paint::default();
    scaled
        .canvas()
        .draw_image(&base_image, (0.0, 0.0), Some(&paint));
    scaled
}
