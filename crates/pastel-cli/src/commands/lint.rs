use std::path::Path;

pub fn run(path: &Path, rules: &Path, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ir = crate::pipeline::compile_file(rules)?;

    let files = collect_files(path)?;
    if files.is_empty() {
        println!("No CSS files found in {}", path.display());
        return Ok(());
    }

    let mut total_violations = 0;
    let mut all_reports = Vec::new();

    for file in &files {
        let report = pastel_lint::lint_file(file, &ir)
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        total_violations += report.violations.len();
        all_reports.push(report);
    }

    match format {
        "json" => {
            let combined = pastel_lint::LintReport {
                violations: all_reports.into_iter().flat_map(|r| r.violations).collect(),
            };
            println!("{}", combined.format_json());
        }
        _ => {
            for report in &all_reports {
                print!("{}", report.format_text());
            }
            if total_violations == 0 {
                println!("  All checks passed");
            }
        }
    }

    if total_violations > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn collect_files(path: &Path) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    let mut files = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_file() {
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if matches!(ext, "css" | "html" | "htm" | "tsx" | "jsx") {
                    files.push(p);
                }
            }
        } else if p.is_dir() {
            files.extend(collect_files(&p)?);
        }
    }
    Ok(files)
}
