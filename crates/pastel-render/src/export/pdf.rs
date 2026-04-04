use std::path::Path;
use skia_safe::pdf;

use pastel_lang::ir::IrDocument;

use crate::layout::LayoutTree;
use crate::painter;

/// Export an IR document to PDF bytes.
/// Each page in the document becomes a PDF page.
/// If no pages exist, the top-level nodes form a single page.
pub fn export_pdf(doc: &IrDocument) -> Vec<u8> {
    let w = doc.canvas.width as f32;
    let h = doc.canvas.height as f32;
    let page_size = skia_safe::Size::new(w, h);

    let mut buf: Vec<u8> = Vec::new();
    let mut pdf_doc = pdf::new_document(&mut buf, None);

    if doc.pages.is_empty() {
        // Single page from top-level nodes
        let mut on_page = pdf_doc.begin_page(page_size, None);
        {
            let canvas = on_page.canvas();
            let layout = LayoutTree::compute(doc, canvas);
            painter::paint_document(canvas, doc, &layout);
        }
        pdf_doc = on_page.end_page();
    } else {
        for page in &doc.pages {
            let mut on_page = pdf_doc.begin_page(page_size, None);
            {
                let canvas = on_page.canvas();
                let layout = LayoutTree::compute_nodes(
                    &page.nodes, doc.canvas.width, doc.canvas.height, canvas,
                );
                painter::paint_nodes(canvas, doc, &page.nodes, &layout);
            }
            pdf_doc = on_page.end_page();
        }
    }

    pdf_doc.close();
    buf
}

/// Export IR document to PDF file.
pub fn export_pdf_file(doc: &IrDocument, path: &Path) -> Result<(), String> {
    let bytes = export_pdf(doc);
    std::fs::write(path, bytes)
        .map_err(|e| format!("failed to write {}: {}", path.display(), e))
}
