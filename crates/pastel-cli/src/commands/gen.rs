use std::path::Path;

pub fn run(file: &Path, format: &str, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let ir = crate::pipeline::compile_file(file)?;

    std::fs::create_dir_all(output)?;

    match format {
        "tokens" => {
            let css = pastel_codegen::tokens::generate_css(&ir.tokens);
            let json = pastel_codegen::tokens::generate_json(&ir.tokens)?;
            std::fs::write(output.join("tokens.css"), &css)?;
            std::fs::write(output.join("tokens.json"), &json)?;
            println!("  Generated tokens.css + tokens.json -> {}", output.display());
        }
        "html" => {
            let html = pastel_codegen::html::generate_html(&ir);
            std::fs::write(output.join("index.html"), &html)?;
            println!("  Generated index.html -> {}", output.display());
        }
        "react" => {
            let (component, tokens_css) = pastel_codegen::react::generate_react(&ir);
            let name = ir.canvas.name.replace('-', "_");
            std::fs::write(output.join(format!("{}.tsx", name)), &component)?;
            std::fs::write(output.join("tokens.css"), &tokens_css)?;
            println!("  Generated {}.tsx + tokens.css -> {}", name, output.display());
        }
        _ => return Err(format!("unknown format '{}' (use: tokens, html, react)", format).into()),
    }
    Ok(())
}
