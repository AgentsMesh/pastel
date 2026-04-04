use std::path::Path;

use crate::pipeline;

pub fn run(file: &Path, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ir = pipeline::compile_file(file)?;

    if json {
        let json_str = serde_json::to_string_pretty(&ir)?;
        println!("{}", json_str);
    } else {
        println!("Document: {} v{}", ir.canvas.name, ir.version);
        println!("Canvas: {}x{}", ir.canvas.width, ir.canvas.height);
        if let Some(bg) = &ir.canvas.background {
            println!("Background: {}", bg.to_hex());
        }
        println!("Assets: {}", ir.assets.len());
        for asset in &ir.assets {
            println!("  {} ({}) -> {}", asset.id, asset.kind, asset.path);
        }
        let total = count_nodes(&ir.nodes);
        println!("Nodes: {}", total);
    }

    Ok(())
}

fn count_nodes(nodes: &[pastel_lang::ir::node::IrNode]) -> usize {
    nodes.iter().map(|n| 1 + count_nodes(&n.children)).sum()
}
