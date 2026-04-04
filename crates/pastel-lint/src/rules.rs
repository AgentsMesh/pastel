use std::path::Path;

use pastel_lang::ir::{IrDocument, IrTokenGroup, IrTokenValue};
use regex::Regex;

use crate::report::{LintReport, Violation};

/// Lint a CSS file against design tokens from an IR document.
pub fn lint_file(path: &Path, ir: &IrDocument) -> Result<LintReport, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    let file_str = path.display().to_string();

    let mut report = LintReport::new();
    let colors = collect_token_colors(&ir.tokens);
    let spacings = collect_token_numbers(&ir.tokens, "spacing");
    let radii = collect_token_numbers(&ir.tokens, "radius");
    let font_sizes = collect_font_sizes(&ir.tokens);

    let hex_re = Regex::new(r"#[0-9A-Fa-f]{6,8}").unwrap();
    let px_re = Regex::new(r"(\d+(?:\.\d+)?)px").unwrap();

    for (line_num, line) in content.lines().enumerate() {
        let line_n = line_num + 1;
        let trimmed = line.trim();

        // Check colors
        for cap in hex_re.find_iter(trimmed) {
            let hex = cap.as_str().to_uppercase();
            if !colors.contains(&hex) && !colors.is_empty() {
                let closest = find_closest_color(&hex, &colors);
                report.add(Violation {
                    file: file_str.clone(), line: line_n,
                    rule: "color-from-token".into(),
                    value: hex.clone(),
                    message: format!("color {} is not in token palette", hex),
                    suggestion: closest.map(|c| format!("use {}", c)),
                });
            }
        }

        // Check spacing (padding, margin, gap)
        if trimmed.contains("padding") || trimmed.contains("margin") || trimmed.contains("gap") {
            for cap in px_re.captures_iter(trimmed) {
                let val: f64 = cap[1].parse().unwrap_or(0.0);
                if val > 0.0 && !spacings.is_empty() && !spacings.contains(&val) {
                    let closest = find_closest_number(val, &spacings);
                    report.add(Violation {
                        file: file_str.clone(), line: line_n,
                        rule: "spacing-from-token".into(),
                        value: format!("{}px", val),
                        message: format!("spacing {}px is not on token scale", val),
                        suggestion: Some(format!("use {}px", closest)),
                    });
                }
            }
        }

        // Check font-size
        if trimmed.contains("font-size") {
            for cap in px_re.captures_iter(trimmed) {
                let val: f64 = cap[1].parse().unwrap_or(0.0);
                if val > 0.0 && !font_sizes.is_empty() && !font_sizes.contains(&val) {
                    let closest = find_closest_number(val, &font_sizes);
                    report.add(Violation {
                        file: file_str.clone(), line: line_n,
                        rule: "font-size-from-token".into(),
                        value: format!("{}px", val),
                        message: format!("font-size {}px has no typography token", val),
                        suggestion: Some(format!("use {}px", closest)),
                    });
                }
            }
        }

        // Check border-radius
        if trimmed.contains("border-radius") {
            for cap in px_re.captures_iter(trimmed) {
                let val: f64 = cap[1].parse().unwrap_or(0.0);
                if val > 0.0 && !radii.is_empty() && !radii.contains(&val) {
                    let closest = find_closest_number(val, &radii);
                    report.add(Violation {
                        file: file_str.clone(), line: line_n,
                        rule: "radius-from-token".into(),
                        value: format!("{}px", val),
                        message: format!("border-radius {}px is not in token", val),
                        suggestion: Some(format!("use {}px", closest)),
                    });
                }
            }
        }
    }

    Ok(report)
}

fn collect_token_colors(tokens: &[IrTokenGroup]) -> Vec<String> {
    tokens.iter()
        .filter(|g| g.name.contains("color"))
        .flat_map(|g| g.entries.iter())
        .filter_map(|e| match &e.value {
            IrTokenValue::Color(c) => Some(c.to_uppercase()),
            _ => None,
        })
        .collect()
}

fn collect_token_numbers(tokens: &[IrTokenGroup], name: &str) -> Vec<f64> {
    tokens.iter()
        .filter(|g| g.name == name)
        .flat_map(|g| g.entries.iter())
        .filter_map(|e| match &e.value {
            IrTokenValue::Number(n) => Some(*n),
            IrTokenValue::Array(arr) => {
                // Flatten scale arrays
                None // handled below
            }
            _ => None,
        })
        .chain(
            tokens.iter()
                .filter(|g| g.name == name)
                .flat_map(|g| g.entries.iter())
                .filter_map(|e| match &e.value {
                    IrTokenValue::Array(arr) => Some(arr.iter().filter_map(|v| {
                        if let IrTokenValue::Number(n) = v { Some(*n) } else { None }
                    }).collect::<Vec<_>>()),
                    _ => None,
                })
                .flatten()
        )
        .collect()
}

fn collect_font_sizes(tokens: &[IrTokenGroup]) -> Vec<f64> {
    tokens.iter()
        .filter(|g| g.name.contains("typo"))
        .flat_map(|g| g.entries.iter())
        .filter_map(|e| match &e.value {
            IrTokenValue::Object(pairs) => pairs.iter()
                .find(|(k, _)| k == "size")
                .and_then(|(_, v)| if let IrTokenValue::Number(n) = v { Some(*n) } else { None }),
            _ => None,
        })
        .collect()
}

fn find_closest_color(hex: &str, palette: &[String]) -> Option<String> {
    palette.first().cloned() // Simplified: return first token color
}

fn find_closest_number(val: f64, scale: &[f64]) -> f64 {
    scale.iter().copied()
        .min_by(|a, b| (a - val).abs().partial_cmp(&(b - val).abs()).unwrap())
        .unwrap_or(val)
}
