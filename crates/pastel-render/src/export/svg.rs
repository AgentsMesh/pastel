use pastel_lang::ir::node::{IrNode, IrNodeData, ShapeType};
use pastel_lang::ir::IrDocument;

use super::svg_effects::{
    blend_attr, blur_filter, dash_attr, fill_str, rotation_attr, shadow_filter_attr,
    write_shadow_def,
};
use super::svg_text::{render_image, render_text};
use crate::layout::{LayoutTree, Rect};

/// Generate SVG string from IR document.
pub fn export_svg(doc: &IrDocument) -> String {
    export_svg_nodes(doc, &doc.nodes)
}

/// Generate SVG string from specific nodes.
pub fn export_svg_nodes(doc: &IrDocument, nodes: &[IrNode]) -> String {
    let w = doc.canvas.width;
    let h = doc.canvas.height;

    let mut surface = skia_safe::surfaces::raster_n32_premul((w as i32, h as i32))
        .expect("failed to create surface for layout");
    let layout = LayoutTree::compute_nodes(nodes, w, h, surface.canvas());

    let bg = doc
        .canvas
        .background
        .as_ref()
        .map(|c| c.to_hex())
        .unwrap_or_else(|| "#FFFFFF".into());

    let mut defs = String::new();
    let mut out = String::new();
    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#
    ));
    out.push('\n');
    out.push_str(&format!(
        r#"  <rect width="{w}" height="{h}" fill="{bg}" />"#
    ));
    out.push('\n');

    for node in nodes {
        render_node(node, &layout, &mut out, &mut defs, "  ", &doc.assets);
    }

    if !defs.is_empty() {
        let mut result = String::new();
        let svg_line_end = out.find('\n').unwrap_or(0) + 1;
        result.push_str(&out[..svg_line_end]);
        result.push_str("  <defs>\n");
        result.push_str(&defs);
        result.push_str("  </defs>\n");
        result.push_str(&out[svg_line_end..]);
        result.push_str("</svg>\n");
        result
    } else {
        out.push_str("</svg>\n");
        out
    }
}

fn render_node(
    node: &IrNode,
    layout: &LayoutTree,
    out: &mut String,
    defs: &mut String,
    indent: &str,
    assets: &[pastel_lang::ir::IrAsset],
) {
    let rect = match layout.get(&node.id) {
        Some(r) => *r,
        None => return,
    };
    match &node.data {
        IrNodeData::Frame(f) => render_frame(node, f, rect, layout, out, defs, indent, assets),
        IrNodeData::Text(t) => render_text(t, rect, node, out, defs, indent),
        IrNodeData::Image(img) => render_image(img, rect, out, indent, corner_radius_attr, assets),
        IrNodeData::Shape(s) => render_shape(s, rect, node, out, defs, indent),
    }
}

