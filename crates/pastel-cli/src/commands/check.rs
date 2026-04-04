use std::path::Path;

use crate::pipeline;

pub fn run(file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let ir = pipeline::compile_file(file)?;
    println!("  {} is valid", file.display());

    let page_nodes: usize = ir.pages.iter().map(|p| count_nodes(&p.nodes)).sum();
    let total = count_nodes(&ir.nodes) + page_nodes;
    let page_info = if ir.pages.is_empty() {
        String::new()
    } else {
        format!(", {} pages", ir.pages.len())
    };
    println!("  {} nodes, {} assets{}", total, ir.assets.len(), page_info);

    Ok(())
}

fn count_nodes(nodes: &[pastel_lang::ir::node::IrNode]) -> usize {
    nodes
        .iter()
        .map(|n| 1 + count_nodes(&n.children))
        .sum()
}
