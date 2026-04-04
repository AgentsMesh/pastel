use pastel_lang::ir::node::{IrNode, IrNodeData};
use pastel_lang::ir::style::*;
use pastel_lang::ir::IrDocument;

/// Generate a standalone HTML page from an IR document.
pub fn generate_html(ir: &IrDocument) -> String {
    let tokens_css = crate::tokens::generate_css(&ir.tokens);
    let nodes = if ir.pages.is_empty() { &ir.nodes } else { &ir.pages[0].nodes };
    let bg = ir.canvas.background.as_ref().map(|c| c.to_hex()).unwrap_or("#FFFFFF".into());

    let mut body = String::new();
    for node in nodes {
        render_node(node, &mut body, 2);
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<style>
{tokens_css}
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{ background: {bg}; width: {w}px; min-height: {h}px; font-family: -apple-system, sans-serif; }}
</style>
</head>
<body>
{body}</body>
</html>
"#,
        title = ir.canvas.name, bg = bg, w = ir.canvas.width, h = ir.canvas.height,
    )
}

fn render_node(node: &IrNode, out: &mut String, indent: usize) {
    let pad = "  ".repeat(indent);
    match &node.data {
        IrNodeData::Frame(f) => {
            let style = frame_style(f);
            out.push_str(&format!("{pad}<div style=\"{style}\">\n"));
            for child in &node.children {
                render_node(child, out, indent + 1);
            }
            out.push_str(&format!("{pad}</div>\n"));
        }
        IrNodeData::Text(t) => {
            let style = text_style(t);
            let tag = if t.wrap == Some(true) { "p" } else { "span" };
            out.push_str(&format!("{pad}<{tag} style=\"{style}\">{}</{tag}>\n", escape_html(&t.content)));
        }
        IrNodeData::Image(img) => {
            let w = dim_css(img.width.as_ref());
            let h = dim_css(img.height.as_ref());
            let fit = img.fit.as_ref().map(|f| format!("object-fit:{};", fit_css(f))).unwrap_or_default();
            out.push_str(&format!("{pad}<img src=\"\" alt=\"{}\" style=\"{w}{h}{fit}\" />\n",
                img.name.as_deref().unwrap_or("image")));
        }
        IrNodeData::Shape(_) => {
            out.push_str(&format!("{pad}<div style=\"{}\"></div>\n", "/* shape */"));
        }
    }
}

fn frame_style(f: &pastel_lang::ir::node::FrameData) -> String {
    let mut s = Vec::new();
    if let Some(w) = &f.width { s.push(format!("width:{}", dim_val(w))); }
    if let Some(h) = &f.height { s.push(format!("height:{}", dim_val(h))); }
    if let Some(p) = &f.padding {
        let [t, r, b, l] = p.0;
        s.push(format!("padding:{}px {}px {}px {}px", t, r, b, l));
    }
    if let Some(layout) = &f.layout {
        match layout.mode {
            LayoutMode::Horizontal => { s.push("display:flex".into()); }
            LayoutMode::Vertical => { s.push("display:flex".into()); s.push("flex-direction:column".into()); }
            LayoutMode::Grid => {
                s.push("display:grid".into());
                if let Some(cols) = layout.columns { s.push(format!("grid-template-columns:repeat({},1fr)", cols)); }
            }
        }
        if let Some(gap) = layout.gap { s.push(format!("gap:{}px", gap)); }
        if let Some(a) = &layout.align { s.push(format!("align-items:{}", align_css(a))); }
        if let Some(j) = &layout.justify { s.push(format!("justify-content:{}", justify_css(j))); }
    }
    if let Some(fill) = &f.visual.fill { s.push(fill_css(fill)); }
    if let Some(stroke) = &f.visual.stroke {
        s.push(format!("border:{}px solid {}", stroke.width, stroke.color.to_hex()));
    }
    if let Some(r) = &f.visual.corner_radius {
        let [a,b,c,d] = r.0; s.push(format!("border-radius:{}px {}px {}px {}px", a, b, c, d));
    }
    if let Some(sh) = &f.visual.shadow {
        s.push(format!("box-shadow:{}px {}px {}px {}", sh.x, sh.y, sh.blur, sh.color.to_hex()));
    }
    if let Some(o) = f.visual.opacity { if o < 1.0 { s.push(format!("opacity:{}", o)); } }
    s.join(";")
}

fn text_style(t: &pastel_lang::ir::node::TextData) -> String {
    let mut s = Vec::new();
    if let Some(fs) = t.font_size { s.push(format!("font-size:{}px", fs)); }
    if let Some(w) = &t.font_weight { s.push(format!("font-weight:{}", w.to_css_value())); }
    if let Some(f) = &t.font_family { s.push(format!("font-family:'{}'", f)); }
    if let Some(c) = &t.color { s.push(format!("color:{}", c.to_hex())); }
    if let Some(a) = &t.text_align { s.push(format!("text-align:{:?}", a).to_lowercase()); }
    if let Some(lh) = t.line_height { s.push(format!("line-height:{}px", lh)); }
    if let Some(ls) = t.letter_spacing { s.push(format!("letter-spacing:{}px", ls)); }
    s.join(";")
}

fn dim_val(d: &Dimension) -> String {
    match d { Dimension::Fixed(n) => format!("{}px", n), Dimension::Fill => "100%".into(), Dimension::Hug => "fit-content".into() }
}

fn dim_css(d: Option<&Dimension>) -> String {
    d.map(|d| format!("{}:{};", if matches!(d, Dimension::Fill) { "width" } else { "width" }, dim_val(d))).unwrap_or_default()
}

fn fill_css(f: &Fill) -> String {
    match f {
        Fill::Solid { color } => format!("background:{}", color.to_hex()),
        Fill::LinearGradient { angle, stops } => {
            let s: Vec<String> = stops.iter().map(|s| format!("{} {}%", s.color.to_hex(), s.position)).collect();
            format!("background:linear-gradient({}deg,{})", angle, s.join(","))
        }
        Fill::RadialGradient { stops, .. } => {
            let s: Vec<String> = stops.iter().map(|s| format!("{} {}%", s.color.to_hex(), s.position)).collect();
            format!("background:radial-gradient(circle,{})", s.join(","))
        }
        Fill::Transparent => "background:transparent".into(),
    }
}

fn align_css(a: &Align) -> &str {
    match a { Align::Start => "flex-start", Align::Center => "center", Align::End => "flex-end", Align::Stretch => "stretch" }
}

fn justify_css(j: &Justify) -> &str {
    match j { Justify::Start => "flex-start", Justify::Center => "center", Justify::End => "flex-end", Justify::SpaceBetween => "space-between", Justify::SpaceAround => "space-around" }
}

fn fit_css(f: &ImageFit) -> &str {
    match f { ImageFit::Cover => "cover", ImageFit::Contain => "contain", ImageFit::Fill => "fill", ImageFit::None => "none" }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
