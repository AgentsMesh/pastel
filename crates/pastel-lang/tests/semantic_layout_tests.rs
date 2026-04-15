use pastel_lang::ir::node::IrNodeData;
use pastel_lang::ir::style::{
    Align, Color, CornerRadius, Dimension, Fill, FontWeight, Justify, LayoutMode,
};
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

// ── Variable resolution ─────────────────────────────────────────────────

#[test]
fn variable_color_resolution() {
    let ir = compile(
        r#"
        let primary = #0066FF
        frame btn {
            fill = primary
        }
    "#,
    );
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        match f.visual.fill.as_ref().unwrap() {
            Fill::Solid { color } => assert_eq!(*color, Color::from_hex("0066FF").unwrap()),
            _ => panic!("expected solid fill"),
        }
    } else {
        panic!("expected frame");
    }
}

#[test]
fn variable_number_resolution() {
    let ir = compile(
        r#"
        let r = 8
        frame box {
            radius = r
        }
    "#,
    );
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        assert_eq!(
            f.visual.corner_radius.as_ref().unwrap(),
            &CornerRadius([8.0; 4])
        );
    } else {
        panic!("expected frame");
    }
}

// ── Dimension resolution ────────────────────────────────────────────────

#[test]
fn dimension_fixed_integer() {
    let ir = compile("frame x { width = 100 }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        assert_eq!(f.width.as_ref().unwrap(), &Dimension::Fixed(100.0));
    } else {
        panic!("expected frame");
    }
}

#[test]
fn dimension_fixed_float() {
    let ir = compile("frame x { width = 50.5 }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        match f.width.as_ref().unwrap() {
            Dimension::Fixed(n) => assert!((n - 50.5).abs() < f64::EPSILON),
            _ => panic!("expected fixed dimension"),
        }
    } else {
        panic!("expected frame");
    }
}

#[test]
fn dimension_keyword_fill() {
    let ir = compile("frame x { width = fill }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        assert_eq!(f.width.as_ref().unwrap(), &Dimension::Fill);
    } else {
        panic!("expected frame");
    }
}

#[test]
fn dimension_keyword_hug() {
    let ir = compile("frame x { width = hug }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        assert_eq!(f.width.as_ref().unwrap(), &Dimension::Hug);
    } else {
        panic!("expected frame");
    }
}

// ── Layout ──────────────────────────────────────────────────────────────

#[test]
fn layout_with_gap() {
    let ir = compile(r#"frame x { layout = horizontal, gap = 16 }"#);
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        let layout = f.layout.as_ref().unwrap();
        assert_eq!(layout.mode, LayoutMode::Horizontal);
        assert_eq!(layout.gap, Some(16.0));
    } else {
        panic!("expected frame");
    }
}

#[test]
fn layout_with_align_justify() {
    let ir = compile(r#"frame x { layout = vertical, align = center, justify = space-between }"#);
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        let layout = f.layout.as_ref().unwrap();
        assert_eq!(layout.mode, LayoutMode::Vertical);
        assert_eq!(layout.align, Some(Align::Center));
        assert_eq!(layout.justify, Some(Justify::SpaceBetween));
    } else {
        panic!("expected frame");
    }
}

// ── Canvas ──────────────────────────────────────────────────────────────

#[test]
fn canvas_defaults() {
    let ir = compile("frame x {}");
    assert_eq!(ir.canvas.name, "untitled");
    assert_eq!(ir.canvas.width, 1440);
    assert_eq!(ir.canvas.height, 900);
}

#[test]
fn canvas_explicit() {
    let ir = compile(
        r#"
        canvas "my-canvas" {
            width = 800
            height = 600
            background = #000000
        }
        frame x {}
    "#,
    );
    assert_eq!(ir.canvas.name, "my-canvas");
    assert_eq!(ir.canvas.width, 800);
    assert_eq!(ir.canvas.height, 600);
    assert_eq!(
        ir.canvas.background,
        Some(Color::from_hex("000000").unwrap())
    );
}

// ── Text properties ─────────────────────────────────────────────────────

#[test]
fn text_content_from_label() {
    let ir = compile(r#"text "Hello World" { size = 16 }"#);
    if let IrNodeData::Text(t) = &ir.nodes[0].data {
        assert_eq!(t.content, "Hello World");
        assert_eq!(t.font_size, Some(16.0));
    } else {
        panic!("expected text");
    }
}

#[test]
fn text_color_and_weight() {
    let ir = compile(r#"text "Title" { color = #111111, weight = bold }"#);
    if let IrNodeData::Text(t) = &ir.nodes[0].data {
        assert_eq!(t.color, Some(Color::from_hex("111111").unwrap()));
        assert_eq!(t.font_weight, Some(FontWeight::Bold));
    } else {
        panic!("expected text");
    }
}

#[test]
fn weight_bold_as_ident() {
    let ir = compile(r#"text "Title" { weight = bold }"#);
    if let IrNodeData::Text(t) = &ir.nodes[0].data {
        assert_eq!(t.font_weight, Some(FontWeight::Bold));
    } else {
        panic!("expected text");
    }
}
