use pastel_lang::ast::{Expression, NodeKind};
use pastel_lang::lexer::Lexer;
use pastel_lang::parser::Parser;

// ── Helper ──────────────────────────────────────────────────────────────

fn parse(src: &str) -> pastel_lang::ast::Program {
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("lexer should succeed");
    let mut parser = Parser::new(tokens);
    parser.parse().expect("parser should succeed")
}

fn parse_err(src: &str) -> pastel_lang::error::PastelError {
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("lexer should succeed");
    let mut parser = Parser::new(tokens);
    parser.parse().unwrap_err()
}

// ── Canvas ──────────────────────────────────────────────────────────────

#[test]
fn canvas_with_attributes() {
    let prog = parse(
        r#"
        canvas "hello" {
            width  = 400
            height = 300
            background = #FFFFFF
        }
    "#,
    );
    let canvas = prog.canvas.expect("canvas should be present");
    assert_eq!(canvas.name, "hello");
    assert_eq!(canvas.attrs.len(), 3);
    assert_eq!(canvas.attrs[0].key, "width");
    assert_eq!(canvas.attrs[1].key, "height");
    assert_eq!(canvas.attrs[2].key, "background");
}

#[test]
fn no_canvas() {
    let prog = parse("frame root {}");
    assert!(prog.canvas.is_none());
}

// ── Asset ───────────────────────────────────────────────────────────────

