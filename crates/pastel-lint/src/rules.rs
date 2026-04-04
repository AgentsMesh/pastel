use pastel_lang::ir::node::{IrNode, IrNodeData};
use pastel_lang::ir::style::*;
use pastel_lang::ir::{IrDocument, IrTokenGroup, IrTokenValue};

use crate::report::{LintReport, Violation};

/// Lint an IR document: check that design values use defined tokens.
pub fn lint_document(ir: &IrDocument, source_file: &str) -> LintReport {
    let mut report = LintReport::new();

    let colors = collect_colors(&ir.tokens);
    let spacings = collect_numbers(&ir.tokens, "spacing");
    let radii = collect_numbers(&ir.tokens, "radius");
    let font_sizes = collect_font_sizes(&ir.tokens);

    // Skip if no tokens defined (nothing to lint against)
    if colors.is_empty() && spacings.is_empty() && radii.is_empty() && font_sizes.is_empty() {
        return report;
    }

    for node in &ir.nodes {
        check_node(node, source_file, &colors, &spacings, &radii, &font_sizes, &mut report);
    }
    for page in &ir.pages {
        for node in &page.nodes {
            check_node(node, source_file, &colors, &spacings, &radii, &font_sizes, &mut report);
        }
    }

    report
}

fn check_node(
    node: &IrNode, file: &str,
    colors: &[String], spacings: &[f64], radii: &[f64], font_sizes: &[f64],
    report: &mut LintReport,
) {
    match &node.data {
        IrNodeData::Frame(f) => {
            check_fill(&f.visual.fill, &node.id, file, colors, report);
            check_stroke_color(&f.visual.stroke, &node.id, file, colors, report);
            check_padding(&f.padding, &node.id, file, spacings, report);
            check_radius(&f.visual.corner_radius, &node.id, file, radii, report);
            if let Some(layout) = &f.layout {
                check_spacing(layout.gap, "gap", &node.id, file, spacings, report);
            }
        }
        IrNodeData::Text(t) => {
            check_text_color(&t.color, &node.id, file, colors, report);
            check_font_size(t.font_size, &node.id, file, font_sizes, report);
        }
        IrNodeData::Image(img) => {
            check_radius(&img.corner_radius, &node.id, file, radii, report);
        }
        IrNodeData::Shape(s) => {
            check_fill(&s.visual.fill, &node.id, file, colors, report);
            check_radius(&s.visual.corner_radius, &node.id, file, radii, report);
        }
    }

    for child in &node.children {
        check_node(child, file, colors, spacings, radii, font_sizes, report);
    }
}

fn check_fill(fill: &Option<Fill>, node_id: &str, file: &str, colors: &[String], report: &mut LintReport) {
    if colors.is_empty() { return; }
    if let Some(Fill::Solid { color }) = fill {
        let hex = color.to_hex().to_uppercase();
        if !colors.contains(&hex) {
            let closest = find_closest_str(&hex, colors);
            report.add(Violation {
                file: file.into(), node: node_id.into(),
                rule: "color-from-token".into(),
                value: hex.clone(),
                message: format!("fill color {} is not in token colors", hex),
                suggestion: closest.map(|c| format!("use {}", c)),
            });
        }
    }
}

fn check_text_color(color: &Option<Color>, node_id: &str, file: &str, colors: &[String], report: &mut LintReport) {
    if colors.is_empty() { return; }
    if let Some(c) = color {
        let hex = c.to_hex().to_uppercase();
        if !colors.contains(&hex) {
            let closest = find_closest_str(&hex, colors);
            report.add(Violation {
                file: file.into(), node: node_id.into(),
                rule: "color-from-token".into(),
                value: hex.clone(),
                message: format!("text color {} is not in token colors", hex),
                suggestion: closest.map(|c| format!("use {}", c)),
            });
        }
    }
}

fn check_stroke_color(stroke: &Option<Stroke>, node_id: &str, file: &str, colors: &[String], report: &mut LintReport) {
    if colors.is_empty() { return; }
    if let Some(s) = stroke {
        let hex = s.color.to_hex().to_uppercase();
        if !colors.contains(&hex) {
            report.add(Violation {
                file: file.into(), node: node_id.into(),
                rule: "color-from-token".into(),
                value: hex, message: "stroke color not in token colors".into(),
                suggestion: None,
            });
        }
    }
}

