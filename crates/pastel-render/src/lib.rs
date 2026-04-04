pub mod layout;
mod layout_place;
pub mod painter;
mod painter_text;
pub mod export;

use pastel_lang::ir::IrDocument;

use layout::LayoutTree;
use painter::paint_document;

/// Render an IR document to a Skia surface.
pub fn render(doc: &IrDocument) -> skia_safe::Surface {
    let w = doc.canvas.width as i32;
    let h = doc.canvas.height as i32;

    let mut surface = skia_safe::surfaces::raster_n32_premul((w, h))
        .expect("failed to create Skia surface");

    let layout = LayoutTree::compute(doc, surface.canvas());
    paint_document(surface.canvas(), doc, &layout);

    surface
}
