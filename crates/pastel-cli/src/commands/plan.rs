use std::path::Path;

use crate::pipeline;
use pastel_lang::ir::node::{IrNode, IrNodeData};
use pastel_lang::ir::style::Dimension;

pub fn run(file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let ir = pipeline::compile_file(file)?;

    println!(
        "Document: {} ({}x{})",
        ir.canvas.name, ir.canvas.width, ir.canvas.height
    );

    if ir.pages.is_empty() {
        for node in &ir.nodes {
            print_node(node, "", true);
        }
    } else {
        for page in &ir.pages {
            println!("\nPage: \"{}\"", page.name);
            for node in &page.nodes {
                print_node(node, "  ", true);
            }
        }
        // Also show any top-level nodes outside pages
        if !ir.nodes.is_empty() {
            println!("\n(top-level)");
            for node in &ir.nodes {
                print_node(node, "", true);
            }
        }
    }

    let page_node_count: usize = ir.pages.iter().map(|p| count_nodes(&p.nodes)).sum();
    let total = count_nodes(&ir.nodes) + page_node_count;
    println!(
        "\nAssets: {}  Nodes: {}  Pages: {}",
        ir.assets.len(),
        total,
        ir.pages.len().max(1),
    );

    Ok(())
}

fn print_node(node: &IrNode, prefix: &str, is_last: bool) {
    let connector = if is_last { "└── " } else { "├── " };
    let desc = describe_node(node);

    println!("{prefix}{connector}{desc}");

    let new_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        print_node(child, &new_prefix, i == child_count - 1);
    }
}

fn describe_node(node: &IrNode) -> String {
    match &node.data {
        IrNodeData::Frame(f) => {
            let mut desc = "frame".to_string();
            if let Some(name) = &f.name {
                desc = format!("{desc} {name}");
            }
            let dims = format_dims(f.width.as_ref(), f.height.as_ref());
            if !dims.is_empty() {
                desc = format!("{desc} ({dims})");
            }
            if let Some(layout) = &f.layout {
                let mut parts = vec![format!("{:?}", layout.mode).to_lowercase()];
                if let Some(gap) = layout.gap {
                    parts.push(format!("gap={gap}"));
                }
                if let Some(j) = &layout.justify {
                    parts.push(format!("{:?}", j).to_lowercase());
                }
                desc = format!("{desc} [{}]", parts.join(", "));
            }
            desc
        }
        IrNodeData::Text(t) => {
            let content = if t.content.len() > 30 {
                format!("\"{}...\"", &t.content[..27])
            } else {
                format!("\"{}\"", t.content)
            };
            let mut desc = format!("text {content}");
            if let Some(size) = t.font_size {
                desc = format!("{desc} ({size}px");
                if let Some(w) = &t.font_weight {
                    desc = format!("{desc}, {}", format!("{:?}", w).to_lowercase());
                }
                desc = format!("{desc})");
            }
            desc
        }
        IrNodeData::Image(img) => {
            let mut desc = "image".to_string();
            if let Some(name) = &img.name {
                desc = format!("{desc} {name}");
            }
            let dims = format_dims(img.width.as_ref(), img.height.as_ref());
            if !dims.is_empty() {
                desc = format!("{desc} ({dims})");
            }
            desc
        }
        IrNodeData::Shape(s) => {
            let mut desc = format!("shape {:?}", s.shape_type).to_lowercase();
            if let Some(name) = &s.name {
                desc = format!("{desc} {name}");
            }
            desc
        }
    }
}

fn format_dims(w: Option<&Dimension>, h: Option<&Dimension>) -> String {
    let ws = w.map(dim_str).unwrap_or_default();
    let hs = h.map(dim_str).unwrap_or_default();
    if !ws.is_empty() && !hs.is_empty() {
        format!("{}x{}", ws, hs)
    } else if !ws.is_empty() {
        ws
    } else {
        hs
    }
}

fn dim_str(d: &Dimension) -> String {
    match d {
        Dimension::Fixed(n) => format!("{n}"),
        Dimension::Fill => "fill".into(),
        Dimension::Hug => "hug".into(),
    }
}

fn count_nodes(nodes: &[IrNode]) -> usize {
    nodes.iter().map(|n| 1 + count_nodes(&n.children)).sum()
}
