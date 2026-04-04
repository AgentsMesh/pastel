use std::path::Path;

use crate::pipeline;

pub fn run(file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let ir = pipeline::compile_file(file)?;
    println!("✓ {} is valid", file.display());
    println!(
        "  {} nodes, {} assets",
        count_nodes(&ir.nodes),
        ir.assets.len()
    );
    Ok(())
}

fn count_nodes(nodes: &[pastel_lang::ir::node::IrNode]) -> usize {
    nodes
        .iter()
        .map(|n| 1 + count_nodes(&n.children))
        .sum()
}
