use std::path::Path;

pub fn run(file: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let ir = crate::pipeline::compile_file(file)?;

    // For now, output IR JSON (renderer will be TypeScript-side)
    let ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("json");

    match ext {
        "json" => {
            let json = serde_json::to_string_pretty(&ir)?;
            std::fs::write(output, json)?;
            println!("✓ IR written to {}", output.display());
        }
        "png" | "svg" => {
            // Write IR JSON to a temp file, then invoke renderer
            // For now, just output IR JSON with a note
            let json_path = output.with_extension("ir.json");
            let json = serde_json::to_string_pretty(&ir)?;
            std::fs::write(&json_path, &json)?;
            println!("✓ IR written to {}", json_path.display());
            println!(
                "  Note: PNG/SVG rendering requires @pastel/renderer (TypeScript)."
            );
            println!(
                "  Run: npx @pastel/renderer render {} -o {}",
                json_path.display(),
                output.display()
            );
        }
        _ => {
            return Err(format!("unsupported output format: .{}", ext).into());
        }
    }

    Ok(())
}
