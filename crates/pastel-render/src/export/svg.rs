use pastel_lang::ir::node::{IrNode, IrNodeData, ShapeType};
use pastel_lang::ir::style::{Color, Fill};
use pastel_lang::ir::IrDocument;

use crate::layout::{LayoutTree, Rect, make_font};

/// Generate SVG string from IR document.
pub fn export_svg(doc: &IrDocument) -> String {
    let w = doc.canvas.width;
    let h = doc.canvas.height;

    // Compute layout using a temp Skia surface for text measurement
    let mut surface = skia_safe::surfaces::raster_n32_premul((w as i32, h as i32))
        .expect("failed to create surface for layout");
    let layout = LayoutTree::compute(doc, surface.canvas());

    let bg = doc.canvas.background.as_ref()
        .map(|c| c.to_hex())
        .unwrap_or_else(|| "#FFFFFF".into());

    let mut out = String::new();
    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#
    ));
    out.push('\n');
    out.push_str(&format!(r#"  <rect width="{w}" height="{h}" fill="{bg}" />"#));
    out.push('\n');

    for node in &doc.nodes {
        render_node(node, &layout, &mut out, "  ");
    }

    out.push_str("</svg>\n");
    out
}

fn render_node(node: &IrNode, layout: &LayoutTree, out: &mut String, indent: &str) {
    let rect = match layout.get(&node.id) {
        Some(r) => *r,
        None => return,
    };

    match &node.data {
        IrNodeData::Frame(f) => {
            let has_kids = !node.children.is_empty();
            let fill_attr = f.visual.fill.as_ref().map(fill_str).unwrap_or("none".into());
            let stroke_attr = f.visual.stroke.as_ref()
                .map(|s| format!(r#" stroke="{}" stroke-width="{}""#, s.color.to_hex(), s.width))
                .unwrap_or_default();
            let opacity_attr = f.visual.opacity
                .filter(|&o| o < 1.0)
                .map(|o| format!(r#" opacity="{}""#, o))
                .unwrap_or_default();
            let radius_attr = corner_radius_attr(f.visual.corner_radius.as_ref().map(|r| &r.0), rect);
            let shadow_filter = shadow_filter_attr(f.visual.shadow.as_ref(), &node.id);

            if let Some(shadow) = &f.visual.shadow {
                out.push_str(&format!("{indent}<defs>\n"));
                out.push_str(&format!(
                    r#"{indent}  <filter id="shadow-{id}"><feDropShadow dx="{dx}" dy="{dy}" stdDeviation="{std}" flood-color="{color}" flood-opacity="{opacity}" /></filter>"#,
                    indent = indent, id = node.id, dx = shadow.x, dy = shadow.y,
                    std = shadow.blur / 2.0, color = shadow.color.to_hex(),
                    opacity = shadow.color.a as f32 / 255.0,
                ));
                out.push('\n');
                out.push_str(&format!("{indent}</defs>\n"));
            }

            if has_kids {
                out.push_str(&format!(
                    "{indent}<g{opacity}{shadow}>\n",
                    opacity = opacity_attr, shadow = shadow_filter,
                ));
                out.push_str(&format!(
                    r#"{indent}  <rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}"{stroke}{radius} />"#,
                    x = rect.x, y = rect.y, w = rect.w, h = rect.h,
                    fill = fill_attr, stroke = stroke_attr, radius = radius_attr,
                ));
                out.push('\n');
                let child_indent = format!("{indent}  ");
                for child in &node.children {
                    render_node(child, layout, out, &child_indent);
                }
                out.push_str(&format!("{indent}</g>\n"));
            } else {
                out.push_str(&format!(
                    r#"{indent}<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}"{stroke}{radius}{opacity}{shadow} />"#,
                    x = rect.x, y = rect.y, w = rect.w, h = rect.h,
                    fill = fill_attr, stroke = stroke_attr, radius = radius_attr,
                    opacity = opacity_attr, shadow = shadow_filter,
                ));
                out.push('\n');
            }
        }

        IrNodeData::Text(t) => {
            let fs = t.font_size.unwrap_or(14.0);
            let fw = t.font_weight.as_ref()
                .map(|w| format!("{:?}", w).to_lowercase())
                .unwrap_or_else(|| "normal".into());
            let ff = t.font_family.as_deref().unwrap_or("sans-serif");
            let color = t.color.as_ref().map(|c| c.to_hex()).unwrap_or_else(|| "#000000".into());

            let font = make_font(t.font_family.as_deref(), &t.font_weight, fs as f32);
            let metrics = font.metrics();
            let text_h = -metrics.1.ascent + metrics.1.descent;
            let ty = rect.y + (rect.h - text_h) / 2.0 - metrics.1.ascent;

            let (tx, anchor) = match t.text_align {
                Some(pastel_lang::ir::style::TextAlign::Center) => (rect.x + rect.w / 2.0, "middle"),
                Some(pastel_lang::ir::style::TextAlign::Right) => (rect.x + rect.w, "end"),
                _ => (rect.x, "start"),
            };

            out.push_str(&format!(
                r#"{indent}<text x="{tx}" y="{ty}" font-size="{fs}" font-weight="{fw}" font-family="{ff}" fill="{color}" text-anchor="{anchor}">{content}</text>"#,
                content = escape_xml(&t.content),
            ));
            out.push('\n');
        }

        IrNodeData::Image(img) => {
            let radius_attr = corner_radius_attr(img.corner_radius.as_ref().map(|r| &r.0), rect);
            out.push_str(&format!(
                "{indent}<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill=\"#E8E8E8\" stroke=\"#D0D0D0\"{radius} />",
                x = rect.x, y = rect.y, w = rect.w, h = rect.h, radius = radius_attr,
            ));
            out.push('\n');
            let label = img.name.as_deref().unwrap_or("image");
            out.push_str(&format!(
                "{indent}<text x=\"{tx}\" y=\"{ty}\" font-size=\"12\" fill=\"#AAAAAA\" text-anchor=\"middle\" dominant-baseline=\"central\">{label}</text>",
                tx = rect.x + rect.w / 2.0, ty = rect.y + rect.h / 2.0,
            ));
            out.push('\n');
        }

        IrNodeData::Shape(s) => {
            let fill_attr = s.visual.fill.as_ref().map(fill_str).unwrap_or("none".into());
            match s.shape_type {
                ShapeType::Ellipse => {
                    out.push_str(&format!(
                        r#"{indent}<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{fill}" />"#,
                        cx = rect.x + rect.w / 2.0, cy = rect.y + rect.h / 2.0,
                        rx = rect.w / 2.0, ry = rect.h / 2.0, fill = fill_attr,
                    ));
                }
                _ => {
                    let radius_attr = corner_radius_attr(s.visual.corner_radius.as_ref().map(|r| &r.0), rect);
                    out.push_str(&format!(
                        r#"{indent}<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}"{radius} />"#,
                        x = rect.x, y = rect.y, w = rect.w, h = rect.h,
                        fill = fill_attr, radius = radius_attr,
                    ));
                }
            }
            out.push('\n');
        }
    }
}

fn fill_str(fill: &Fill) -> String {
    match fill {
        Fill::Solid { color } => color.to_hex(),
        Fill::Transparent => "none".into(),
    }
}

fn corner_radius_attr(cr: Option<&[f64; 4]>, rect: Rect) -> String {
    match cr {
        Some(r) => {
            let max_r = (rect.w.min(rect.h) / 2.0) as f64;
            let vals: Vec<f64> = r.iter().map(|v| v.min(max_r)).collect();
            if vals[0] == vals[1] && vals[1] == vals[2] && vals[2] == vals[3] && vals[0] > 0.0 {
                format!(r#" rx="{}" ry="{}""#, vals[0], vals[0])
            } else {
                String::new() // SVG rect doesn't support per-corner radius natively
            }
        }
        None => String::new(),
    }
}

fn shadow_filter_attr(shadow: Option<&pastel_lang::ir::style::Shadow>, id: &str) -> String {
    if shadow.is_some() {
        format!(r#" filter="url(#shadow-{})""#, id)
    } else {
        String::new()
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
