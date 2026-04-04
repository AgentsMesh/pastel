use std::path::Path;

pub fn run(
    file: &Path, output: &Path, page_filter: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ir = crate::pipeline::compile_file(file)?;

    let ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");

    // If pages exist and no specific page requested, render all pages
    if !ir.pages.is_empty() && page_filter.is_none() && ext != "pdf" {
        return render_all_pages(file, output, &ir, ext);
    }

    // If a specific page is requested, find and render it
    if let Some(page_name) = page_filter {
        let page = ir.pages.iter().find(|p| p.name == page_name).ok_or_else(|| {
            format!("page '{}' not found (available: {})",
                page_name,
                ir.pages.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", "))
        })?;
        return render_single(file, output, &ir, &page.nodes, ext);
    }

    // No pages or PDF: render normally (top-level nodes)
    match ext {
        "png" => {
            let mut surface = pastel_render::render(&ir);
            pastel_render::export::export_png(&mut surface, output)
                .map_err(|e| e.to_string())?;
            println!("  Rendered {} -> {}", file.display(), output.display());
        }
        "svg" => {
            pastel_render::export::export_svg_file(&ir, output)
                .map_err(|e| e.to_string())?;
            println!("  Exported SVG {} -> {}", file.display(), output.display());
        }
        "pdf" => {
            pastel_render::export::export_pdf_file(&ir, output)
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
            println!("  Exported PDF {} -> {}", file.display(), output.display());
        }
        "json" => {
            let json = serde_json::to_string_pretty(&ir)?;
            std::fs::write(output, json)?;
            println!("  IR written to {}", output.display());
        }
        _ => {
            return Err(format!("unsupported output format: .{}", ext).into());
        }
    }

    Ok(())
}

fn render_all_pages(
    file: &Path, output: &Path, ir: &pastel_lang::ir::IrDocument, ext: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let stem = output.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let parent = output.parent().unwrap_or(Path::new("."));

    for page in &ir.pages {
        let page_path = parent.join(format!("{}_{}.{}", stem, page.name, ext));
        render_single(file, &page_path, ir, &page.nodes, ext)?;
    }
    Ok(())
}

fn render_single(
    file: &Path, output: &Path, ir: &pastel_lang::ir::IrDocument,
    nodes: &[pastel_lang::ir::node::IrNode], ext: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match ext {
        "png" => {
            let mut surface = pastel_render::render_nodes(ir, nodes);
            pastel_render::export::export_png(&mut surface, output)
                .map_err(|e| e.to_string())?;
            println!("  Rendered {} -> {}", file.display(), output.display());
        }
        "svg" => {
            pastel_render::export::export_svg_nodes_file(ir, nodes, output)
                .map_err(|e| e.to_string())?;
            println!("  Exported SVG {} -> {}", file.display(), output.display());
        }
        _ => return Err(format!("unsupported page format: .{}", ext).into()),
    }
    Ok(())
}
