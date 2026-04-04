use std::path::Path;

pub fn run(file: &Path, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    // Verify file exists and compiles first
    let ir = crate::pipeline::compile_file(file)?;

    println!("pastel serve: {}", file.display());
    println!("  ✓ Compiled ({} nodes)", count_nodes(&ir.nodes));
    println!("  Preview server requires @pastel/preview (TypeScript).");
    println!(
        "  Run: npx @pastel/preview serve {} --port {}",
        file.display(),
        port
    );

    Ok(())
}

fn count_nodes(nodes: &[pastel_lang::ir::node::IrNode]) -> usize {
    nodes.iter().map(|n| 1 + count_nodes(&n.children)).sum()
}
