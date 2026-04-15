/// Full pipeline integration test: landing-page example
use pastel_lang::ir::node::IrNodeData;
use pastel_lang::ir::style::{Color, CornerRadius, Fill};
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
fn integration_landing_page() {
    let source = r#"
canvas "landing-page" {
    width      = 1440
    height     = 900
    background = #F8F9FA
}

asset logo = image("./assets/logo.svg")
asset hero = image("./assets/hero.jpg")

let primary   = #0066FF
let text_dark = #111111
let radius_md = 8

frame navbar {
    width   = fill
    height  = 64
    padding = [0, 40]
    layout  = horizontal
    align   = center
    justify = space-between
    fill    = #FFFFFF
    shadow  = [0, 2, 8, #00000012]

    image logo { width = 120, height = 32 }

    frame nav-links {
        layout = horizontal
        gap    = 32

        text "Features" { size = 14, color = text_dark }
        text "Pricing"  { size = 14, color = text_dark }
        text "Docs"     { size = 14, color = text_dark }
    }

    frame cta-btn {
        padding = [8, 20]
        fill    = primary
        radius  = radius_md

        text "Get Started" { size = 14, weight = medium, color = #FFFFFF }
    }
}

frame hero-section {
    width   = fill
    height  = 600
    layout  = vertical
    align   = center
    justify = center
    gap     = 32

    text "Design as Code" {
        size = 72, weight = bold, color = text_dark, align = center
    }

    text "AI writes .pastel files. Compiler renders pixels." {
        size = 20, color = #666666, align = center
    }

    frame buttons {
        layout = horizontal
        gap    = 16

        frame primary-btn {
            padding = [14, 32]
            fill    = primary
            radius  = radius_md
            text "Start Building" { size = 16, weight = medium, color = #FFFFFF }
        }

        frame secondary-btn {
            padding = [14, 32]
            fill    = transparent
            radius  = radius_md
            stroke  = [1, #DDDDDD]
            text "View Docs" { size = 16, weight = medium, color = text_dark }
        }
    }

    image hero {
        width = 800, height = 450, radius = 12
        shadow = [0, 8, 32, #00000020]
        fit = cover
    }
}
"#;

    let ir = compile(source);
    assert_eq!(ir.canvas.name, "landing-page");
    assert_eq!(ir.canvas.width, 1440);
    assert_eq!(ir.assets.len(), 2);
    assert_eq!(ir.nodes.len(), 2);
    assert_eq!(ir.nodes[0].id, "navbar");
    assert_eq!(ir.nodes[1].id, "hero-section");

    // Variable resolution: text_dark -> #111111
    let nav_links = &ir.nodes[0].children[1];
    if let IrNodeData::Text(t) = &nav_links.children[0].data {
        assert_eq!(t.color, Some(Color::from_hex("111111").unwrap()));
    } else {
        panic!("expected text");
    }

    // Variable resolution: primary -> solid fill #0066FF
    let cta = &ir.nodes[0].children[2];
    if let IrNodeData::Frame(f) = &cta.data {
        match f.visual.fill.as_ref().unwrap() {
            Fill::Solid { color } => assert_eq!(*color, Color::from_hex("0066FF").unwrap()),
            _ => panic!("expected solid fill"),
        }
        assert_eq!(
            f.visual.corner_radius.as_ref().unwrap(),
            &CornerRadius([8.0; 4])
        );
    } else {
        panic!("expected frame");
    }

    // Image auto-links asset
    if let IrNodeData::Image(img) = &ir.nodes[1].children[3].data {
        assert_eq!(img.asset, "hero");
    } else {
        panic!("expected image");
    }

    // Transparent fill
    let secondary = &ir.nodes[1].children[2].children[1];
    if let IrNodeData::Frame(f) = &secondary.data {
        assert!(matches!(f.visual.fill.as_ref().unwrap(), Fill::Transparent));
    } else {
        panic!("expected frame");
    }

    // JSON structure
    let json_str = serde_json::to_string_pretty(&ir).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["version"], 1);
    assert_eq!(json["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(json["assets"].as_array().unwrap().len(), 2);
}
