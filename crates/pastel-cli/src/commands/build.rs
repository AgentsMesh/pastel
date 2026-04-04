use std::path::Path;

pub fn run(file: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let ir = crate::pipeline::compile_file(file)?;

    let ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");

    match ext {
        "png" => {
            let mut surface = pastel_render::render(&ir);
            pastel_render::export::export_png(&mut surface, output)
                .map_err(|e| e.to_string())?;
            println!("✓ Rendered {} -> {}", file.display(), output.display());
        }
        "json" => {
            let json = serde_json::to_string_pretty(&ir)?;
            std::fs::write(output, json)?;
            println!("✓ IR written to {}", output.display());
        }
        _ => {
            return Err(format!("unsupported output format: .{}", ext).into());
        }
    }

    Ok(())
}
