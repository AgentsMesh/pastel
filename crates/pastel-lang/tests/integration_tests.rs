/// Full pipeline integration test: hello-world example
use pastel_lang::ir::node::IrNodeData;
use pastel_lang::ir::style::Color;
use pastel_lang::lexer::Lexer;
use pastel_lang::parser::Parser;
use pastel_lang::semantic::SemanticAnalyzer;

fn compile(src: &str) -> pastel_lang::ir::IrDocument {
    let tokens = Lexer::new(src).tokenize().expect("lexer should succeed");
    let program = Parser::new(tokens).parse().expect("parser should succeed");
    SemanticAnalyzer::new()
        .analyze(&program)
        .expect("semantic analysis should succeed")
}

#[test]
fn integration_hello_world() {
    let source = r#"
canvas "hello-world" {
    width  = 400
    height = 300
    background = #FFFFFF
}

frame main {
    width   = fill
    height  = fill
    layout  = vertical
    align   = center
    justify = center
    gap     = 16

    text "Hello, Pastel!" {
        size   = 32
        weight = bold
        color  = #111111
    }

    text "Design as Code" {
        size  = 16
        color = #666666
    }
}
"#;

    let ir = compile(source);

    // Canvas
    assert_eq!(ir.canvas.name, "hello-world");
    assert_eq!(ir.canvas.width, 400);
    assert_eq!(ir.canvas.height, 300);
    assert_eq!(
        ir.canvas.background,
        Some(Color::from_hex("FFFFFF").unwrap())
    );

    // One top-level frame
    assert_eq!(ir.nodes.len(), 1);
    let main = &ir.nodes[0];
    assert_eq!(main.id, "main");

    // Layout
    if let IrNodeData::Frame(f) = &main.data {
        let layout = f.layout.as_ref().unwrap();
        assert_eq!(layout.mode, pastel_lang::ir::style::LayoutMode::Vertical);
        assert_eq!(layout.gap, Some(16.0));
    } else {
        panic!("expected frame");
    }

    // Two text children
    assert_eq!(main.children.len(), 2);
    if let IrNodeData::Text(t) = &main.children[0].data {
        assert_eq!(t.content, "Hello, Pastel!");
        assert_eq!(t.font_size, Some(32.0));
    } else {
        panic!("expected text");
    }
    if let IrNodeData::Text(t) = &main.children[1].data {
        assert_eq!(t.content, "Design as Code");
    } else {
        panic!("expected text");
    }

    // JSON round-trip
    let json_str = serde_json::to_string(&ir).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["version"], 1);
}
