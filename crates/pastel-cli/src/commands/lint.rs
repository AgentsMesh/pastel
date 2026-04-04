use std::path::Path;

pub fn run(path: &Path, _rules: &Path, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Lint a .pastel file against its own token definitions
    let ir = crate::pipeline::compile_file(path)?;

    let report = pastel_lint::lint_document(&ir, &path.display().to_string());

    match format {
        "json" => println!("{}", report.format_json()),
        _ => print!("{}", report.format_text()),
    }

    if !report.violations.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}