#[allow(clippy::too_many_arguments)]
fn render_frame(
    node: &IrNode,
    f: &pastel_lang::ir::node::FrameData,
    rect: Rect,
    layout: &LayoutTree,
    out: &mut String,
    defs: &mut String,
    indent: &str,
    assets: &[pastel_lang::ir::IrAsset],
) {
    let has_kids = !node.children.is_empty();
    let fill_a = f
        .visual
        .fill
        .as_ref()
        .map(|fill| fill_str(fill, &node.id, defs))
        .unwrap_or("none".into());
    let stroke_a = f
        .visual
        .stroke
        .as_ref()
        .map(|s| {
            format!(
                r#" stroke="{}" stroke-width="{}""#,
                s.color.to_hex(),
                s.width
            )
        })
        .unwrap_or_default();
    let opacity_a = f
        .visual
        .opacity
        .filter(|&o| o < 1.0)
        .map(|o| format!(r#" opacity="{}""#, o))
        .unwrap_or_default();
    let radius_a = corner_radius_attr(f.visual.corner_radius.as_ref().map(|r| &r.0), rect);
    let rot_a = rotation_attr(f.rotation, rect);
    let blend_a = blend_attr(&f.visual);
    let dash_a = dash_attr(f.visual.stroke.as_ref());
    let blur_a = blur_filter(&f.visual, &node.id, defs);
    let shadow_a = shadow_filter_attr(f.visual.shadow.as_ref(), &node.id);
    let filter_a = if !blur_a.is_empty() { blur_a } else { shadow_a };

    if let Some(shadow) = &f.visual.shadow {
        write_shadow_def(defs, &node.id, shadow);
    }

    if has_kids {
        out.push_str(&format!(
            "{indent}<g{opacity_a}{filter_a}{rot_a}{blend_a}>\n"
        ));
        out.push_str(&format!(
            r#"{indent}  <rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill_a}"{stroke_a}{dash_a}{radius_a} />"#,
            x = rect.x, y = rect.y, w = rect.w, h = rect.h,
        ));
        out.push('\n');
        let ci = format!("{indent}  ");
        for child in &node.children {
            render_node(child, layout, out, defs, &ci, assets);
        }
        out.push_str(&format!("{indent}</g>\n"));
    } else {
        out.push_str(&format!(
            r#"{indent}<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill_a}"{stroke_a}{dash_a}{radius_a}{opacity_a}{filter_a}{rot_a}{blend_a} />"#,
            x = rect.x, y = rect.y, w = rect.w, h = rect.h,
        ));
        out.push('\n');
    }
}

fn render_shape(
    s: &pastel_lang::ir::node::ShapeData,
    rect: Rect,
    node: &IrNode,
    out: &mut String,
    defs: &mut String,
    indent: &str,
) {
    let fill_a = s
        .visual
        .fill
        .as_ref()
        .map(|fill| fill_str(fill, &node.id, defs))
        .unwrap_or("none".into());
    let stroke_a = s
        .visual
        .stroke
        .as_ref()
        .map(|st| {
            format!(
                r#" stroke="{}" stroke-width="{}""#,
                st.color.to_hex(),
                st.width
            )
        })
        .unwrap_or_default();
    let rot_a = rotation_attr(s.rotation, rect);
    let blend_a = blend_attr(&s.visual);
    let dash_a = dash_attr(s.visual.stroke.as_ref());
    let blur_a = blur_filter(&s.visual, &node.id, defs);

    match s.shape_type {
        ShapeType::Path => {
            if let Some(d) = &s.path {
                // Apply the same viewBox scaling as the Skia painter:
                // declared width/height = coordinate space, rect = layout target.
                let view_w = s.width.as_ref().and_then(|d| match d {
                    pastel_lang::ir::style::Dimension::Fixed(n) => Some(*n as f32), _ => None,
                }).unwrap_or(rect.w);
                let view_h = s.height.as_ref().and_then(|d| match d {
                    pastel_lang::ir::style::Dimension::Fixed(n) => Some(*n as f32), _ => None,
                }).unwrap_or(rect.h);

                let sx = if view_w > 0.0 { rect.w / view_w } else { 1.0 };
                let sy = if view_h > 0.0 { rect.h / view_h } else { 1.0 };
                let tx = rect.x;
                let ty = rect.y;

                let transform_a = if (sx - 1.0).abs() > 0.001 || (sy - 1.0).abs() > 0.001
                    || tx.abs() > 0.001 || ty.abs() > 0.001
                {
                    format!(r#" transform="translate({tx},{ty}) scale({sx},{sy})""#)
                } else { String::new() };

                out.push_str(&format!(
                    r#"{indent}<path d="{d}" fill="{fill_a}"{stroke_a}{dash_a}{rot_a}{blend_a}{blur_a}{transform_a} />"#,
                ));
            }
        }
        ShapeType::Ellipse => out.push_str(&format!(
            r#"{indent}<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{fill_a}"{stroke_a}{dash_a}{rot_a}{blend_a}{blur_a} />"#,
            cx = rect.x + rect.w / 2.0, cy = rect.y + rect.h / 2.0,
            rx = rect.w / 2.0, ry = rect.h / 2.0,
        )),
        _ => {
            let radius_a = corner_radius_attr(s.visual.corner_radius.as_ref().map(|r| &r.0), rect);
            out.push_str(&format!(
                r#"{indent}<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill_a}"{stroke_a}{dash_a}{radius_a}{rot_a}{blend_a}{blur_a} />"#,
                x = rect.x, y = rect.y, w = rect.w, h = rect.h,
            ));
        }
    }
    out.push('\n');
}

// -- Helpers --

pub(super) fn corner_radius_attr(cr: Option<&[f64; 4]>, rect: Rect) -> String {
    match cr {
        Some(r) => {
            let max_r = (rect.w.min(rect.h) / 2.0) as f64;
            let vals: Vec<f64> = r.iter().map(|v| v.min(max_r)).collect();
            if vals[0] == vals[1] && vals[1] == vals[2] && vals[2] == vals[3] && vals[0] > 0.0 {
                format!(r#" rx="{}" ry="{}""#, vals[0], vals[0])
            } else {
                String::new()
            }
        }
        None => String::new(),
    }
}