#[test]
fn asset_image_keyword() {
    let prog = parse(r#"asset logo = image("./logo.svg")"#);
    assert_eq!(prog.assets.len(), 1);
    assert_eq!(prog.assets[0].name, "logo");
    assert_eq!(prog.assets[0].kind, "image");
    assert_eq!(prog.assets[0].path, "./logo.svg");
}

#[test]
fn asset_font_ident() {
    let prog = parse(r#"asset body = font("./Inter.ttf")"#);
    assert_eq!(prog.assets.len(), 1);
    assert_eq!(prog.assets[0].name, "body");
    assert_eq!(prog.assets[0].kind, "font");
    assert_eq!(prog.assets[0].path, "./Inter.ttf");
}

#[test]
fn multiple_assets() {
    let prog = parse(
        r#"
        asset logo = image("./logo.svg")
        asset hero = image("./hero.jpg")
    "#,
    );
    assert_eq!(prog.assets.len(), 2);
}

// ── Let declarations ────────────────────────────────────────────────────

#[test]
fn let_integer() {
    let prog = parse("let gap_sm = 8");
    assert_eq!(prog.variables.len(), 1);
    assert_eq!(prog.variables[0].name, "gap_sm");
    assert!(matches!(prog.variables[0].value, Expression::Integer(8)));
}

#[test]
fn let_color() {
    let prog = parse("let primary = #0066FF");
    assert_eq!(prog.variables[0].name, "primary");
    assert!(matches!(prog.variables[0].value, Expression::Color(ref c) if c == "0066FF"));
}

#[test]
fn let_string() {
    let prog = parse(r#"let name = "hello""#);
    assert!(matches!(prog.variables[0].value, Expression::String(ref s) if s == "hello"));
}

#[test]
fn let_bool() {
    let prog = parse("let visible = true");
    assert!(matches!(prog.variables[0].value, Expression::Bool(true)));
}

#[test]
fn let_float() {
    let prog = parse("let ratio = 1.5");
    assert!(
        matches!(prog.variables[0].value, Expression::Float(n) if (n - 1.5).abs() < f64::EPSILON)
    );
}

#[test]
fn let_array() {
    let prog = parse("let pad = [8, 16, 8, 16]");
    if let Expression::Array(items) = &prog.variables[0].value {
        assert_eq!(items.len(), 4);
    } else {
        panic!("expected array expression");
    }
}

// ── Frame ───────────────────────────────────────────────────────────────

#[test]
fn frame_with_name_and_attrs() {
    let prog = parse(
        r#"
        frame main {
            width = fill
            height = 100
        }
    "#,
    );
    assert_eq!(prog.nodes.len(), 1);
    let node = &prog.nodes[0];
    assert_eq!(node.kind, NodeKind::Frame);
    assert_eq!(node.name.as_deref(), Some("main"));
    assert_eq!(node.attrs.len(), 2);
}

#[test]
fn frame_empty_block() {
    let prog = parse("frame empty {}");
    let node = &prog.nodes[0];
    assert_eq!(node.kind, NodeKind::Frame);
    assert_eq!(node.name.as_deref(), Some("empty"));
    assert!(node.attrs.is_empty());
    assert!(node.children.is_empty());
}

#[test]
fn frame_without_name() {
    let prog = parse("frame { width = 100 }");
    let node = &prog.nodes[0];
    assert_eq!(node.kind, NodeKind::Frame);
    assert!(node.name.is_none());
}

// ── Text ────────────────────────────────────────────────────────────────

#[test]
fn text_with_label() {
    let prog = parse(r#"text "Hello" { size = 14 }"#);
    let node = &prog.nodes[0];
    assert_eq!(node.kind, NodeKind::Text);
    assert_eq!(node.label.as_deref(), Some("Hello"));
    assert_eq!(node.attrs.len(), 1);
    assert_eq!(node.attrs[0].key, "size");
}

#[test]
fn text_with_label_and_name() {
    let prog = parse(r#"text "Heading" title { size = 24 }"#);
    let node = &prog.nodes[0];
    assert_eq!(node.kind, NodeKind::Text);
    assert_eq!(node.label.as_deref(), Some("Heading"));
    assert_eq!(node.name.as_deref(), Some("title"));
}

// ── Image ───────────────────────────────────────────────────────────────

#[test]
fn image_with_asset_ref() {
    let prog = parse(
        r#"
        asset hero = image("./hero.jpg")
        image hero {
            width  = 800
            height = 450
        }
    "#,
    );
    let node = &prog.nodes[0];
    assert_eq!(node.kind, NodeKind::Image);
    assert_eq!(node.name.as_deref(), Some("hero"));
}

// ── Inline comma-separated attributes ───────────────────────────────────

#[test]
fn inline_attrs_comma_separated() {
    let prog = parse(r#"text "Hi" { size = 14, color = #FFF000 }"#);
    let node = &prog.nodes[0];
    assert_eq!(node.attrs.len(), 2);
    assert_eq!(node.attrs[0].key, "size");
    assert_eq!(node.attrs[1].key, "color");
}

// ── Nested frames ───────────────────────────────────────────────────────

#[test]
fn deeply_nested_frames() {
    let prog = parse(
        r#"
        frame outer {
            width = fill
            frame middle {
                frame inner {
                    width = 100
                }
            }
        }
    "#,
    );
    let outer = &prog.nodes[0];
    assert_eq!(outer.children.len(), 1);
    let middle = &outer.children[0];
    assert_eq!(middle.name.as_deref(), Some("middle"));
    assert_eq!(middle.children.len(), 1);
    let inner = &middle.children[0];
    assert_eq!(inner.name.as_deref(), Some("inner"));
}

// ── Mixed attributes and children ───────────────────────────────────────

#[test]
fn mixed_attrs_and_children() {
    let prog = parse(
        r#"
        frame container {
            width = fill
            layout = vertical
            gap = 16

            text "Title" { size = 24 }
            text "Body"  { size = 14 }
        }
    "#,
    );
    let node = &prog.nodes[0];
    assert_eq!(node.attrs.len(), 3);
    assert_eq!(node.children.len(), 2);
}

// ── Include ─────────────────────────────────────────────────────────────

#[test]
fn include_declaration() {
    let prog = parse(r#"include "./components.pastel""#);
    assert_eq!(prog.includes.len(), 1);
    assert_eq!(prog.includes[0].path, "./components.pastel");
}

// ── Expression types in attributes ──────────────────────────────────────

#[test]
fn attr_ident_value() {
    let prog = parse("frame x { width = fill }");
    let attr = &prog.nodes[0].attrs[0];
    assert_eq!(attr.key, "width");
    assert!(matches!(attr.value, Expression::Ident(ref s) if s == "fill"));
}

#[test]
fn attr_array_value() {
    let prog = parse("frame x { padding = [8, 16] }");
    let attr = &prog.nodes[0].attrs[0];
    if let Expression::Array(items) = &attr.value {
        assert_eq!(items.len(), 2);
    } else {
        panic!("expected array");
    }
}

// ── Error cases ─────────────────────────────────────────────────────────

#[test]
fn error_unexpected_top_level_token() {
    let err = parse_err("42");
    assert_eq!(err.kind, pastel_lang::error::ErrorKind::ExpectedDeclaration);
}

#[test]
fn error_missing_brace() {
    let err = parse_err("frame x { width = 100");
    assert_eq!(err.kind, pastel_lang::error::ErrorKind::ExpectedToken);
}

// ── Full program ────────────────────────────────────────────────────────

#[test]
fn full_hello_world_structure() {
    let prog = parse(
        r#"
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
    "#,
    );
    assert!(prog.canvas.is_some());
    assert_eq!(prog.nodes.len(), 1);
    assert_eq!(prog.nodes[0].children.len(), 2);
}
