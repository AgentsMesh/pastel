use pastel_lang::ir::node::IrNodeData;
use pastel_lang::ir::style::{Color, CornerRadius, Fill, Padding};
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

// ── Fill ────────────────────────────────────────────────────────────────

#[test]
fn fill_color() {
    let ir = compile("frame x { fill = #FF0066 }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        match f.visual.fill.as_ref().unwrap() {
            Fill::Solid { color } => assert_eq!(*color, Color::from_hex("FF0066").unwrap()),
            _ => panic!("expected solid fill"),
        }
    } else {
        panic!("expected frame");
    }
}

#[test]
fn fill_transparent() {
    let ir = compile("frame x { fill = transparent }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        assert!(matches!(f.visual.fill.as_ref().unwrap(), Fill::Transparent));
    } else {
        panic!("expected frame");
    }
}

// ── Stroke ──────────────────────────────────────────────────────────────

#[test]
fn stroke_array() {
    let ir = compile("frame x { stroke = [1, #DDDDDD] }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        let stroke = f.visual.stroke.as_ref().unwrap();
        assert_eq!(stroke.width, 1.0);
        assert_eq!(stroke.color, Color::from_hex("DDDDDD").unwrap());
    } else {
        panic!("expected frame");
    }
}

#[test]
fn stroke_invalid() {
    let err = compile_err("frame x { stroke = 42 }");
    assert_eq!(err.kind, pastel_lang::error::ErrorKind::TypeMismatch);
}

// ── Shadow ──────────────────────────────────────────────────────────────

#[test]
fn shadow_array() {
    let ir = compile("frame x { shadow = [0, 2, 8, #00000012] }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        let shadow = f.visual.shadow.as_ref().unwrap();
        assert_eq!(shadow.x, 0.0);
        assert_eq!(shadow.y, 2.0);
        assert_eq!(shadow.blur, 8.0);
        assert_eq!(shadow.color, Color::from_hex("00000012").unwrap());
    } else {
        panic!("expected frame");
    }
}

#[test]
fn shadow_invalid_count() {
    let err = compile_err("frame x { shadow = [0, 2] }");
    assert_eq!(err.kind, pastel_lang::error::ErrorKind::TypeMismatch);
}

// ── Opacity ─────────────────────────────────────────────────────────────

#[test]
fn opacity_value() {
    let ir = compile("frame x { opacity = 0.5 }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        assert_eq!(f.visual.opacity, Some(0.5));
    } else {
        panic!("expected frame");
    }
}

// ── Padding ─────────────────────────────────────────────────────────────

#[test]
fn padding_single_number() {
    let ir = compile("frame x { padding = 16 }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        assert_eq!(f.padding.as_ref().unwrap(), &Padding([16.0, 16.0, 16.0, 16.0]));
    } else {
        panic!("expected frame");
    }
}

#[test]
fn padding_1_value_array() {
    let ir = compile("frame x { padding = [16] }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        assert_eq!(f.padding.as_ref().unwrap(), &Padding([16.0, 16.0, 16.0, 16.0]));
    } else {
        panic!("expected frame");
    }
}

#[test]
fn padding_2_value_array() {
    let ir = compile("frame x { padding = [8, 16] }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        assert_eq!(f.padding.as_ref().unwrap(), &Padding([8.0, 16.0, 8.0, 16.0]));
    } else {
        panic!("expected frame");
    }
}

#[test]
fn padding_4_value_array() {
    let ir = compile("frame x { padding = [1, 2, 3, 4] }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        assert_eq!(f.padding.as_ref().unwrap(), &Padding([1.0, 2.0, 3.0, 4.0]));
    } else {
        panic!("expected frame");
    }
}

#[test]
fn padding_invalid_3_values() {
    let err = compile_err("frame x { padding = [1, 2, 3] }");
    assert_eq!(err.kind, pastel_lang::error::ErrorKind::InvalidValue);
}

// ── Corner radius ───────────────────────────────────────────────────────

#[test]
fn radius_single_number() {
    let ir = compile("frame x { radius = 8 }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        assert_eq!(f.visual.corner_radius.as_ref().unwrap(), &CornerRadius([8.0; 4]));
    } else {
        panic!("expected frame");
    }
}

#[test]
fn radius_4_value_array() {
    let ir = compile("frame x { radius = [4, 8, 12, 16] }");
    if let IrNodeData::Frame(f) = &ir.nodes[0].data {
        assert_eq!(
            f.visual.corner_radius.as_ref().unwrap(),
            &CornerRadius([4.0, 8.0, 12.0, 16.0])
        );
    } else {
        panic!("expected frame");
    }
}
