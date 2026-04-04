use pastel_lang::ir::node::IrNodeData;
use pastel_lang::ir::style::ImageFit;
use pastel_lang::lexer::Lexer;
use pastel_lang::parser::Parser;
use pastel_lang::semantic::SemanticAnalyzer;

fn compile(src: &str) -> pastel_lang::ir::IrDocument {
    let tokens = Lexer::new(src).tokenize().expect("lexer should succeed");
    let program = Parser::new(tokens).parse().expect("parser should succeed");
    SemanticAnalyzer::new().analyze(&program).expect("semantic analysis should succeed")
}

fn compile_err(src: &str) -> pastel_lang::error::PastelError {
    let tokens = Lexer::new(src).tokenize().expect("lexer should succeed");
    let program = Parser::new(tokens).parse().expect("parser should succeed");
    SemanticAnalyzer::new().analyze(&program).unwrap_err()
}

// ── Image asset auto-linking ────────────────────────────────────────────

#[test]
fn image_asset_auto_link() {
    let ir = compile(
        r#"
        asset logo = image("./logo.svg")
        image logo {
            width = 120
            height = 32
        }
    "#,
    );
    if let IrNodeData::Image(img) = &ir.nodes[0].data {
        assert_eq!(img.asset, "logo");
    } else {
        panic!("expected image");
    }
}

#[test]
fn image_fit() {
    let ir = compile(
        r#"
        asset hero = image("./hero.jpg")
        image hero { width = 800, height = 450, fit = cover }
    "#,
    );
    if let IrNodeData::Image(img) = &ir.nodes[0].data {
        assert_eq!(img.fit, Some(ImageFit::Cover));
    } else {
        panic!("expected image");
    }
}

// ── Assets in IR ────────────────────────────────────────────────────────

#[test]
fn assets_in_ir() {
    let ir = compile(
        r#"
        asset logo = image("./logo.svg")
        asset hero = image("./hero.jpg")
        frame x {}
    "#,
    );
    assert_eq!(ir.assets.len(), 2);
    let ids: Vec<&str> = ir.assets.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"logo"));
    assert!(ids.contains(&"hero"));
}

// ── Auto-generated IDs ─────────────────────────────────────────────────

#[test]
fn auto_generated_id_for_unnamed_node() {
    let ir = compile(r#"text "anon" { size = 12 }"#);
    assert!(ir.nodes[0].id.starts_with("text_"));
}

#[test]
fn named_node_uses_name_as_id() {
    let ir = compile("frame header { width = fill }");
    assert_eq!(ir.nodes[0].id, "header");
}

// ── Node kind mapping ───────────────────────────────────────────────────

#[test]
fn node_kinds() {
    let ir = compile(
        r#"
        frame f {}
        text "t" { size = 12 }
        image i { width = 100 }
        shape s { width = 50 }
    "#,
    );
    assert!(matches!(ir.nodes[0].data, IrNodeData::Frame(_)));
    assert!(matches!(ir.nodes[1].data, IrNodeData::Text(_)));
    assert!(matches!(ir.nodes[2].data, IrNodeData::Image(_)));
    assert!(matches!(ir.nodes[3].data, IrNodeData::Shape(_)));
}

// ── Nested children IR ──────────────────────────────────────────────────

#[test]
fn nested_children_in_ir() {
    let ir = compile(
        r#"
        frame outer {
            frame inner {
                text "deep" { size = 10 }
            }
        }
    "#,
    );
    assert_eq!(ir.nodes[0].id, "outer");
    assert_eq!(ir.nodes[0].children.len(), 1);
    assert_eq!(ir.nodes[0].children[0].id, "inner");
    assert_eq!(ir.nodes[0].children[0].children.len(), 1);
    if let IrNodeData::Text(t) = &ir.nodes[0].children[0].children[0].data {
        assert_eq!(t.content, "deep");
    } else {
        panic!("expected text");
    }
}

// ── Full pipeline: source → IR JSON ─────────────────────────────────────

#[test]
fn full_pipeline_to_json() {
    let ir = compile(
        r#"
        canvas "test" {
            width = 400
            height = 300
        }
        frame root {
            width = fill
            text "Hello" { size = 16 }
        }
    "#,
    );

    let json_str = serde_json::to_string_pretty(&ir).expect("serialize to JSON");
    let json: serde_json::Value = serde_json::from_str(&json_str).expect("parse JSON");

    assert_eq!(json["version"], 1);
    assert_eq!(json["canvas"]["name"], "test");
    assert_eq!(json["canvas"]["width"], 400);
    assert_eq!(json["canvas"]["height"], 300);
    assert!(json["nodes"].is_array());
    assert_eq!(json["nodes"][0]["id"], "root");
    assert_eq!(json["nodes"][0]["type"], "frame");
    assert_eq!(json["nodes"][0]["children"][0]["type"], "text");
    assert_eq!(json["nodes"][0]["children"][0]["content"], "Hello");
}

// ── Type mismatch errors ────────────────────────────────────────────────

#[test]
fn type_mismatch_dimension_string() {
    let err = compile_err(r#"frame x { width = "not-a-dim" }"#);
    assert_eq!(err.kind, pastel_lang::error::ErrorKind::TypeMismatch);
}

#[test]
fn type_mismatch_color_as_number() {
    let err = compile_err("frame x { fill = 42 }");
    assert_eq!(err.kind, pastel_lang::error::ErrorKind::TypeMismatch);
}