fn check_padding(padding: &Option<Padding>, node_id: &str, file: &str, spacings: &[f64], report: &mut LintReport) {
    if spacings.is_empty() { return; }
    if let Some(Padding(vals)) = padding {
        for &v in vals {
            if v > 0.0 && !spacings.contains(&v) {
                report.add(Violation {
                    file: file.into(), node: node_id.into(),
                    rule: "spacing-from-token".into(),
                    value: format!("{}px", v),
                    message: format!("padding {}px is not on spacing scale", v),
                    suggestion: Some(format!("use {}px", closest_num(v, spacings))),
                });
                break; // one violation per node
            }
        }
    }
}

fn check_radius(cr: &Option<CornerRadius>, node_id: &str, file: &str, radii: &[f64], report: &mut LintReport) {
    if radii.is_empty() { return; }
    if let Some(CornerRadius(vals)) = cr {
        for &v in vals {
            if v > 0.0 && !radii.contains(&v) {
                report.add(Violation {
                    file: file.into(), node: node_id.into(),
                    rule: "radius-from-token".into(),
                    value: format!("{}px", v),
                    message: format!("radius {}px is not in token radius", v),
                    suggestion: Some(format!("use {}px", closest_num(v, radii))),
                });
                break;
            }
        }
    }
}

fn check_spacing(val: Option<f64>, prop: &str, node_id: &str, file: &str, spacings: &[f64], report: &mut LintReport) {
    if spacings.is_empty() { return; }
    if let Some(v) = val {
        if v > 0.0 && !spacings.contains(&v) {
            report.add(Violation {
                file: file.into(), node: node_id.into(),
                rule: "spacing-from-token".into(),
                value: format!("{}px", v),
                message: format!("{} {}px is not on spacing scale", prop, v),
                suggestion: Some(format!("use {}px", closest_num(v, spacings))),
            });
        }
    }
}

fn check_font_size(size: Option<f64>, node_id: &str, file: &str, sizes: &[f64], report: &mut LintReport) {
    if sizes.is_empty() { return; }
    if let Some(v) = size {
        if v > 0.0 && !sizes.contains(&v) {
            report.add(Violation {
                file: file.into(), node: node_id.into(),
                rule: "font-size-from-token".into(),
                value: format!("{}px", v),
                message: format!("font-size {}px has no typography token", v),
                suggestion: Some(format!("use {}px", closest_num(v, sizes))),
            });
        }
    }
}

// -- Token collectors --

fn collect_colors(tokens: &[IrTokenGroup]) -> Vec<String> {
    tokens.iter().filter(|g| g.name.contains("color"))
        .flat_map(|g| &g.entries)
        .filter_map(|e| match &e.value { IrTokenValue::Color(c) => Some(c.to_uppercase()), _ => None })
        .collect()
}

fn collect_numbers(tokens: &[IrTokenGroup], name: &str) -> Vec<f64> {
    tokens.iter().filter(|g| g.name == name)
        .flat_map(|g| &g.entries)
        .flat_map(|e| match &e.value {
            IrTokenValue::Number(n) => vec![*n],
            IrTokenValue::Array(arr) => arr.iter().filter_map(|v| match v { IrTokenValue::Number(n) => Some(*n), _ => None }).collect(),
            _ => vec![],
        })
        .collect()
}

fn collect_font_sizes(tokens: &[IrTokenGroup]) -> Vec<f64> {
    tokens.iter().filter(|g| g.name.contains("typo"))
        .flat_map(|g| &g.entries)
        .filter_map(|e| match &e.value {
            IrTokenValue::Object(pairs) => pairs.iter().find(|(k, _)| k == "size")
                .and_then(|(_, v)| match v { IrTokenValue::Number(n) => Some(*n), _ => None }),
            _ => None,
        })
        .collect()
}

// -- Helpers --

fn closest_num(val: f64, scale: &[f64]) -> f64 {
    scale.iter().copied().min_by(|a, b| (a - val).abs().partial_cmp(&(b - val).abs()).unwrap()).unwrap_or(val)
}

fn find_closest_str(target: &str, options: &[String]) -> Option<String> {
    options.first().cloned()
}
