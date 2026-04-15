pub mod export;
pub mod image_cache;
pub mod layout;
mod layout_measure;
mod layout_place;
pub mod painter;
mod painter_effects;
mod painter_leaf;
mod painter_text;
pub mod text_shaping;

use pastel_lang::ir::node::IrNode;
use pastel_lang::ir::IrDocument;

use image_cache::ImageCache;
use layout::LayoutTree;
use painter::paint_document;

/// Render an IR document to a Skia surface (all top-level nodes or single implicit page).
pub fn render(doc: &IrDocument) -> skia_safe::Surface {
    let w = doc.canvas.width as i32;
    let h = doc.canvas.height as i32;

    let mut surface =
        skia_safe::surfaces::raster_n32_premul((w, h)).expect("failed to create Skia surface");

    let mut images = ImageCache::new();
    images.load_from_assets(&doc.assets);

    let layout = LayoutTree::compute(doc, surface.canvas());
    paint_document(surface.canvas(), doc, &layout, &images);

    surface
}

/// Render a specific set of nodes (e.g., a single page) to a Skia surface.
pub fn render_nodes(doc: &IrDocument, nodes: &[IrNode]) -> skia_safe::Surface {
    let w = doc.canvas.width as i32;
    let h = doc.canvas.height as i32;

    let mut surface =
        skia_safe::surfaces::raster_n32_premul((w, h)).expect("failed to create Skia surface");

    let mut images = ImageCache::new();
    images.load_from_assets(&doc.assets);

    let layout =
        LayoutTree::compute_nodes(nodes, doc.canvas.width, doc.canvas.height, surface.canvas());
    painter::paint_nodes(surface.canvas(), doc, nodes, &layout, &images);

    surface
}
