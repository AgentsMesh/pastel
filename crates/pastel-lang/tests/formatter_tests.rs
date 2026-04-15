use pastel_lang::formatter::Formatter;
use pastel_lang::lexer::Lexer;
use pastel_lang::parser::Parser;

fn parse_and_format(src: &str) -> String {
    let tokens = Lexer::new(src).tokenize().expect("lex");
    let program = Parser::new(tokens).parse().expect("parse");
    Formatter::new().format(&program)
}

// ── Idempotency: format(parse(format(parse(src)))) == format(parse(src)) ──

#[test]
fn format_is_idempotent() {
    let source = r#"
include "./shared.pastel"
asset logo = image("./logo.svg")
let primary = #0066FF
canvas "test" { width = 1440, height = 900 }
frame header {
  width = fill
  fill = primary
  layout = horizontal
  gap = 16
  text "Hello" { size = 32, weight = bold, color = #111111 }
  frame inner {
    padding = [8, 16, 8, 16]
    image logo { width = 120, height = 32 }
  }
}
"#;

    let first = parse_and_format(source);
    let second = parse_and_format(&first);
    assert_eq!(first, second, "formatter should be idempotent");
}

// ── Individual declaration formatting ──────────────────────────────

#[test]
fn format_let_declaration() {
    let out = parse_and_format("let   color   =    #FF0000");
    assert!(out.contains("let color = #FF0000"));
}

#[test]
fn format_canvas_block() {
    let out = parse_and_format(r#"canvas  "test"  {  width=400  height=300 }"#);
    assert!(out.contains("canvas \"test\" {"));
    assert!(out.contains("    width = 400"));
    assert!(out.contains("    height = 300"));
    assert!(out.contains("}"));
}

#[test]
fn format_text_inline_short_attrs() {
    let out = parse_and_format(r#"text "Hi" { size = 14, color = #FFF000 }"#);
    assert!(out.contains("text \"Hi\" { size = 14, color = #FFF000 }"));
}

#[test]
fn format_text_multiline_many_attrs() {
    let out = parse_and_format(
        r#"text "Hello" { size = 14, weight = bold, color = #111111, font = "Inter", align = center }"#,
    );
    // Should use block format with 5 attrs
    assert!(out.contains("text \"Hello\" {"));
    assert!(out.contains("    size = 14"));
    assert!(out.contains("    weight = bold"));
}

#[test]
fn format_frame_with_children() {
    let out = parse_and_format(r#"frame main { width = fill, frame child { height = 100 } }"#);
    assert!(out.contains("frame main {"));
    assert!(out.contains("    width = fill"));
    assert!(out.contains("    frame child {"));
    assert!(out.contains("        height = 100"));
}

#[test]
fn format_include() {
    let out = parse_and_format(r#"include  "./lib.pastel""#);
    assert!(out.contains("include \"./lib.pastel\""));
}

#[test]
fn format_asset() {
    let out = parse_and_format(r#"asset  logo  =  image ( "./logo.svg" )"#);
    assert!(out.contains("asset logo = image(\"./logo.svg\")"));
}

#[test]
fn format_empty_frame() {
    let out = parse_and_format("frame empty {}");
    assert!(out.contains("frame empty {}"));
}

#[test]
fn format_blank_lines_between_declarations() {
    let out = parse_and_format("let a = 1\nlet b = 2\nframe x {}");
    let lines: Vec<&str> = out.lines().collect();
    // Should have blank line between let b and frame x
    assert!(
        lines.contains(&""),
        "expected blank lines between declarations"
    );
}

#[test]
fn format_component() {
    let out =
        parse_and_format(r#"component btn(label, color = #0066FF) { frame x { fill = color } }"#);
    assert!(out.contains("component btn(label, color = #0066FF) {"));
    assert!(out.contains("    frame x {"));
}
