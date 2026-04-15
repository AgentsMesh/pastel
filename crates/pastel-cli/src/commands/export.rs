use std::path::Path;

/// Export icons from a .pastel file for a specific platform.
pub fn run(
    file: &Path,
    output_dir: &Path,
    platform: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ir = crate::pipeline::compile_file(file)?;

    if ir.pages.is_empty() {
        return Err("no pages found — each page becomes an icon asset".into());
    }

    match platform {
        "ios" => export_ios(file, output_dir, &ir),
        "android" => export_android(file, output_dir, &ir),
        _ => Err(format!(
            "unsupported platform: '{}' (available: ios, android)",
            platform
        )
        .into()),
    }
}

/// Export iOS Asset Catalog format:
/// <output>/<name>.imageset/
///   ├── <name>.png      (1x)
///   ├── <name>@2x.png   (2x)
///   ├── <name>@3x.png   (3x)
///   └── Contents.json
fn export_ios(
    _file: &Path,
    output_dir: &Path,
    ir: &pastel_lang::ir::IrDocument,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output_dir)?;

    let _base_w = ir.canvas.width;
    let _base_h = ir.canvas.height;

    for page in &ir.pages {
        let name = &page.name;
        let imageset_dir = output_dir.join(format!("{}.imageset", name));
        std::fs::create_dir_all(&imageset_dir)?;

        // Render @1x, @2x, @3x
        for scale in [1u32, 2, 3] {
            let suffix = if scale == 1 {
                String::new()
            } else {
                format!("@{}x", scale)
            };
            let filename = format!("{}{}.png", name, suffix);
            let out_path = imageset_dir.join(&filename);

            let mut surface =
                pastel_render::export::render_nodes_scaled(ir, &page.nodes, scale as f32);
            pastel_render::export::export_png(&mut surface, &out_path)
                .map_err(|e| e.to_string())?;
        }

        // Write Contents.json
        let contents = serde_json::json!({
            "images": [
                {
                    "filename": format!("{}.png", name),
                    "idiom": "universal",
                    "scale": "1x"
                },
                {
                    "filename": format!("{}@2x.png", name),
                    "idiom": "universal",
                    "scale": "2x"
                },
                {
                    "filename": format!("{}@3x.png", name),
                    "idiom": "universal",
                    "scale": "3x"
                }
            ],
            "info": {
                "author": "pastel",
                "version": 1
            }
        });

        let json_path = imageset_dir.join("Contents.json");
        std::fs::write(&json_path, serde_json::to_string_pretty(&contents)?)?;

        println!("  Exported {}.imageset (1x/2x/3x)", name);
    }

    println!("  iOS assets -> {}", output_dir.display());
    Ok(())
}

/// Export Android drawable format:
/// <output>/
///   ├── drawable-mdpi/<name>.png     (1x)
///   ├── drawable-hdpi/<name>.png     (1.5x)
///   ├── drawable-xhdpi/<name>.png    (2x)
///   ├── drawable-xxhdpi/<name>.png   (3x)
///   └── drawable-xxxhdpi/<name>.png  (4x)
fn export_android(
    _file: &Path,
    output_dir: &Path,
    ir: &pastel_lang::ir::IrDocument,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output_dir)?;

    let densities: &[(&str, f32)] = &[
        ("drawable-mdpi", 1.0),
        ("drawable-hdpi", 1.5),
        ("drawable-xhdpi", 2.0),
        ("drawable-xxhdpi", 3.0),
        ("drawable-xxxhdpi", 4.0),
    ];

    for page in &ir.pages {
        let name = &page.name;

        for (dir_name, scale) in densities {
            let density_dir = output_dir.join(dir_name);
            std::fs::create_dir_all(&density_dir)?;

            let filename = format!("{}.png", name);
            let out_path = density_dir.join(&filename);

            let mut surface = pastel_render::export::render_nodes_scaled(ir, &page.nodes, *scale);
            pastel_render::export::export_png(&mut surface, &out_path)
                .map_err(|e| e.to_string())?;
        }

        println!("  Exported {} (mdpi~xxxhdpi)", name);
    }

    println!("  Android drawables -> {}", output_dir.display());
    Ok(())
}
