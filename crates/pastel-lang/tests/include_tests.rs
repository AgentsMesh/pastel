use std::path::Path;

use pastel_lang::error::ErrorKind;
use pastel_lang::ir::node::IrNodeData;
use pastel_lang::ir::style::Fill;
use pastel_lang::lexer::Lexer;
use pastel_lang::parser::Parser;
use pastel_lang::semantic::SemanticAnalyzer;

fn fixtures_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").leak()
}

fn compile_file(name: &str) -> pastel_lang::ir::IrDocument {
    let dir = fixtures_dir();
    let path = dir.join(name);
    let source = std::fs::read_to_string(&path).expect("read fixture file");
    let tokens = Lexer::new(&source).tokenize().expect("lex");
    let program = Parser::new(tokens).parse().expect("parse");
    SemanticAnalyzer::new()
        .analyze_with_base(&program, Some(dir))
        .expect("semantic analysis should succeed")
}

fn compile_file_err(name: &str) -> pastel_lang::error::PastelError {
    let dir = fixtures_dir();
    let path = dir.join(name);
    let source = std::fs::read_to_string(&path).expect("read fixture file");
    let tokens = Lexer::new(&source).tokenize().expect("lex");
    let program = Parser::new(tokens).parse().expect("parse");
    SemanticAnalyzer::new()
        .analyze_with_base(&program, Some(dir))
        .unwrap_err()
}

// ── Basic include merges variables ─────────────────────────────────

#[test]
fn include_merges_variables() {
    let ir = compile_file("main_with_include.pastel");

    // The included shared_color (#FF0000) should be used by the frame's fill
    assert_eq!(ir.nodes.len(), 1);
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        match &f.visual.fill {
            Some(Fill::Solid { color }) => {
                assert_eq!(color.r, 255);
                assert_eq!(color.g, 0);
                assert_eq!(color.b, 0);
            }
            _ => panic!("expected solid fill from included variable"),
        }
    } else {
        panic!("expected frame node");
    }
}

// ── Circular include detection ─────────────────────────────────────

#[test]
fn circular_include_detected() {
    let err = compile_file_err("circular_a.pastel");
    assert_eq!(err.kind, ErrorKind::CircularInclude);
    assert!(err.message.contains("circular include"));
}

// ── Include with no base dir uses default ──────────────────────────

#[test]
fn analyze_without_base_dir_still_works() {
    let source = r#"
        let color = #00FF00
        frame box { fill = color }
    "#;
    let tokens = Lexer::new(source).tokenize().expect("lex");
    let program = Parser::new(tokens).parse().expect("parse");
    // analyze() without base dir should work for files without includes
    let ir = SemanticAnalyzer::new().analyze(&program).expect("ok");
    assert_eq!(ir.nodes.len(), 1);
}
